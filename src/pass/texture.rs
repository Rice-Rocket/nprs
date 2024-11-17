use std::path::PathBuf;

use serde::{Deserialize, Deserializer};

use crate::image::{pixel::rgba::Rgba, Image};

use super::Pass;

#[derive(Deserialize)]
pub struct Texture {
    #[serde(deserialize_with = "Texture::deserialize_image")]
    im: Image<4, f32, Rgba<f32>>,
}

impl Texture {
    pub const NAME: &'static str = "texture";
    
    fn deserialize_image<'de, D>(de: D) -> Result<Image<4, f32, Rgba<f32>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut path = PathBuf::deserialize(de)?;
        let im = Image::<4, f32, Rgba<f32>>::read(path).unwrap();
        Ok(im)
    }
}

impl Pass for Texture {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    fn apply(&self, target: &mut Image<4, f32, Rgba<f32>>, _aux_images: &[&Image<4, f32, Rgba<f32>>]) {
        *target = self.im.clone();
    }
}
