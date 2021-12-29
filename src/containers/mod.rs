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

    /// Gets an iterator that visits the values in the container in ascending
    /// order.
    pub(crate) fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }

    /// Returns the approximate in-memory size of the container, in bytes.
    pub(crate) fn mem_size(&self) -> usize {
        match *self {
            Container::Array(ref array) => array.mem_size(),
            Container::Bitmap(ref bitmap) => bitmap.mem_size(),
        }
    }
}

pub(crate) enum Iter<'a> {
    /// Array container iterator.
    Array(array::Iter<'a>),
    /// Bitmap container iterator.
    Bitmap(bitmap::Iter<'a>),
}

impl<'a> Iter<'a> {
    fn new(container: &'a Container) -> Self {
        match *container {
            Container::Array(ref array) => Self::Array(array.iter()),
            Container::Bitmap(ref bitmap) => Self::Bitmap(bitmap.iter()),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        match *self {
            Self::Array(ref mut array) => array.next(),
            Self::Bitmap(ref mut bitmap) => bitmap.next(),
        }
    }
}
