/// `RoaringTwoLevels` bitmap entry.
pub(super) struct Entry {
    /// Most significant bits (48).
    pub(super) hi: u64,
    /// Least significant bits (16).
    pub(super) lo: u16,
}

impl Entry {
    /// Initialize a new entry from its lower and higher parts.
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
}
