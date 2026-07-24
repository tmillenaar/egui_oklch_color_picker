#[derive(Debug, Clone, PartialEq)]
pub struct OklchUiState {
    // normalized 0..1
    pub l: f32,
    pub c: f32,
    pub h: f32,
    pub a: f32,
}
