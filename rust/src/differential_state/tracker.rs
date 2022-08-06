//! The main [`Tracker`] type and its implementation

use nohash_hasher::BuildNoHashHasher;
use pixelflut::pixmap::Color;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

/// A type that keeps track of the most recent pixmap update of each pixel
pub struct Tracker {
    /// The size of the pixmap in which this change occurred
    pixmap_size: (usize, usize),

    /// The changes that have occurred since they were last retrieved
    changes: HashSet<TrackedChange, BuildNoHashHasher<TrackedChange>>,
}

impl Tracker {
    /// Create a new Tracker that can tracker changes for a pixmap of the given size
    pub fn new(pixmap_width: usize, pixmap_height: usize) -> Self {
        Self {
            pixmap_size: (pixmap_width, pixmap_height),
            changes: HashSet::with_capacity_and_hasher(
                pixmap_width * pixmap_height,
                BuildNoHashHasher::default(),
            ),
        }
    }

    /// Add a new change that should be tracked
    pub fn add(&mut self, x: usize, y: usize, color: Color) {
        self.changes.insert(TrackedChange {
            pixmap_index: y * self.pixmap_size.0 + x,
            coordinates: (x, y),
            color,
        });
    }

    /// Retrieve an iterator over the list of changes
    ///
    /// Also resets the tracker so that subsequent calls to `get_changes()` with no other action in between
    /// will then return an empty iterator.
    pub fn get_changes(&mut self) -> impl Iterator<Item = TrackedChange> + ExactSizeIterator + '_ {
        self.changes.drain()
    }

    /// Clear the internal list of all tracked changes
    pub fn clear(&mut self) {
        self.changes.clear();
    }
}

/// A change to a certain pixel inside a certain pixmap that can be tracked using a [`Tracker`]
#[derive(Debug)]
pub struct TrackedChange {
    /// The index of the changed pixel inside the pixmap
    pixmap_index: usize,

    /// (x, y) coordinates of this change
    pub coordinates: (usize, usize),

    /// The color to which the pixel was changed
    pub color: Color,
}

impl nohash_hasher::IsEnabled for TrackedChange {}

impl Hash for TrackedChange {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.pixmap_index)
    }
}

impl PartialEq<Self> for TrackedChange {
    fn eq(&self, other: &Self) -> bool {
        self.pixmap_index == other.pixmap_index
    }
}

impl Eq for TrackedChange {}

#[cfg(test)]
#[test]
pub fn test_add_and_get_changes() {
    let mut tracker = Tracker::new(10, 10);

    // check that an added item that was added is in the changes
    tracker.add(5, 5, Color(42, 42, 42));
    assert_eq!(
        tracker.get_changes().collect::<Vec<TrackedChange>>(),
        vec![TrackedChange {
            pixmap_index: 55,
            coordinates: (5, 5),
            color: Color(42, 42, 42)
        }]
    );

    // check that the changes have been reset
    assert_eq!(tracker.get_changes().collect::<Vec<TrackedChange>>(), vec![]);

    // check that only the most recent item is kept
    tracker.add(5, 5, Color(42, 42, 42));
    tracker.add(5, 5, Color(120, 120, 120));
    assert_eq!(
        tracker.get_changes().collect::<Vec<TrackedChange>>(),
        vec![TrackedChange {
            pixmap_index: 55,
            coordinates: (5, 5),
            color: Color(120, 120, 120)
        }]
    );

    // check that there are no hash collisions
    for x in 0..10 {
        for y in 0..10 {
            tracker.add(x, y, Color(x as u8, y as u8, 0));
        }
    }
    assert_eq!(tracker.get_changes().len(), 100);
}
