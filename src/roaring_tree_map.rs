use crate::Roaring;
use std::collections::BTreeMap;

/// Compressed bitmap for 64-bit integers.
///
/// Uses a set of 32-bit Roaring bitmaps, indexed by a 32-bit key through a
/// tree-based map (hence the name).
#[derive(Default)]
pub struct RoaringTreeMap {
    /// Underlying Roaring bitmaps, indexed by the 32 most significant bits of
    /// the integer.
    bitmaps: BTreeMap<u32, Roaring>,
}

impl RoaringTreeMap {
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
                if removed && slot.get().cardinality() == 0 {
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
}

/// `RoaringTreeMap` entry.
pub(super) struct Entry {
    /// Most significant bits.
    pub(super) hi: u32,
    /// Least significant bits.
    pub(super) lo: u32,
}

impl Entry {
    pub(super) fn from_parts(hi: u32, lo: u32) -> Self {
        Self { hi, lo }
    }
}

impl From<u64> for Entry {
    #[allow(clippy::cast_possible_truncation)] // We truncate on purpose here.
    fn from(value: u64) -> Self {
        Self::from_parts((value >> 32) as u32, (value & 0xFFFF_FFFF) as u32)
    }
}

impl From<Entry> for u64 {
    fn from(entry: Entry) -> Self {
        u64::from(entry.hi) << 32 | u64::from(entry.lo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry() {
        let value = 0x0000_0000_0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000);
        assert_eq!(entry.lo, 0x0000);
        assert_eq!(u64::from(entry), value);

        let value = 0x0000_0000_0000_0001;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000_0000);
        assert_eq!(entry.lo, 0x0000_0001);
        assert_eq!(u64::from(entry), value);

        let value = 0x0000_0000_1000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000_0000);
        assert_eq!(entry.lo, 0x1000_0000);
        assert_eq!(u64::from(entry), value);

        let value = 0x0000_0001_0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x0000_0001);
        assert_eq!(entry.lo, 0x0000_0000);
        assert_eq!(u64::from(entry), value);

        let value = 0x1000_0000_0000_0000;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0x1000_0000);
        assert_eq!(entry.lo, 0x0000_0000);
        assert_eq!(u64::from(entry), value);

        let value = 0xFEED_FACE_CAFE_BEEF;
        let entry = Entry::from(value);
        assert_eq!(entry.hi, 0xFEED_FACE);
        assert_eq!(entry.lo, 0xCAFE_BEEF);
        assert_eq!(u64::from(entry), value);
    }

    #[test]
    fn insertion_deletion() {
        let mut bitmap = RoaringTreeMap::new();
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
        let mut bitmap = RoaringTreeMap::new();
        assert_eq!(bitmap.contains(42), false);

        bitmap.insert(42);
        assert_eq!(bitmap.contains(42), true);
    }

    #[test]
    fn already_exists() {
        let mut bitmap = RoaringTreeMap::new();

        assert_eq!(bitmap.insert(42), true, "new entry");
        assert_eq!(bitmap.insert(42), false, "already exists");
    }

    #[test]
    fn missing() {
        let mut bitmap = RoaringTreeMap::new();

        bitmap.insert(11);

        assert_eq!(bitmap.remove(11), true, "found");
        assert_eq!(bitmap.remove(11), false, "missing entry");
    }
}
