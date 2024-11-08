use super::Pass;

pub struct Luminance {

}

impl Luminance {
    const NAME: &'static str = "luminance";
}

impl Pass for Luminance {
    fn name() -> &'static str {
        Self::NAME
    }

    fn dependencies(&self) -> &[&'static str] {
        &[]
    }
}
