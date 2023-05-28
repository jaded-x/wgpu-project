#[repr(align(16))]
#[derive(Debug, Copy, Clone)]
pub struct Align16<T>(pub T);