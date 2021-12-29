use super::{Entry, Iter};
use crate::Roaring;
use std::{collections::BTreeMap, mem};

/// Compressed bitmap for 64-bit integers.
///
/// Uses a set of 32-bit Roaring bitmaps, indexed by a 32-bit key through a
/// tree-based map (hence the name).
#[derive(Default)]
pub struct Bitmap {
    /// Underlying Roaring bitmaps, indexed by the 32 most significant bits of
    /// the integer.
    bitmaps: BTreeMap<u32, Roaring>,
}

impl Bitmap {
    /// Create an empty bitmap.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a value to the bitmap.
    ///
    /// If the bitmap did not have this value present, true is returned.
    /// If the bitmap did have this value present, false is returned.
    pub fn insert(&mut self, value: u64) -> bool {
        let entry = Entry::from(value);

        self.bitmaps
            .entry(entry.hi)
            .or_insert_with(Roaring::new)
            .insert(entry.lo)
    }

    /// Removes a value from the bitmap.
    ///
    /// Returns whether the value was present or not.
    pub fn remove(&mut self, value: u64) -> bool {
        let entry = Entry::from(value);

        match self.bitmaps.entry(entry.hi) {
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                let removed = slot.get_mut().remove(entry.lo);

                // Remove unused bitmap.
                if slot.get().is_empty() {
                    slot.remove();
                }
                removed
            },
            std::collections::btree_map::Entry::Vacant(_) => false,
        }
    }

    /// Returns true if the bitmap contains the value.
    pub fn contains(&self, value: u64) -> bool {
        let entry = Entry::from(value);

        self.bitmaps
            .get(&entry.hi)
            .map_or(false, |bitmap| bitmap.contains(entry.lo))
    }

    /// Computes the bitmap cardinality.
    pub fn cardinality(&self) -> usize {
        self.bitmaps
            .values()
            .fold(0, |acc, bitmap| acc + bitmap.cardinality())
    }

    /// Finds the smallest value in the bitmap.
    pub fn min(&self) -> Option<u64> {
        // TODO: use `first_key_value` when stable.
        self.bitmaps.iter().next().and_then(|(key, bitmap)| {
            bitmap.min().map(|min| Entry::from_parts(*key, min).into())
        })
    }

    /// Finds the largest value in the bitmap.
    pub fn max(&self) -> Option<u64> {
        // TODO: use `last_key_value` when stable.
        self.bitmaps.iter().last().and_then(|(key, bitmap)| {
            bitmap.max().map(|max| Entry::from_parts(*key, max).into())
        })
    }

    /// Clears the bitmap, removing all values.
    pub fn clear(&mut self) {
        self.bitmaps.clear();
    }

    /// Returns true if the bitmap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.bitmaps.is_empty()
    }

    /// Gets an iterator that visits the values in the bitmap in ascending
    /// order.
    pub(super) fn iter(&self) -> Iter<'_> {
        Iter::new(self.bitmaps.iter())
    }

    /// Returns the approximate in-memory size of the bitmap, in bytes.
    pub fn mem_size(&self) -> usize {
        mem::size_of_val(self)
            + self.bitmaps.iter().fold(0, |acc, (key, bitmap)| {
                acc + mem::size_of_val(key) + bitmap.mem_size()
            })
    }
}

impl Extend<u64> for Bitmap {
    fn extend<I: IntoIterator<Item = u64>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

impl FromIterator<u64> for Bitmap {
    fn from_iter<I: IntoIterator<Item = u64>>(iterator: I) -> Self {
        let mut bitmap = Self::new();
        bitmap.extend(iterator);
        bitmap
    }
}

impl<'a> IntoIterator for &'a Bitmap {
    type Item = u64;
    type IntoIter = Iter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_deletion() {
        let mut bitmap = Bitmap::new();
        assert_eq!(bitmap.cardinality(), 0);
        assert_eq!(bitmap.min(), None);
        assert_eq!(bitmap.max(), None);
        // No allocation for empty bitmap.
        assert_eq!(bitmap.bitmaps.len(), 0);

        // Bitmaps are created as needed.
        bitmap.insert(250070690272783730);
        bitmap.insert(250070690272783732);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.bitmaps.len(), 1);
        bitmap.insert(188740018811086);
        assert_eq!(bitmap.cardinality(), 3);
        assert_eq!(bitmap.bitmaps.len(), 2);

        // Operation works accross bitmaps.
        assert_eq!(bitmap.min(), Some(188740018811086));
        assert_eq!(bitmap.max(), Some(250070690272783732));

        // Bitmaps are deleted when empty.
        bitmap.remove(188740018811086);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.bitmaps.len(), 1);
    }

    #[test]
    fn contains() {
        let mut bitmap = Bitmap::new();
        assert_eq!(bitmap.contains(42), false);

        bitmap.insert(42);
        assert_eq!(bitmap.contains(42), true);

        bitmap.remove(42);
        assert_eq!(bitmap.contains(42), false);
    }

    #[test]
    fn already_exists() {
        let mut bitmap = Bitmap::new();

        assert_eq!(bitmap.insert(42), true, "new entry");
        assert_eq!(bitmap.insert(42), false, "already exists");
    }

    #[test]
    fn missing() {
        let mut bitmap = Bitmap::new();

        bitmap.insert(11);

        assert_eq!(bitmap.remove(11), true, "found");
        assert_eq!(bitmap.remove(11), false, "missing entry");
    }

    #[test]
    fn is_empty() {
        let mut bitmap = Bitmap::new();
        assert_eq!(bitmap.is_empty(), true);

        bitmap.insert(250070690292783730);
        bitmap.insert(250070690272783732);
        bitmap.insert(188740018811086);
        assert_eq!(bitmap.is_empty(), false);

        bitmap.clear();
        assert_eq!(bitmap.is_empty(), true);
    }

    #[test]
    fn iterator_sparse() {
        let input = (0..10_000).step_by(10).collect::<Vec<_>>();
        let bitmap = input.iter().copied().collect::<Bitmap>();
        let values = (&bitmap).into_iter().collect::<Vec<_>>();

        assert_eq!(values, input);
    }

    #[test]
    fn iterator_dense() {
        let input = (0..10_000).step_by(2).collect::<Vec<_>>();
        let bitmap = input.iter().copied().collect::<Bitmap>();
        let values = (&bitmap).into_iter().collect::<Vec<_>>();

        assert_eq!(values, input);
    }

    #[test]
    fn mem_size() {
        let bitmap = (0..10_000).step_by(2).collect::<Bitmap>();
        let bitmaps_size =
            bitmap.bitmaps.iter().fold(0, |acc, (key, bitmap)| {
                acc + mem::size_of_val(key) + bitmap.mem_size()
            });

        // Ensure we don't forget to account for the BTreeMap overhead.
        assert!(bitmap.mem_size() > bitmaps_size);
    }
}
