//!
//! Each pixel is encoded into 3 bytes for the color channels red, green and blue.
// Those bytes are then simply appended to each other in row-major order.
// At the end everything is base64 encoded.
//!

use anyhow::Result;

use crate::pixmap::Color;
use crate::state_encoding::Encoder;

/// An encoder that implements *rgb64* encoding.
/// See module level documentation for more details.
#[derive(Debug, Copy, Clone)]
pub struct Rgb64Encoder {}

impl Encoder for Rgb64Encoder {
    type ResultFormat = String;

    fn encode(pixmap_width: usize, pixmap_height: usize, pixmap_data: &[Color]) -> Self::ResultFormat {
        let mut result_data = Vec::with_capacity(pixmap_width * pixmap_height * 3);

        for i in pixmap_data {
            let i: u32 = i.into();
            let color = i.to_le_bytes();
            result_data.push(color[0]);
            result_data.push(color[1]);
            result_data.push(color[2]);
        }

        base64::encode(&result_data)
    }

    fn decode(data: &Self::ResultFormat) -> Result<Vec<Color>> {
        let mut result = Vec::new();

        let mut color = [0u8; 3];
        for (i, i_value) in base64::decode(data)?.iter().enumerate() {
            if i % 3 == 0 {
                color[0] = *i_value;
            } else if i % 3 == 1 {
                color[1] = *i_value;
            } else if i % 3 == 2 {
                color[2] = *i_value;
                result.push(color.into());
            }
        }

        Ok(result)
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
            pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1 * 3
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
            let i  = (y * pixmap.get_size().unwrap().0 + x) * 3;
            let encoded_color = &encoded_bytes[i..i+3];
            let decoded_color = Color(encoded_color[0], encoded_color[1], encoded_color[2]);
            TestResult::from_bool(decoded_color == color)
        }
    }
}
