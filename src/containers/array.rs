use super::bitmap::Bitmap;
use std::{iter::FromIterator, mem};

/// A sorted array of packed 16-bit integers.
pub(crate) struct Array(Vec<u16>);

impl Array {
    /// Initializes a new array with the given value.
    pub(super) fn new(value: u16) -> Self {
        Self(vec![value])
    }

    /// Adds a value to the array.
    ///
    /// If the array did not have this value present, true is returned.
    /// If the array did have this value present, false is returned.
    pub(super) fn insert(&mut self, value: u16) -> bool {
        self.0
            .binary_search(&value)
            .map_err(|index| self.0.insert(index, value))
            .is_err()
    }

    /// Removes a value from the array.
    ///
    /// Returns whether the value was present or not.
    pub(super) fn remove(&mut self, value: u16) -> bool {
        self.0
            .binary_search(&value)
            .map(|index| self.0.remove(index))
            .is_ok()
    }

    /// Returns true if the array contains the value.
    pub(super) fn contains(&self, value: u16) -> bool {
        self.0.binary_search(&value).is_ok()
    }

    /// Finds the smallest value in the array.
    pub(super) fn min(&self) -> Option<u16> {
        self.0.first().copied()
    }

    /// Finds the largest value in the array.
    pub(super) fn max(&self) -> Option<u16> {
        self.0.last().copied()
    }

    /// Gets an iterator that visits the values in the array in ascending order.
    pub(super) fn iter(&self) -> Iter<'_> {
        Iter(self.0.iter().copied())
    }

    /// Returns the approximate in-memory size of the array, in bytes.
    pub(super) fn mem_size(&self) -> usize {
        mem::size_of_val(self) + self.0.len() * mem::size_of::<u16>()
    }

    #[cfg(test)]
    fn is_sorted(&self) -> bool {
        self.0.windows(2).all(|pair| pair[0] <= pair[1])
    }
}

impl FromIterator<u16> for Array {
    fn from_iter<I: IntoIterator<Item = u16>>(iter: I) -> Self {
        Self(Vec::from_iter(iter))
    }
}

impl From<&Bitmap> for Array {
    fn from(bitmap: &Bitmap) -> Self {
        bitmap.iter().by_ref().collect()
    }
}

pub(crate) struct Iter<'a>(std::iter::Copied<std::slice::Iter<'a, u16>>);

impl<'a> Iterator for Iter<'a> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserve_ordering() {
        let mut array = Array::new(42);
        assert!(array.is_sorted());

        array.insert(11);
        array.insert(77);
        array.insert(100);
        array.insert(3);
        assert!(array.is_sorted(), "insert preserve ordering");

        array.remove(100);
        array.remove(42);
        assert!(array.is_sorted(), "remove preserve ordering");
    }

    #[test]
    fn contains() {
        let mut array = Array::new(42);
        assert_eq!(array.contains(11), false);

        array.insert(11);
        assert_eq!(array.contains(11), true);

        array.remove(11);
        assert_eq!(array.contains(11), false);
    }

    #[test]
    fn already_exists() {
        let mut array = Array::new(42);

        assert_eq!(array.insert(42), false, "already exists");
        assert_eq!(array.insert(11), true, "new entry");
    }

    #[test]
    fn missing() {
        let mut array = Array::new(42);

        assert_eq!(array.remove(42), true, "found");
        assert_eq!(array.remove(11), false, "missing entry");
    }

    #[test]
    fn min_max() {
        let mut array = Array::new(42);
        assert_eq!(array.min(), Some(42));
        assert_eq!(array.max(), Some(42));

        array.insert(11);
        array.insert(100);
        array.insert(77);
        array.insert(3);
        assert_eq!(array.min(), Some(3));
        assert_eq!(array.max(), Some(100));
    }

    #[test]
    fn from_bitmap() {
        let mut bitmap = Bitmap::new();
        bitmap.insert(11);
        bitmap.insert(100);
        bitmap.insert(77);
        bitmap.insert(3);

        let array = Array::from(&bitmap);
        assert_eq!(array.iter().collect::<Vec<_>>(), vec![3u16, 11, 77, 100]);
    }

    #[test]
    fn mem_size() {
        let mut array = Array::new(42);
        let size = array.mem_size();

        array.insert(11);
        array.insert(100);

        // Size grows as we insert values.
        assert!(size <= array.mem_size());
    }
}
