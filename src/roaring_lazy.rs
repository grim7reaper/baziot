use crate::roaring_tree_map::Entry;
use crate::superchunk::SuperChunk;

/// Compressed bitmap for 64-bit integers, using a 2-level indexing.
///
/// The first level indexes chunks using the 32 most significant bits, then
/// each chunk indexes a container using the 16 most significant bits from the
/// lower half of the value.
#[derive(Default)]
pub struct RoaringLazy {
    /// Bitmap super chunks, indexed by the 32 most significant bits of the
    /// integer.
    chunks: Vec<SuperChunk>,
}

impl RoaringLazy {
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

        match self.chunks.binary_search_by_key(&entry.hi, SuperChunk::key) {
            Ok(index) => self.chunks[index].insert(entry.lo),
            Err(index) => {
                self.chunks.insert(index, SuperChunk::new(&entry));
                true
            },
        }
    }

    /// Removes a value from the bitmap.
    ///
    /// Returns whether the value was present or not.
    pub fn remove(&mut self, value: u64) -> bool {
        let entry = Entry::from(value);

        self.chunks
            .binary_search_by_key(&entry.hi, SuperChunk::key)
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
    pub fn contains(&self, value: u64) -> bool {
        let entry = Entry::from(value);

        self.chunks
            .binary_search_by_key(&entry.hi, SuperChunk::key)
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
    pub fn min(&self) -> Option<u64> {
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
    pub fn max(&self) -> Option<u64> {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_deletion() {
        let mut bitmap = RoaringLazy::new();
        assert_eq!(bitmap.cardinality(), 0);
        assert_eq!(bitmap.min(), None);
        assert_eq!(bitmap.max(), None);
        // No allocation for empty bitmap.
        assert_eq!(bitmap.chunks.len(), 0);

        // Chunks are created as needed.
        bitmap.insert(250070690292783730);
        bitmap.insert(250070690272783732);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.chunks.len(), 1);
        bitmap.insert(188740018811086);
        assert_eq!(bitmap.cardinality(), 3);
        assert_eq!(bitmap.chunks.len(), 2);

        // Operation works accross chunks.
        assert_eq!(bitmap.min(), Some(188740018811086));
        assert_eq!(bitmap.max(), Some(250070690292783730));

        // Chunks are deleted when empty.
        bitmap.remove(188740018811086);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.chunks.len(), 1);
    }

    #[test]
    fn contains() {
        let mut bitmap = RoaringLazy::new();
        assert_eq!(bitmap.contains(42), false);

        bitmap.insert(42);
        assert_eq!(bitmap.contains(42), true);

        bitmap.remove(42);
        assert_eq!(bitmap.contains(42), false);
    }

    #[test]
    fn already_exists() {
        let mut bitmap = RoaringLazy::new();

        assert_eq!(bitmap.insert(42), true, "new entry");
        assert_eq!(bitmap.insert(42), false, "already exists");
    }

    #[test]
    fn missing() {
        let mut bitmap = RoaringLazy::new();

        bitmap.insert(11);

        assert_eq!(bitmap.remove(11), true, "found");
        assert_eq!(bitmap.remove(11), false, "missing entry");
    }

    #[test]
    fn is_empty() {
        let mut bitmap = RoaringLazy::new();
        assert_eq!(bitmap.is_empty(), true);

        bitmap.insert(250070690292783730);
        bitmap.insert(250070690272783732);
        bitmap.insert(188740018811086);
        assert_eq!(bitmap.is_empty(), false);

        bitmap.clear();
        assert_eq!(bitmap.is_empty(), true);
    }
}
