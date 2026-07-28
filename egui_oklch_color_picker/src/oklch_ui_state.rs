use crate::oklch::Oklch;

#[derive(Debug, Clone, PartialEq)]
pub struct OklchUiState {
    // normalized 0..1
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub a: f32,
}

impl OklchUiState {
    pub fn to_oklch(&self) -> Oklch {
        Oklch {
            l: self.l,
            c: self.c * 0.325,
            h: self.h * std::f32::consts::TAU,
            alpha: self.a,
        }
    }
}
