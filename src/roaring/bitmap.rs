use super::{Entry, Header, Iter};
use crate::{Chunk, Container, Stats};
use std::mem;

/// Compressed bitmap for 32-bit integers.
#[derive(Default)]
pub struct Bitmap {
    /// Bitmap chunks, indexed by the 16 most significant bits of the integer.
    chunks: Vec<Chunk<Header>>,
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
    pub fn insert(&mut self, value: u32) -> bool {
        let entry = Entry::from(value);

        match self.chunks.binary_search_by_key(&entry.hi, Chunk::key) {
            Ok(index) => self.chunks[index].insert(entry.lo),
            Err(index) => {
                let header = Header::new(entry.hi);
                self.chunks.insert(index, Chunk::new(header, entry.lo));
                true
            },
        }
    }

    /// Removes a value from the bitmap.
    ///
    /// Returns whether the value was present or not.
    pub fn remove(&mut self, value: u32) -> bool {
        let entry = Entry::from(value);

        self.chunks
            .binary_search_by_key(&entry.hi, Chunk::key)
            .map(|index| {
                let old_cardinality = self.chunks[index].cardinality();
                let removed = self.chunks[index].remove(entry.lo);

                // Chunk is now empty (last element removed), delete it.
                if old_cardinality == 1 && removed {
                    self.chunks.remove(index);
                }
                removed
            })
            .unwrap_or(false)
    }

    /// Returns true if the bitmap contains the value.
    pub fn contains(&self, value: u32) -> bool {
        let entry = Entry::from(value);

        self.chunks
            .binary_search_by_key(&entry.hi, Chunk::key)
            .map(|index| self.chunks[index].contains(entry.lo))
            .unwrap_or(false)
    }

    /// Computes the bitmap cardinality.
    pub fn cardinality(&self) -> usize {
        self.chunks
            .iter()
            .fold(0, |acc, chunk| acc + chunk.cardinality())
    }

    /// Finds the smallest value in the bitmap.
    pub fn min(&self) -> Option<u32> {
        self.chunks
            .iter()
            .filter_map(|chunk| {
                chunk
                    .min()
                    .map(|min| Entry::from_parts(chunk.key(), min).into())
            })
            .min()
    }

    /// Finds the largest value in the bitmap.
    pub fn max(&self) -> Option<u32> {
        self.chunks
            .iter()
            .filter_map(|chunk| {
                chunk
                    .max()
                    .map(|max| Entry::from_parts(chunk.key(), max).into())
            })
            .max()
    }

    /// Clears the bitmap, removing all values.
    pub fn clear(&mut self) {
        self.chunks.clear();
    }

    /// Returns true if the bitmap contains no elements.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    /// Gets an iterator that visits the values in the bitmap in ascending
    /// order.
    pub fn iter(&self) -> Iter<'_> {
        Iter::new(self.chunks.iter())
    }

    /// Returns the approximate in-memory size of the bitmap, in bytes.
    pub fn mem_size(&self) -> usize {
        mem::size_of_val(self)
            + self
                .chunks
                .iter()
                .fold(0, |acc, chunk| acc + chunk.mem_size())
    }

    /// Returns detailed statistics about the composition of the bitmap.
    pub fn stats(&self) -> Stats<u32> {
        let mut stats = Stats {
            nb_containers: self.chunks.len(),
            nb_array_containers: 0,
            nb_bitmap_containers: 0,

            nb_values: self.cardinality(),
            nb_values_array_containers: 0,
            nb_values_bitmap_containers: 0,

            nb_bytes: self.mem_size(),
            nb_bytes_array_containers: 0,
            nb_bytes_bitmap_containers: 0,

            min_value: self.min(),
            max_value: self.max(),
        };

        for chunk in &self.chunks {
            match *chunk.container() {
                Container::Array(_) => {
                    stats.nb_array_containers += 1;
                    stats.nb_values_array_containers += chunk.cardinality();
                    stats.nb_bytes_array_containers += chunk.mem_size();
                },
                Container::Bitmap(_) => {
                    stats.nb_bitmap_containers += 1;
                    stats.nb_values_bitmap_containers += chunk.cardinality();
                    stats.nb_bytes_bitmap_containers += chunk.mem_size();
                },
            }
        }

        stats
    }
}

impl Extend<u32> for Bitmap {
    fn extend<I: IntoIterator<Item = u32>>(&mut self, iterator: I) {
        for value in iterator {
            self.insert(value);
        }
    }
}

impl FromIterator<u32> for Bitmap {
    fn from_iter<I: IntoIterator<Item = u32>>(iterator: I) -> Self {
        let mut bitmap = Self::new();
        bitmap.extend(iterator);
        bitmap
    }
}

impl<'a> IntoIterator for &'a Bitmap {
    type Item = u32;
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
        assert_eq!(bitmap.chunks.len(), 0);

        // Chunks are created as needed.
        bitmap.insert(1538809352);
        bitmap.insert(1538809350);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.chunks.len(), 1);
        bitmap.insert(370099062);
        assert_eq!(bitmap.cardinality(), 3);
        assert_eq!(bitmap.chunks.len(), 2);

        // Operation works accross chunks.
        assert_eq!(bitmap.min(), Some(370099062));
        assert_eq!(bitmap.max(), Some(1538809352));

        // Chunks are deleted when empty.
        bitmap.remove(370099062);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.chunks.len(), 1);
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

        bitmap.insert(1538809352);
        bitmap.insert(1538809350);
        bitmap.insert(370099062);
        assert_eq!(bitmap.is_empty(), false);

        bitmap.clear();
        assert_eq!(bitmap.is_empty(), true);
    }

    #[test]
    fn iterator_sparse() {
        let input = (0..10_000).step_by(10).collect::<Vec<_>>();
        let bitmap = input.iter().copied().collect::<Bitmap>();

        let stats = bitmap.stats();
        assert_eq!(stats.nb_bitmap_containers, 0, "sparse bitmap");

        let values = (&bitmap).into_iter().collect::<Vec<_>>();
        assert_eq!(values, input);
    }

    #[test]
    fn iterator_dense() {
        let input = (0..10_000).step_by(2).collect::<Vec<_>>();
        let bitmap = input.iter().copied().collect::<Bitmap>();

        let stats = bitmap.stats();
        assert_eq!(stats.nb_array_containers, 0, "dense bitmap");

        let values = (&bitmap).into_iter().collect::<Vec<_>>();
        assert_eq!(values, input);
    }

    #[test]
    fn mem_size() {
        let bitmap = (0..10_000).step_by(2).collect::<Bitmap>();
        let chunks_size = bitmap
            .chunks
            .iter()
            .fold(0, |acc, chunk| acc + chunk.mem_size());

        // Ensure we don't forget to account for the Vec overhead.
        assert!(bitmap.mem_size() > chunks_size);
    }
}
