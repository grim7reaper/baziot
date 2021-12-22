mod array;
mod bitmap;

use array::Array;
use bitmap::Bitmap;

/// Integers container for chunks, bounded to 8 kB at most.
pub(crate) enum Container {
    /// Array container for sparse chunks.
    Array(Array),
    /// Bitmap container for dense chunks.
    Bitmap(Bitmap),
}

impl Container {
    /// Initializes a new container with the given value.
    pub(crate) fn new(value: u16) -> Self {
        Container::Array(Array::new(value))
    }

    /// Adds a value to the container.
    ///
    /// If the container did not have this value present, true is returned.
    /// If the container did have this value present, false is returned.
    pub(crate) fn insert(&mut self, value: u16) -> bool {
        match *self {
            Container::Array(ref mut array) => array.insert(value),
            Container::Bitmap(ref mut bitmap) => bitmap.insert(value),
        }
    }

    /// Removes a value from the container.
    ///
    /// Returns whether the value was present or not.
    pub(crate) fn remove(&mut self, value: u16) -> bool {
        match *self {
            Container::Array(ref mut array) => array.remove(value),
            Container::Bitmap(ref mut bitmap) => bitmap.remove(value),
        }
    }

    /// Returns true if the container contains the value.
    pub(crate) fn contains(&self, value: u16) -> bool {
        match *self {
            Container::Array(ref array) => array.contains(value),
            Container::Bitmap(ref bitmap) => bitmap.contains(value),
        }
    }

    /// Finds the smallest value in the container.
    pub(crate) fn min(&self) -> Option<u16> {
        match *self {
            Container::Array(ref array) => array.min(),
            Container::Bitmap(ref bitmap) => bitmap.min(),
        }
    }

    /// Finds the largest value in the container.
    pub(crate) fn max(&self) -> Option<u16> {
        match *self {
            Container::Array(ref array) => array.max(),
            Container::Bitmap(ref bitmap) => bitmap.max(),
        }
    }
}
