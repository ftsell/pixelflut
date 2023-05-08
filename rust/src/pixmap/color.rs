use anyhow::anyhow;
use std::fmt::{Formatter, UpperHex};

#[cfg(test)]
use quickcheck::{Arbitrary, Gen};

/// Color data represented as red, green, and blue channels each having a depth of 8 bits
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Color(pub u8, pub u8, pub u8);

// From RGB channels
impl From<[u8; 3]> for Color {
    fn from(data: [u8; 3]) -> Self {
        Self(data[0], data[1], data[2])
    }
}

// Into RGB channels
impl From<Color> for [u8; 3] {
    fn from(value: Color) -> Self {
        [value.0, value.1, value.2]
    }
}

// Into RGBA channels
impl From<Color> for [u8; 4] {
    fn from(value: Color) -> Self {
        [value.0, value.1, value.2, 0]
    }
}

// From RGBA channels as u32
impl From<u32> for Color {
    fn from(src: u32) -> Self {
        let b = src.to_be_bytes();
        Self(b[1], b[2], b[3])
    }
}

// Into RGBA channels as u32
impl From<Color> for u32 {
    fn from(value: Color) -> Self {
        u32::from_be_bytes([value.0, value.1, value.2, 0])
    }
}

impl Into<u32> for &Color {
    fn into(self) -> u32 {
        0u32 | (self.0 as u32) | (self.1 as u32) << 8 | (self.2 as u32) << 16
    }
}

impl TryFrom<&[u8]> for Color {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value.len() {
            3 => Ok(Self(value[0], value[1], value[2])),
            _ => Err(anyhow!(
                "cannot convert slices of more or less than three elements to color"
            )),
        }
    }
}

impl Into<Vec<u8>> for Color {
    fn into(self) -> Vec<u8> {
        vec![self.0, self.1, self.2]
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

impl UpperHex for Color {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        // format each byte as hex string with at least two characters and leading zeroes
        f.write_fmt(format_args!("{:02X}{:02X}{:02X}", self.0, self.1, self.2))
    }
}

#[cfg(test)]
impl Arbitrary for Color {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        u32::arbitrary(g).into()
    }
}

#[cfg(test)]
#[test]
fn test_u32_conversion() {
    assert_eq!(Color::from(0u32), Color(0, 0, 0));
    assert_eq!(Color::from(0xFFu32), Color(0, 0, 255));
    assert_eq!(Color::from(0x00FF00u32), Color(0, 255, 0));
    assert_eq!(Color::from(0xFF0000u32), Color(255, 0, 0));
}
