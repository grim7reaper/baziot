use crate::chunk::{self, Chunk};

/// Compressed bitmap for 32-bit integers.
#[derive(Default)]
pub struct Roaring {
    /// Bitmap chunks, indexed by the 16 most significant bits of the integer.
    chunks: Vec<Chunk<Header>>,
}

impl Roaring {
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
}

/// Roaring bitmap entry.
pub(super) struct Entry {
    /// Most significant bits.
    pub(super) hi: u16,
    /// Least significant bits.
    pub(super) lo: u16,
}

impl Entry {
    pub(super) fn from_parts(hi: u16, lo: u16) -> Self {
        Self { hi, lo }
    }
}

impl From<u32> for Entry {
    #[allow(clippy::cast_possible_truncation)] // We truncate on purpose here.
    fn from(value: u32) -> Self {
        Self::from_parts((value >> 16) as u16, (value & 0xFFFF) as u16)
    }
}

impl From<Entry> for u32 {
    fn from(entry: Entry) -> Self {
        u32::from(entry.hi) << 16 | u32::from(entry.lo)
    }
}

/// Chunk header.
pub(super) struct Header {
    /// The 16 most significant bits.
    key: u16,
    /// Chunk's cardinality minus one.
    ///
    /// -1 allows to count up to 65536 while staying on 16-bit, and it's
    /// safe because the minimum size is 1 (empty chunks are deallocated).
    cardinality: u16,
}

impl Header {
    /// Initializes a new Chunk's header.
    pub(super) fn new(key: u16) -> Self {
        Self {
            key,
            cardinality: 0,
        }
    }
}

impl chunk::Header for Header {
    type Key = u16;

    fn key(&self) -> Self::Key {
        self.key
    }

    fn cardinality(&self) -> usize {
        usize::from(self.cardinality) + 1
    }

    fn increase_cardinality(&mut self) {
        self.cardinality += 1;
    }

    fn decrease_cardinality(&mut self) {
        self.cardinality = self.cardinality.saturating_sub(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry() {
        let value = 0x0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u32::from(entry), value);

        let value = 0x0000_0001;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000);
        assert_eq!(entry.lo, 0x0001);
        assert_eq!(u32::from(entry), value);

        let value = 0x0000_1000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000);
        assert_eq!(entry.lo, 0x1000);
        assert_eq!(u32::from(entry), value);

        let value = 0x0001_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0001);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u32::from(entry), value);

        let value = 0x1000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x1000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u32::from(entry), value);

        let value = 0xDEAD_BEEF;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0xDEAD);
        assert_eq!(entry.lo, 0xBEEF);
        assert_eq!(u32::from(entry), value);
    }

    #[test]
    fn insertion_deletion() {
        let mut bitmap = Roaring::new();
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
        let mut bitmap = Roaring::new();
        assert_eq!(bitmap.contains(42), false);

        bitmap.insert(42);
        assert_eq!(bitmap.contains(42), true);

        bitmap.remove(42);
        assert_eq!(bitmap.contains(42), false);
    }

    #[test]
    fn already_exists() {
        let mut bitmap = Roaring::new();

        assert_eq!(bitmap.insert(42), true, "new entry");
        assert_eq!(bitmap.insert(42), false, "already exists");
    }

    #[test]
    fn missing() {
        let mut bitmap = Roaring::new();

        bitmap.insert(11);

        assert_eq!(bitmap.remove(11), true, "found");
        assert_eq!(bitmap.remove(11), false, "missing entry");
    }
}
