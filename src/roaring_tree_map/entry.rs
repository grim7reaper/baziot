/// `RoaringTreeMap` entry.
pub(crate) struct Entry {
    /// Most significant bits.
    pub(crate) hi: u32,
    /// Least significant bits.
    pub(crate) lo: u32,
}

impl Entry {
    /// Initialize a new entry from its lower and higher parts.
    pub(crate) fn from_parts(hi: u32, lo: u32) -> Self {
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
}
