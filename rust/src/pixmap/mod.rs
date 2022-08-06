//!
//! Data structures to store pixel data, also called *Pixmaps*
//!

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

use anyhow::Result;
use thiserror::Error;

pub use color::*;
pub use file_backed_pixmap::FileBackedPixmap;
pub use in_memory_pixmap::InMemoryPixmap;
pub use remote_pixmap::RemotePixmap;
pub use replicating_pixmap::ReplicatingPixmap;

mod color;
mod file_backed_pixmap;
#[cfg(feature = "gui")]
pub mod gdk_pixbuf_pixmap;
mod in_memory_pixmap;
#[cfg(feature = "gui")]
pub mod paintable_pixmap;
pub mod pixmap_actor;
mod remote_pixmap;
mod replicating_pixmap;

/// A [`Pixmap`] which can be used throughout multiple threads
///
/// This is simply an [`Arc`] around any pixmap because pixmaps are already required to implement
/// interior mutability and thus are already [`Send`] and [`Sync`]. The Arc then allows actual
/// sharing between multiple users because it provides a [`Clone`] implementation that refers
/// to the same data.
pub type SharedPixmap<P> = Arc<P>;

#[derive(Debug, Error)]
enum GenericError {
    #[error("could not access coordinates {},{} on pixmap of size {}*{}", .target.0, .target.1, .size.0, .size.1)]
    InvalidCoordinates {
        target: (usize, usize),
        size: (usize, usize),
    },
    #[error("cannot create pixmap with invalid size {0}*{1}")]
    InvalidSize(usize, usize),
}

///
/// Generic trait for accessing pixel data in a unified way
///
pub trait Pixmap {
    /// Get the color value of the pixel at position (x,y)
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color>;

    /// Set the pixel value at position (x,y) to the specified color
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()>;

    /// Get the size of this pixmap as (width, height) tuple
    fn get_size(&self) -> Result<(usize, usize)>;

    /// Get all of the contained pixel data
    fn get_raw_data(&self) -> Result<Vec<Color>>;

    /// Overwrite all of the contained pixel data.
    ///
    /// If the given *data* is too small, the remaining pixmap colors will be kept as they are.
    ///
    /// If the given *data* is too large, left over data will simply be ignored.
    fn put_raw_data(&self, data: &Vec<Color>) -> Result<()>;
}

/// Calculates the index of the specified coordinates when pixels are stored in a Vector in
/// row-major order
fn pixel_coordinates_2_index(pixmap: &impl Pixmap, x: usize, y: usize) -> Result<usize> {
    Ok(y * pixmap.get_size()?.0 + x)
}

/// Calculate the coordinates `(x, y)` for the pixel with the given index assuming the pixels are
/// stored in row-major order
#[cfg(feature = "gui")]
fn pixel_index_2_coordinates(pixmap: &impl Pixmap, i: usize) -> Result<(usize, usize)> {
    let (width, _height) = pixmap.get_size()?;
    let x = i % width;
    let y = (i - x) / width;
    Ok((x, y))
}

/// Verify that the given coordinates are inside the given pixmap by returning an error if not
fn verify_coordinates_are_inside(pixmap: &impl Pixmap, x: usize, y: usize) -> Result<()> {
    let size = pixmap.get_size()?;

    // we don't need to check for >=0 because x and y are unsigned types
    if !(x < size.0 && y < size.1) {
        Err(GenericError::InvalidCoordinates {
            target: (x, y),
            size: pixmap.get_size()?,
        }
        .into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use super::*;

    pub(crate) fn test_set_and_get_pixel(
        pixmap: impl Pixmap,
        x: usize,
        y: usize,
        color: Color,
    ) -> TestResult {
        match pixmap.set_pixel(x, y, color) {
            Err(_) => TestResult::discard(),
            Ok(_) => quickcheck::TestResult::from_bool(pixmap.get_pixel(x, y).unwrap() == color),
        }
    }

    pub(crate) fn test_put_and_get_raw_data(pixmap: &impl Pixmap, color: Color) -> TestResult {
        // setup
        let data = vec![color; pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1];

        // execution
        pixmap.put_raw_data(&data).unwrap();
        let data_out = pixmap.get_raw_data().unwrap();

        // verification
        println!("{:?}", data);
        println!("{:?}", data_out);
        TestResult::from_bool(data == data_out)
    }

    pub(crate) fn test_put_raw_data_with_incorrect_size_data(pixmap: &impl Pixmap) {
        // setup
        let size = pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1;

        // empty data
        pixmap.set_pixel(0, 0, Color(42, 42, 42)).unwrap();
        pixmap.set_pixel(1, 0, Color(43, 43, 43)).unwrap();
        pixmap.put_raw_data(&Vec::new()).unwrap();
        let output_data = pixmap.get_raw_data().unwrap();
        assert_eq!(output_data[0], Color(42, 42, 42));
        assert_eq!(output_data[1], Color(43, 43, 43));
        assert_eq!(output_data[2..], vec![Color(0, 0, 0); size - 2]);

        // too small data
        let input_data = vec![Color(42, 42, 42); 10];
        pixmap.put_raw_data(&input_data).unwrap();
        let output_data = pixmap.get_raw_data().unwrap();
        assert_eq!(output_data[0..10], input_data);
        assert_eq!(output_data[10..], vec![Color(0, 0, 0); size - 10]);

        // too large data
        let input_data = vec![Color(42, 42, 42); size + 10];
        pixmap.put_raw_data(&input_data).unwrap();
        let output_data = pixmap.get_raw_data().unwrap();
        assert_eq!(output_data, vec![Color(42, 42, 42); size]);
    }
}
