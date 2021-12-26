/// `Roaring` bitmap entry.
pub(crate) struct Entry {
    /// Most significant bits.
    pub(crate) hi: u16,
    /// Least significant bits.
    pub(crate) lo: u16,
}

impl Entry {
    /// Initialize a new entry from its lower and higher parts.
    pub(crate) fn from_parts(hi: u16, lo: u16) -> Self {
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
}
