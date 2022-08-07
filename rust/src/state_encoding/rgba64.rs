//!
//! Each pixel is encoded into 4 bytes for the color channels red, green, blue and alpha whereby alpha is always 255.
//! These bytes are then simply appended to each other in row-major order.
//! At the end everything is base64 encoded.
//!

use super::Encoder;
use crate::pixmap::Color;

/// An encoder that implements *rgba64* encoding.
/// See module level documentation for more details.
#[derive(Debug, Copy, Clone)]
pub struct Rgba64Encoder {}

impl Encoder for Rgba64Encoder {
    type ResultFormat = String;

    fn encode(pixmap_width: usize, pixmap_height: usize, pixmap_data: &[Color]) -> Self::ResultFormat {
        let mut result_data = Vec::with_capacity(pixmap_width * pixmap_height * 4);

        for i in pixmap_data {
            let i: u32 = i.into();
            let color = i.to_le_bytes();
            result_data.push(color[0]);
            result_data.push(color[1]);
            result_data.push(color[2]);
            result_data.push(255);
        }

        base64::encode(&result_data)
    }

    fn decode(_data: &Self::ResultFormat) -> anyhow::Result<Vec<Color>> {
        todo!()
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use crate::pixmap::{Color, InMemoryPixmap};

    use super::*;

    #[test]
    fn test_encoded_content_has_correct_length() {
        let pixmap = SharedPixmap::<InMemoryPixmap>::default();
        let encoded = encode(&pixmap);
        let encoded_bytes = base64::decode(&encoded).unwrap();
        assert_eq!(
            encoded_bytes.len(),
            pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1 * 4
        )
    }

    quickcheck! {
        fn test_encoded_color_is_correctly_decodable(x: usize, y: usize, color: u32) -> TestResult {
            // prepare
            let pixmap = SharedPixmap::<InMemoryPixmap>::default();
            let color = color.into();
            if pixmap.set_pixel(x, y, color).is_err() {
                return TestResult::discard()
            }

            // execute
            let encoded = encode(&pixmap);
            let encoded_bytes = base64::decode(&encoded).unwrap();

            // verify
            let i  = (y * pixmap.get_size().unwrap().0 + x) * 4;
            let encoded_color = &encoded_bytes[i..i+3];
            let decoded_color = Color(encoded_color[0], encoded_color[1], encoded_color[2]);
            TestResult::from_bool(decoded_color == color)
        }
    }
}
