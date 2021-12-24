use crate::chunk::{self, Chunk};

/// Compressed bitmap for 64-bit integers, using 48-bit prefix key.
#[derive(Default)]
pub struct RoaringTwoLevels {
    /// Bitmap chunks, indexed by the 48 most significant bits of the integer.
    chunks: Vec<Chunk<Header>>,
}

impl RoaringTwoLevels {
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
    pub fn remove(&mut self, value: u64) -> bool {
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
    pub fn contains(&self, value: u64) -> bool {
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

/// `RoaringTwoLevels` bitmap entry.
pub(super) struct Entry {
    /// Most significant bits (48).
    pub(super) hi: u64,
    /// Least significant bits (16).
    pub(super) lo: u16,
}

impl Entry {
    pub(super) fn from_parts(hi: u64, lo: u16) -> Self {
        Self { hi, lo }
    }
}

impl From<u64> for Entry {
    #[allow(clippy::cast_possible_truncation)] // We truncate on purpose here.
    fn from(value: u64) -> Self {
        Self::from_parts((value >> 16) as u64, (value & 0xFFFF) as u16)
    }
}

impl From<Entry> for u64 {
    fn from(entry: Entry) -> Self {
        entry.hi << 16 | u64::from(entry.lo)
    }
}

/// Chunk header.
pub(super) struct Header {
    /// Header's data.
    ///
    /// Contains both the chunk's key (in the upper 48 bits) and the chunk's
    /// cardinality minus one (in the lower 16 bits) packed into a single
    /// 64-bit integer.
    ///
    /// Storing `cardinality - 1` allows to count up to 65536 while staying on
    /// 16-bit (that way it fits alongside the key), and it's safe because the
    /// minimum size is 1 (empty chunks are deallocated).
    data: u64,
}

impl Header {
    /// Initializes a new Chunk's header.
    pub(super) fn new(key: u64) -> Self {
        Self { data: key << 16 }
    }

    /// Extracts the cardinality from the packed data field.
    #[allow(clippy::cast_possible_truncation)] // We truncate on purpose here.
    fn unpack_cardinality(&self) -> u16 {
        (self.data & 0xFFFF) as u16
    }

    /// Packs a new cardinality value into the packed data field.
    fn pack_cardinality(&mut self, cardinality: u16) {
        const CARDINALITY_MASK: u64 = 0xFFFF_FFFF_FFFF_0000;
        self.data = (self.data & CARDINALITY_MASK) | u64::from(cardinality);
    }
}

impl chunk::Header for Header {
    type Key = u64;

    fn key(&self) -> Self::Key {
        self.data >> 16
    }

    fn cardinality(&self) -> usize {
        usize::from(self.unpack_cardinality()) + 1
    }

    fn increase_cardinality(&mut self) {
        let cardinality = self.unpack_cardinality() + 1;
        self.pack_cardinality(cardinality);
    }

    fn decrease_cardinality(&mut self) {
        let cardinality = self.unpack_cardinality().saturating_sub(1);
        self.pack_cardinality(cardinality);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Header as HeaderTrait;

    #[test]
    fn entry() {
        let value = 0x0000_0000_0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u64::from(entry), value);

        let value = 0x0000_0000_0000_0001;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000);
        assert_eq!(entry.lo, 0x0001);
        assert_eq!(u64::from(entry), value);

        let value = 0x0000_0000_1000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000_0000_1000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u64::from(entry), value);

        let value = 0x0000_0001_0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000_0001_0000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u64::from(entry), value);

        let value = 0x1000_0000_0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x1000_0000_0000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u64::from(entry), value);

        let value = 0xFEED_FACE_CAFE_BEEF;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0xFEED_FACE_CAFE);
        assert_eq!(entry.lo, 0xBEEF);
        assert_eq!(u64::from(entry), value);
    }

    #[test]
    fn header() {
        let mut header = Header::new(0xFEED_DEAD_BEEF);
        assert_eq!(header.data, 0xFEED_DEAD_BEEF_0000);
        assert_eq!(header.key(), 0xFEED_DEAD_BEEF);
        assert_eq!(header.unpack_cardinality(), 0);

        header.increase_cardinality();
        assert_eq!(header.key(), 0xFEED_DEAD_BEEF);
        assert_eq!(header.unpack_cardinality(), 1);

        header.decrease_cardinality();
        assert_eq!(header.key(), 0xFEED_DEAD_BEEF);
        assert_eq!(header.unpack_cardinality(), 0);
    }

    #[test]
    fn insertion_deletion() {
        let mut bitmap = RoaringTwoLevels::new();
        assert_eq!(bitmap.cardinality(), 0);
        assert_eq!(bitmap.min(), None);
        assert_eq!(bitmap.max(), None);
        // No allocation for empty bitmap.
        assert_eq!(bitmap.chunks.len(), 0);

        // Chunks are created as needed.
        bitmap.insert(250070690272783730);
        bitmap.insert(250070690272783732);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.chunks.len(), 1);
        bitmap.insert(188740018811086);
        assert_eq!(bitmap.cardinality(), 3);
        assert_eq!(bitmap.chunks.len(), 2);

        // Operation works accross chunks.
        assert_eq!(bitmap.min(), Some(188740018811086));
        assert_eq!(bitmap.max(), Some(250070690272783732));

        // Chunks are deleted when empty.
        bitmap.remove(188740018811086);
        assert_eq!(bitmap.cardinality(), 2);
        assert_eq!(bitmap.chunks.len(), 1);
    }

    #[test]
    fn contains() {
        let mut bitmap = RoaringTwoLevels::new();
        assert_eq!(bitmap.contains(42), false);

        bitmap.insert(42);
        assert_eq!(bitmap.contains(42), true);

        bitmap.remove(42);
        assert_eq!(bitmap.contains(42), false);
    }

    #[test]
    fn already_exists() {
        let mut bitmap = RoaringTwoLevels::new();

        assert_eq!(bitmap.insert(42), true, "new entry");
        assert_eq!(bitmap.insert(42), false, "already exists");
    }

    #[test]
    fn missing() {
        let mut bitmap = RoaringTwoLevels::new();

        bitmap.insert(11);

        assert_eq!(bitmap.remove(11), true, "found");
        assert_eq!(bitmap.remove(11), false, "missing entry");
    }

    #[test]
    fn is_empty() {
        let mut bitmap = RoaringTwoLevels::new();
        assert_eq!(bitmap.is_empty(), true);

        bitmap.insert(250070690292783730);
        bitmap.insert(250070690272783732);
        bitmap.insert(188740018811086);
        assert_eq!(bitmap.is_empty(), false);

        bitmap.clear();
        assert_eq!(bitmap.is_empty(), true);
    }
}
