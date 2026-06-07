pub trait Processor {
    type Input;
    type Output;

    fn process(&self, input: &Self::Input) -> Self::Output;
}

pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}