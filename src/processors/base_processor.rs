pub trait Processor {
    type Input: 'static;
    type Output: 'static;

    fn process(&self, input: &Self::Input, parameters: Option<std::collections::HashMap<String, Box<dyn std::any::Any>>>) -> Self::Output;
}

pub struct RawImage {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}