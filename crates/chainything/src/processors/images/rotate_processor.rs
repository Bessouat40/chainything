use image::{GrayImage, RgbImage};
use std::sync::Arc;

use crate::processors::base_processor::{Processor, ProcessorError};
use crate::processors::images::greyscale_processor::RawImage;

/// Rotates a [`RawImage`] by a multiple of 90 degrees (clockwise).
///
/// The angle is quantised to one of `90`, `180` or `270` degrees. For `90` and
/// `270` the output width and height are swapped. Works on both RGB and
/// greyscale images.
///
/// - **Input:** `inputs[0]` = `Arc<RawImage>`, `inputs[1]` = `Arc<u32>` (angle in degrees).
/// - **Output:** one `Arc<RawImage>`, rotated.
/// - **Errors:** [`ProcessorError::MissingInput`] if fewer than 2 inputs,
///   [`ProcessorError::InvalidInput`] if a type is wrong or the angle is not one
///   of 90, 180, 270.
pub struct RotateProcessor {
    id: String,
    input_image: Option<Arc<RawImage>>,
    angle: Option<Arc<u32>>,
    output: Option<Arc<RawImage>>,
}

impl RotateProcessor {
    pub fn new(id: String) -> Self {
        RotateProcessor {
            id,
            input_image: None,
            angle: None,
            output: None,
        }
    }

    fn rotate(image: &RawImage, angle: u32) -> RawImage {
        let is_rgb = image.pixels.len() == (image.width * image.height * 3) as usize;

        if is_rgb {
            let buf = match RgbImage::from_raw(image.width, image.height, image.pixels.clone()) {
                Some(b) => b,
                None => return image.clone(),
            };
            let rotated = match angle {
                90 => image::imageops::rotate90(&buf),
                270 => image::imageops::rotate270(&buf),
                _ => image::imageops::rotate180(&buf),
            };
            RawImage {
                width: rotated.width(),
                height: rotated.height(),
                pixels: rotated.into_raw(),
            }
        } else {
            let buf = match GrayImage::from_raw(image.width, image.height, image.pixels.clone()) {
                Some(b) => b,
                None => return image.clone(),
            };
            let rotated = match angle {
                90 => image::imageops::rotate90(&buf),
                270 => image::imageops::rotate270(&buf),
                _ => image::imageops::rotate180(&buf),
            };
            RawImage {
                width: rotated.width(),
                height: rotated.height(),
                pixels: rotated.into_raw(),
            }
        }
    }
}

impl Processor for RotateProcessor {
    fn id(&self) -> &str {
        &self.id
    }

    fn set_input(
        &mut self,
        mut inputs: Vec<Arc<dyn std::any::Any + Send + Sync>>,
    ) -> Result<(), ProcessorError> {
        if inputs.len() < 2 {
            return Err(ProcessorError::MissingInput(format!(
                "Processor {} requires 2 inputs (image, angle), got {}",
                self.id(),
                inputs.len()
            )));
        }

        let first_input = inputs.remove(0);
        let angle_input = inputs.remove(0);

        if let Ok(typed_image) = first_input.downcast::<RawImage>() {
            self.input_image = Some(typed_image);
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for image (expected RawImage) for processor {}",
                self.id()
            )));
        }

        let angle_val = if let Ok(typed_angle) = angle_input.clone().downcast::<u32>() {
            *typed_angle
        } else if let Ok(typed_string) = angle_input.downcast::<String>() {
            typed_string.parse().map_err(|_| {
                ProcessorError::InvalidInput(format!(
                    "Cannot parse angle as u32 for processor {}",
                    self.id()
                ))
            })?
        } else {
            return Err(ProcessorError::InvalidInput(format!(
                "Invalid input type for angle (expected u32 or String) for processor {}",
                self.id()
            )));
        };

        if !matches!(angle_val, 90 | 180 | 270) {
            return Err(ProcessorError::InvalidInput(format!(
                "Angle must be one of 90, 180, 270 (got {}) for processor {}",
                angle_val,
                self.id()
            )));
        }

        self.angle = Some(Arc::new(angle_val));
        Ok(())
    }

    fn get_output(&self) -> Vec<Arc<dyn std::any::Any + Send + Sync>> {
        self.output
            .clone()
            .into_iter()
            .map(|out| out as Arc<dyn std::any::Any + Send + Sync>)
            .collect()
    }

    fn process(&mut self) -> Result<(), ProcessorError> {
        let image = self.input_image.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing input image for processor {}", self.id()))
        })?;

        let angle = self.angle.as_ref().ok_or_else(|| {
            ProcessorError::MissingInput(format!("Missing angle for processor {}", self.id()))
        })?;

        let rotated = Self::rotate(image, **angle);
        self.output = Some(Arc::new(rotated));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_rgb_image(width: u32, height: u32) -> Arc<RawImage> {
        let pixels = vec![100u8; (width * height * 3) as usize];
        Arc::new(RawImage {
            width,
            height,
            pixels,
        })
    }

    #[test]
    fn test_rotate_90_swaps_dimensions() {
        let image = create_rgb_image(4, 2);
        let mut proc = RotateProcessor::new("rotate".into());
        proc.set_input(vec![image, Arc::new(90u32)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 2);
        assert_eq!(result.height, 4);
    }

    #[test]
    fn test_rotate_180_keeps_dimensions() {
        let image = create_rgb_image(4, 2);
        let mut proc = RotateProcessor::new("rotate".into());
        proc.set_input(vec![image, Arc::new(180u32)]).unwrap();
        proc.process().unwrap();

        let output = proc.get_output();
        let result = output[0].downcast_ref::<RawImage>().unwrap();
        assert_eq!(result.width, 4);
        assert_eq!(result.height, 2);
    }

    #[test]
    fn test_rotate_accepts_string_angle() {
        let image = create_rgb_image(2, 2);
        let mut proc = RotateProcessor::new("rotate".into());
        proc.set_input(vec![image, Arc::new("270".to_string())])
            .unwrap();
        assert!(proc.process().is_ok());
    }

    #[test]
    fn test_rotate_invalid_angle_returns_error() {
        let image = create_rgb_image(2, 2);
        let mut proc = RotateProcessor::new("rotate".into());
        assert!(matches!(
            proc.set_input(vec![image, Arc::new(45u32)]).unwrap_err(),
            ProcessorError::InvalidInput(_)
        ));
    }

    #[test]
    fn test_rotate_without_image_returns_error() {
        let mut proc = RotateProcessor::new("rotate".into());
        assert!(matches!(
            proc.process().unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }

    #[test]
    fn test_rotate_missing_angle_returns_error() {
        let image = create_rgb_image(2, 2);
        let mut proc = RotateProcessor::new("rotate".into());
        assert!(matches!(
            proc.set_input(vec![image]).unwrap_err(),
            ProcessorError::MissingInput(_)
        ));
    }
}
