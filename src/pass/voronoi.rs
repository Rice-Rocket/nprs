use glam::{IVec2, UVec2, Vec2, Vec3};
use rand::Rng;
use voronoi::Point;

use crate::{image::{pixel::{rgba::Rgba, Pixel as _}, sampler::{Sampler, WrapMode2D}, Image}, pass::{luminance::Luminance, tfm::TangentFlowMap}, render_graph::ANY_IMAGE};

use super::Pass;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum VoronoiRelaxWeightMode {
    /// Weights the centroids of voronoi regions based on luminance of the image. 
    /// This is only really useful with stippling.
    Luminance,
    /// Weights the centroids of voronoi regions based on frequency of the image.
    /// For stippling, this accentuates edge lines.
    /// For the mosaic, this creates smaller tiles near edges, leading to clearer edge lines.
    Frequency,
}

#[derive(Clone, Copy, PartialEq)]
pub enum VoronoiMode {
    /// Treat the centroids of the voronoi regions as stippling points.
    Stippling {
        /// The background color of the stippled image.
        background: Rgba<f32>,

        /// The color of the stippling dots.
        stipple: Rgba<f32>,

        /// The radius of the stippled dots.
        stipple_radius: f32,
    },
    /// Treat voronoi regions as tiles of a mosaic, coloring them based on the original image. 
    /// The sharpness of edges can be controlled by changing the standard deviation of the
    /// `SobelPostBlur` gaussian blur pass. A higher standard deviation of this blur makes edges
    /// sharper. A sufficient standard deviation for sharp edges is 5.
    Mosaic,
}

pub struct RelaxedVoronoi {
    /// The number of points to distribute and relax.
    points: usize,

    /// The number of iterations taken to relax the image. A value of 20 is usually more than enough, but
    /// for higher resolution images more may be required.
    relax_iterations: usize,

    /// The method with which to weight the centroids of voronoi regions during relaxation.
    relax_mode: VoronoiRelaxWeightMode,

    /// The method with which to display the final image.
    mode: VoronoiMode,

    /// The amount by which the centroids of voronoi regions are weighted. 
    ///
    /// A value of 0 uniformly distributes voronoi centroids and may be desirable to achieve
    /// certain mosaics. For nice, sharp edges on mosaics, a weight scale of 0.5 combined with a
    /// `SobelPostBlur` standard deviation of 5 creates clear and clean edges.
    ///
    /// For stippling, a value of 10 is recommended.
    weight_scale: f32,

    /// Whether or not to invert centroid weights. Recommended for stippling and not recommended
    /// for mosaics.
    invert: bool,
}

impl RelaxedVoronoi {
    pub const NAME: &'static str = "relaxedvoronoi";

    /// Create a new [`RelaxedVoronoi`] pass that performs stippling-like effect on the given number of
    /// `points`.
    pub fn stipple(points: usize) -> RelaxedVoronoi {
        RelaxedVoronoi {
            points,
            relax_iterations: 20,
            relax_mode: VoronoiRelaxWeightMode::Luminance,
            mode: VoronoiMode::Stippling {
                background: Rgba::new(1.0, 1.0, 1.0, 1.0),
                stipple: Rgba::new(0.0, 0.0, 0.0, 1.0),
                stipple_radius: 1.0,
            },
            weight_scale: 10.0,
            invert: true,
        }
    }

    /// Create a new [`RelaxedVoronoi`] pass that performs mosaic-like effect on the given number of
    /// `points`.
    pub fn mosaic(points: usize) -> RelaxedVoronoi {
        RelaxedVoronoi {
            points,
            relax_iterations: 20,
            relax_mode: VoronoiRelaxWeightMode::Frequency,
            mode: VoronoiMode::Mosaic,
            weight_scale: 0.5,
            invert: false,
        }
    }

    /// The background color of the stippled image.
    ///
    /// Defaults to `Rgba(1.0, 1.0, 1.0, 1.0)`
    pub fn background_color(mut self, color: Rgba<f32>) -> Self {
        if let VoronoiMode::Stippling {
            background,
            stipple,
            stipple_radius,
        } = &mut self.mode {
            *background = color;
        }

        self
    }

    /// The color of the stippling dots.
    ///
    /// Defaults to `Rgba(0.0, 0.0, 0.0, 1.0)`
    pub fn stipple_color(mut self, color: Rgba<f32>) -> Self {
        if let VoronoiMode::Stippling {
            background,
            stipple,
            stipple_radius,
        } = &mut self.mode {
            *stipple = color;
        }

        self
    }

    /// The radius of the stippled dots.
    ///
    /// Defaults to `1.0`
    pub fn stipple_radius(mut self, radius: f32) -> Self {
        if let VoronoiMode::Stippling {
            background,
            stipple,
            stipple_radius,
        } = &mut self.mode {
            *stipple_radius = radius;
        }

        self
    }

    /// The number of iterations taken to relax the image. A value of 20 is usually more than enough, but
    /// for higher resolution images more may be required.
    ///
    /// Defaults to `20`
    pub fn relax_iterations(mut self, iterations: usize) -> Self {
        self.relax_iterations = iterations;
        self
    }

    /// Invert centroid weights. Recommended for stippling and not recommended
    /// for mosaics.
    ///
    /// Default for stipple.
    pub fn invert(mut self) -> Self {
        self.invert = true;
        self
    }

    /// Does not invert centroid weights. Recommended for mosaics and not recommended
    /// for stippling.
    ///
    /// Default for mosaic.
    pub fn no_invert(mut self) -> Self {
        self.invert = false;
        self
    }

    /// The amount by which the centroids of voronoi regions are weighted. 
    ///
    /// A value of 0 uniformly distributes voronoi centroids and may be desirable to achieve
    /// certain mosaics. For nice, sharp edges on mosaics, a weight scale of 0.5 combined with a
    /// `SobelPostBlur` standard deviation of 5 creates clear and clean edges.
    ///
    /// For stippling, a value of 10 is recommended.
    ///
    /// Defaults to `10` for stipple and `0.5` for mosaic.
    pub fn weight_scale(mut self, weight_scale: f32) -> Self {
        self.weight_scale = weight_scale;
        self
    }

    /// Weights the centroids of voronoi regions based on frequency of the image.
    /// For stippling, this accentuates edge lines.
    /// For the mosaic, this creates smaller tiles near edges, leading to clearer edge lines.
    ///
    /// Default for mosaic.
    pub fn relax_with_frequency(mut self) -> Self {
        self.relax_mode = VoronoiRelaxWeightMode::Frequency;
        self
    }

    /// Weights the centroids of voronoi regions based on luminance of the image. 
    /// This is only really useful with stippling.
    ///
    /// Default for stipple.
    pub fn relax_with_luminance(mut self) -> Self {
        self.relax_mode = VoronoiRelaxWeightMode::Luminance;
        self
    }
}

impl Pass for RelaxedVoronoi {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        if let VoronoiRelaxWeightMode::Luminance = self.relax_mode {
            vec![ANY_IMAGE, Luminance::NAME]
        } else {
            vec![ANY_IMAGE, TangentFlowMap::NAME]
        }
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        let source = aux_images[0];
        let weights = aux_images[1];

        let mut rng = rand::thread_rng();
        let res = target.resolution();

        // Initialize seed points
        let mut seeds = Vec::new();
        while seeds.len() < self.points {
            let u: f32 = rng.gen();
            let v: f32 = rng.gen();
            let l = if let VoronoiRelaxWeightMode::Luminance = self.relax_mode {
                weights.sample(Vec2::new(u, v), Sampler::LINEAR_CLAMP).r
            } else {
                weights.sample(Vec2::new(u, v), Sampler::LINEAR_CLAMP).b
            };

            if l < rng.gen() {
                seeds.push(voronoi::Point::new((u * res.x as f32) as f64, (v * res.y as f32) as f64));
            }
        }
        
        // Relax voronoi diagram
        for i in 0..self.relax_iterations {
            let vor_diagram = voronoi::voronoi(seeds, res.x as f64);
            let faces = voronoi::make_polygons(&vor_diagram);

            let mut new_points = Vec::new();
            for face in faces {
                let poly = UnsortedPolygon::from_face(face);
                let sorted_poly = poly.sort();
                let cr = weighted_centroid(
                    &sorted_poly,
                    res.x as f32,
                    weights,
                    self.invert,
                    self.weight_scale,
                    self.relax_mode == VoronoiRelaxWeightMode::Luminance,
                );

                new_points.push(cr);
            }

            seeds = new_points;
        }

        match self.mode {
            VoronoiMode::Stippling {
                background,
                stipple,
                stipple_radius,
            } => {
                *target = Image::new_fill(target.resolution(), background);

                // Draw relaxed centroids
                for p in seeds {
                    let p = Vec2::new(f64::try_from(p.x).unwrap() as f32, f64::try_from(p.y).unwrap() as f32);

                    // TODO: Antialiased circle
                    for x in (p.x - stipple_radius).floor() as i32..(p.x + stipple_radius).ceil() as i32 {
                        for y in (p.y - stipple_radius).floor() as i32..(p.y + stipple_radius).ceil() as i32 {
                            if p.distance_squared(Vec2::new(x as f32, y as f32)) < stipple_radius * stipple_radius {
                                target.store_wrapped(IVec2::new(x, y), stipple);
                            }
                        }
                    }
                }
            },
            VoronoiMode::Mosaic => {
                // Draw voronoi regions based on relaxed centroids
                let vor_diagram = voronoi::voronoi(seeds, res.x as f64);
                let cells = voronoi::make_polygons(&vor_diagram);

                for face in cells {
                    let sorted = UnsortedPolygon::from_face(face).sort();
                    let centroid = weighted_centroid(
                        &sorted,
                        res.x as f32,
                        weights,
                        self.invert,
                        self.weight_scale,
                        self.relax_mode == VoronoiRelaxWeightMode::Luminance,
                    );

                    let c = Vec2::new(f64::try_from(centroid.x).unwrap() as f32, f64::try_from(centroid.y).unwrap() as f32);
                    let color = source.load_wrapped(c.round().as_ivec2(), WrapMode2D::CLAMP);

                    // Draw the polygon
                    let bbox = polygon_raster_bbox(&sorted);
                    for y in bbox.1.x..bbox.1.y {
                        let nodes = scanline_nodes(&sorted, y as f32, res.x as f32);
                        if !nodes.is_empty() {
                            // Draw the scanline
                            let line = Line::from_nodes(nodes);

                            let x1 = line.p1.x as i32;
                            let y1 = line.p1.y as i32;
                            let x2 = line.p2.x as i32;
                            let y2 = line.p2.y as i32;

                            let dx = x2 - x1;
                            let dy = y2 - y1;

                            for x in i32::min(x1, x2)..i32::max(x1, x2) {
                                let y = y1 + dy * (x - x1) / dx;
                                target.store_wrapped(IVec2::new(x, y), color);
                            }
                        }
                    }
                }
            },
        }
    }
}

struct SortedPolygon {
    vertices: Vec<Vec2>
}

struct UnsortedPolygon {
    vertices: Vec<Vec2>,
}

struct Line {
    p1: Vec2,
    p2: Vec2,
}

impl SortedPolygon {
    fn create_edges(&self) -> Vec<Line> {
        let n = self.vertices.len();
        let mut lines = Vec::new();
        for i in 0..(n - 1) {
            lines.push(Line {
                p1: self.vertices[i],
                p2: self.vertices[i + 1],
            });
        }

        lines.push(Line {
            p1: self.vertices[n - 1],
            p2: self.vertices[0],
        });

        lines
    }
}

impl UnsortedPolygon {
    fn from_face(face: Vec<voronoi::Point>) -> Self {
        let vertices = face.iter()
            .map(|x| Vec2::new(f64::try_from(x.x).unwrap() as f32, f64::try_from(x.y).unwrap() as f32))
            .collect();

        Self {
            vertices
        }
    }

    fn vertex_centroid(&self) -> Vec2 {
        let mut p = Vec2::ZERO;
        for &v in self.vertices.iter() {
            p += v;
        }

        p / self.vertices.len() as f32
    }

    fn sort(mut self) -> SortedPolygon {
        let mut sorted = Vec::new();
        let centroid = self.vertex_centroid();

        let v_a = Vec3::new(centroid[0] + 300.0, centroid[1], 0.0);
        for v in self.vertices {
            let v_b = Vec3::new(v.x - centroid.x, v.y - centroid.y, 0.0);

            let numerator = v_a.dot(v_b);
            let denominator = v_a.length() * v_b.length();

            let mut theta = (numerator / denominator).acos();
            let s = v_a.cross(v_b);
            let sign = s / s.length();
            theta *= sign.z;
            sorted.push((v, theta))
        }

        radsort::sort_by_key(&mut sorted, |k| k.1);
        let vertices = sorted.into_iter().map(|x| x.0).collect();

        SortedPolygon { vertices }
    }
}

impl Line {
    fn from_nodes(nodes: Vec<IVec2>) -> Self {
        Self {
            p1: nodes[0].as_vec2(),
            p2: nodes[1].as_vec2(),
        }
    }

    fn intersect(self, other: Self) -> IVec2 {
        let x1 = self.p1.x;
        let y1 = self.p1.y;
        let x2 = self.p2.x;
        let y2 = self.p2.y;

        let x3 = other.p1.x;
        let y3 = other.p1.y;
        let x4 = other.p2.x;
        let y4 = other.p2.y;

        let px = ((x1 * y2 - y1 * x2) * (x3 - x4) - (x1 - x2) * (x3 * y4 - y3 * x4))
            / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));

        let py = ((x1 * y2 - y1 * x2) * (y3 - y4) - (y1 - y2) * (x3 * y4 - y3 * x4))
            / ((x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4));

        Vec2::new(px, py).round().as_ivec2()
    }
}

fn polygon_raster_bbox(poly: &SortedPolygon) -> (IVec2, IVec2) {
    let edges = poly.create_edges();

    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();

    for e in edges {
        x_vals.push(e.p1.x);
        x_vals.push(e.p2.x);
        y_vals.push(e.p1.y);
        y_vals.push(e.p2.y);
    }

    let n = x_vals.len();
    x_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    y_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let x_min = x_vals[0];
    let y_min = y_vals[0];
    let x_max = x_vals[n - 1];
    let y_max = y_vals[n - 1];

    (
        Vec2::new(x_min, x_max + 1.0).round().as_ivec2(),
        Vec2::new(y_min, y_max + 1.0).round().as_ivec2(),
    )
}

fn scanline_nodes(poly: &SortedPolygon, scan_y: f32, width: f32) -> Vec<IVec2> {
    let mut nodes = Vec::new();
    let mut p1 = Vec2::ZERO;
    let mut p2 = Vec2::ZERO;

    for edge in poly.create_edges() {
        p1.y = edge.p1.y;
        p2.y = edge.p2.y;

        if p1.y < scan_y && p2.y >= scan_y || p1.y >= scan_y && p2.y < scan_y {
            p1.x = edge.p1.x;
            p2.x = edge.p2.x;

            if p1.x < 0.0 {
                p1.x = 0.0;
            }

            let scanline = Line { p1, p2 };
            let line = Line { p1: Vec2::new(0.0, scan_y), p2: Vec2::new(width, scan_y) };

            let mut node = scanline.intersect(line);
            node.x += 1;
            nodes.push(node);
        }
    }

    nodes
}

fn weighted_centroid(
    poly: &SortedPolygon,
    width: f32,
    weights: &Image<4, f32, Rgba<f32>>,
    invert: bool,
    scale: f32,
    luminance: bool,
) -> Point {
    let bbox = polygon_raster_bbox(poly);

    let mut c = Vec2::ZERO;
    let mut pixel_count = 0;
    let mut total_weight = 0.0;

    for y in bbox.1.x..bbox.1.y - 1 {
        let nodes = scanline_nodes(poly, y as f32, width);
        if !nodes.is_empty() {
            let n_a = nodes[0].x;
            let n_b = nodes[1].x;
            let x1 = i32::min(n_a, n_b);
            let x2 = i32::max(n_a, n_b);

            for x in x1..x2 - 1 {
                let mut weight = if luminance {
                    weights.load_wrapped(IVec2::new(x, y), WrapMode2D::CLAMP).r
                } else {
                    weights.load_wrapped(IVec2::new(x, y), WrapMode2D::CLAMP).b
                };

                if invert {
                    weight = 1.0 - weight;
                }

                weight = weight.powf(scale);

                c += weight * Vec2::new(x as f32, y as f32);

                total_weight += weight;
                pixel_count += 1;
            }
        }
    }

    if pixel_count == 0 || total_weight == 0.0 {
        voronoi::Point::new(0.0, 0.0)
    } else {
        c /= total_weight;
        voronoi::Point::new(c.x as f64, c.y as f64)
    }
}
