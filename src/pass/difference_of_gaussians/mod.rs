use std::f32::consts::PI;

use aa::FDoGAntiAlias;
use blur1::FDoGBlur1;
use glam::{Vec2, Vec4, Vec4Swizzles};
use nprs_derive::FromParsedValue;
use threshold::FDoGBlur2Theshold;

use crate::{image::{pixel::rgba::Rgba, Image}, render_graph::ANY_IMAGE};

use super::{tfm::TangentFlowMap, Pass, SubPass};

mod blur1;
mod threshold;
mod aa;

#[derive(FromParsedValue)]
#[nprs(from = DifferenceOfGaussiansBuilder)]
pub struct DifferenceOfGaussians {
    blur1: FDoGBlur1,
    threshold: FDoGBlur2Theshold,
    aa: FDoGAntiAlias,
}

impl DifferenceOfGaussians {
    pub const NAME: &'static str = "difference_of_gaussians";
    
    pub fn new() -> Self {
        Self {
            blur1: FDoGBlur1 {
                sigma_e: 2.0,
                k: 1.6,
                tau: 100.0,
            },
            threshold: FDoGBlur2Theshold {
                sigma_m: 2.0,
                integral_convolution_stepsizes: Vec2::ONE,
                threshold_mode: FDoGThresholdMode::HyperbolicTangent {
                    phi: 5.0,
                    white_point: 0.5,
                },
                invert: false,
            },
            aa: FDoGAntiAlias {
                sigma_a: 2.0,
                integral_convolution_stepsizes: Vec2::ONE,
            },
        }
    }

    /// The standard deviation to use when computing the difference of gaussians.
    ///
    /// Larger values (>5) lead to less detailed edges, giving a blotchy look. Smaller values (<2) lead to more detailed edges, giving a finer look.
    ///
    /// Defaults to `2.0`
    pub fn dog_deviation(mut self, sigma_e: f32) -> Self {
        self.blur1.sigma_e = sigma_e;
        self
    }

    /// The standard deviation to use when computing the line integral.
    ///
    /// This parameter has very little effect on the final image, with very large values (>50)
    /// giving only a slightly dirtier look.
    ///
    /// Defaults to `2.0`
    pub fn line_integral_deviation(mut self, sigma_m: f32) -> Self {
        self.threshold.sigma_m = sigma_m;
        self
    }

    /// The standard deviation to use when performing the anti-aliasing cross-edge blur.
    ///
    /// Larger values lead to slightly smoother edges, but a value of `2` is usually sufficient for
    /// clean anti-aliasing.
    ///
    /// Defaults to `2.0`
    pub fn edge_smooth_deviation(mut self, sigma_a: f32) -> Self {
        self.aa.sigma_a = sigma_a;
        self
    }

    /// The amount by which to scale the standard deviation when computing the difference of
    /// gaussians.
    ///
    /// Larger values capture finer edge lines and result in more contours. Smaller values create
    /// more of a shading effect. Values less than one again capture finer edge lines and result in
    /// more contours, while also capturing subtle shading. 
    ///
    /// A value of `0.8` captures contours and provides nice shading.
    /// A value of `5.0` captures more contours, but loses some shading detail.
    /// A value of `1.6` captures some contours, but focuses more on shading.
    ///
    /// Defaults to `1.6`
    pub fn sigma_scale(mut self, k: f32) -> Self {
        self.blur1.k = k;
        self
    }

    /// The sharpness coefficient of the difference of gaussians.
    ///
    /// With low DoG standard deviations, larger values (>500) accentuate edge lines while smaller values soften the image.
    /// With high DoG standard deviations, larger values (>500) discard edge lines.
    ///
    /// Defaults to `100.0`
    pub fn sharpness(mut self, tau: f32) -> Self {
        self.blur1.tau = tau;
        self
    }

    /// Whether of not to invert the result of the difference of gaussians.
    ///
    /// Defaults to `false`
    pub fn invert(mut self, invert: bool) -> Self {
        self.threshold.invert = invert;
        self
    }
    
    /// Set thresholding mode to `None`.
    pub fn threshold_none(mut self) -> Self {
        self.threshold.threshold_mode = FDoGThresholdMode::None;
        self
    }

    /// Set thresholding mode to `HyperbolicTangent`, given a `white_point` (hard thresholding
    /// value) and `phi` (soft thresholding value).
    ///
    /// Smaller values of `white_point` (<0.25) better isolate edge lines.
    /// Large values of `phi` (>10) create hard contrast around the `white_point`. Smaller values
    /// of `phi` (<5) soften the image around the `white_point` and reveal more of the underlying
    /// image.
    ///
    /// - Default `white_point` is `0.5`
    /// - Default `phi` is `5.0`
    pub fn threshold_hyperbolic_tangent(mut self, white_point: f32, phi: f32) -> Self {
        self.threshold.threshold_mode = FDoGThresholdMode::HyperbolicTangent { white_point, phi };
        self
    }

    /// Set thresholding mode to `Quantization`, given a `white_point` (hard thresholding value),
    /// `phi` (soft thresholding value) and `palette_size` (number of colors to use).
    ///
    /// Smaller values of `white_point` (<0.25) better isolate edge lines. Larger values of
    /// `white_point` leave more room for dark regions which will be quantized.
    /// Larger values of `phi` (>1) condense quantized regions, leading to fewer 'in-between'
    /// values that aren't white or black. Smaller values of `phi` (<1)
    ///
    /// - Default `white_point` is `0.5`
    /// - Default `palette_size` is `3`
    /// - Default `phi` is `1.0`
    pub fn threshold_quantization(mut self, white_point: f32, palette_size: f32, phi: f32) -> Self {
        self.threshold.threshold_mode = FDoGThresholdMode::Quantization { white_point, palette_size, phi };
        self
    }

    /// Set thresholding mode to `SmoothQuantization`, given the `palette_size` (number of colors
    /// to use) and `phi` (soft thresholding value).
    ///
    /// - Default `palette_size` is `3`
    /// - Default `phi` is `5.0`
    pub fn threshold_smooth_quantization(mut self, palette_size: f32, phi: f32) -> Self {
        self.threshold.threshold_mode = FDoGThresholdMode::SmoothQuantization { palette_size, phi };
        self
    }

    /// Set thresholding mode to `WhitePoint`, given the hard thresholding value.
    ///
    /// - Default `white_point` is 0.5
    pub fn threshold_white_point(mut self, white_point: f32) -> Self {
        self.threshold.threshold_mode = FDoGThresholdMode::WhitePoint { white_point };
        self
    }

    pub fn integral_convolution_stepsizes(mut self, stepsizes: Vec4) -> Self {
        self.threshold.integral_convolution_stepsizes = stepsizes.xy();
        self.aa.integral_convolution_stepsizes = stepsizes.zw();
        self
    }

    fn threshold_mode(mut self, mode: FDoGThresholdMode) -> Self {
        self.threshold.threshold_mode = mode;
        self
    }
}

impl Pass for DifferenceOfGaussians {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![ANY_IMAGE, TangentFlowMap::NAME]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];
        let tfm = aux_images[1];

        self.blur1.apply_subpass(target, &[source, tfm]);
        self.threshold.apply_subpass(target, &[tfm]);
    }
}

#[derive(FromParsedValue)]
pub enum FDoGThresholdMode {
    HyperbolicTangent {
        white_point: f32,
        phi: f32,
    },
    Quantization {
        white_point: f32,
        palette_size: f32,
        phi: f32,
    },
    SmoothQuantization {
        palette_size: f32,
        phi: f32,
    },
    WhitePoint {
        white_point: f32,
    },
    None,
}

fn gaussian(sigma: f32, x: f32) -> f32 {
    (1.0 / f32::sqrt(2.0 * PI * sigma * sigma)) * f32::exp(-(x * x) / (2.0 * sigma * sigma))
}

#[derive(FromParsedValue)]
pub struct DifferenceOfGaussiansBuilder {
    #[nprs(alias = dog_deviation)]
    sigma_e: f32,
    #[nprs(alias = sigma_scale)]
    k: f32,
    #[nprs(alias = sharpness)]
    tau: f32,
    #[nprs(alias = line_integral_deviation)]
    sigma_m: f32,
    integral_convolution_stepsizes: Vec4,
    threshold_mode: FDoGThresholdMode,
    invert: bool,
    #[nprs(alias = edge_smooth_deviation)]
    sigma_a: f32,
}

impl From<DifferenceOfGaussiansBuilder> for DifferenceOfGaussians {
    fn from(builder: DifferenceOfGaussiansBuilder) -> Self {
        DifferenceOfGaussians::new()
            .dog_deviation(builder.sigma_e)
            .sigma_scale(builder.k)
            .sharpness(builder.tau)
            .line_integral_deviation(builder.sigma_m)
            .integral_convolution_stepsizes(builder.integral_convolution_stepsizes)
            .threshold_mode(builder.threshold_mode)
            .invert(builder.invert)
            .edge_smooth_deviation(builder.sigma_a)
    }
}
