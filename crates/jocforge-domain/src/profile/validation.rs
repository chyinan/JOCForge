// pattern: Functional Core
//! Validation-only render selection; production resolution never consumes this type.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderTarget {
    OpenJoc714,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidationProfile {
    render_target: RenderTarget,
}
impl ValidationProfile {
    pub const fn new(render_target: RenderTarget) -> Self {
        Self { render_target }
    }
}
