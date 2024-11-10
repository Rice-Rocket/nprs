use glam::{IVec2, UVec2};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Sampler {
    pub wrap_mode: WrapMode2D,
    pub filter: Filter,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum WrapMode {
    Black,
    Clamp,
    Repeat,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct WrapMode2D {
    x: WrapMode,
    y: WrapMode,
}

impl WrapMode2D {
    pub const BLACK: WrapMode2D = WrapMode2D { x: WrapMode::Black, y: WrapMode::Black };
    pub const CLAMP: WrapMode2D = WrapMode2D { x: WrapMode::Clamp, y: WrapMode::Clamp };
    pub const REPEAT: WrapMode2D = WrapMode2D { x: WrapMode::Repeat, y: WrapMode::Repeat };

    pub fn new(x: WrapMode, y: WrapMode) -> WrapMode2D {
        WrapMode2D { x, y }
    }

    pub fn remap(self, mut p: IVec2, resolution: IVec2) -> Option<UVec2> {
        for (c, wrap) in [self.x, self.y].into_iter().enumerate() {
            if p[c] >= 0 && p[c] < resolution[c] {
                continue;
            }

            match wrap {
                WrapMode::Black => return None,
                WrapMode::Clamp => p[c] = p[c].clamp(0, resolution[c] - 1),
                WrapMode::Repeat => p[c] = modulo(p[c], resolution[c]),
            }
        }

        Some(p.as_uvec2())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Filter {
    NearestNeighbor,
    Linear,
}

fn modulo(a: i32, b: i32) -> i32 {
    let c = a - (a / b) * b;

    if c < 0 {
        c + b
    } else {
        c
    }
}
