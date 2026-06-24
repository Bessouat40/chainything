use chainything::processors::greyscale_processor::RawImage;

#[derive(Clone)]
pub enum MyNode {
    TextInputProcessor(String),
    ImageReaderProcessor(String),
    ImageDisplay(String),
}
impl MyNode {
    const fn _name(&self) -> &str {
        match self {
            MyNode::ImageReaderProcessor(_) => "ImageReaderProcessor",
            MyNode::TextInputProcessor(_) => "TextInputProcessor",
            MyNode::ImageDisplay(_) => "ImageDisplay",
        }
    }

    pub fn string_in(&self) -> &String {
        match self {
            MyNode::ImageReaderProcessor(value) => value,
            MyNode::TextInputProcessor(value) => value,
            MyNode::ImageDisplay(value) => value,
        }
    }

    pub fn _image_out(&self) -> Option<&RawImage> {
        match self {
            MyNode::ImageReaderProcessor(_) => None,
            _ => None,
        }
    }

}