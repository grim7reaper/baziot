use super::array::Array;
use std::iter::FromIterator;

/// Bitmap size, in 64-bit words.
const BITMAP_WORD_COUNT: usize = 1024;

/// 2¹⁶-bit bitmap.
pub(crate) struct Bitmap(Box<[u64; BITMAP_WORD_COUNT]>);

impl Bitmap {
    /// Initializes a new empty bitmap.
    pub(super) fn new() -> Self {
        Self(Box::new([0; BITMAP_WORD_COUNT]))
    }

    /// Adds a value to the bitmap.
    ///
    /// If the bitmap did not have this value present, true is returned.
    /// If the bitmap did have this value present, false is returned.
    pub(crate) fn insert(&mut self, value: u16) -> bool {
        let index = value.into();
        let exists = self.tst(&index);

        self.set(&index);

        !exists
    }

    /// Removes a value from the bitmap.
    ///
    /// Returns whether the value was present or not.
    pub(crate) fn remove(&mut self, value: u16) -> bool {
        let index = value.into();
        let exists = self.tst(&index);

        self.clr(&index);

        exists
    }

    /// Returns true if the bitmap contains the value.
    pub(crate) fn contains(&self, value: u16) -> bool {
        self.tst(&value.into())
    }

    /// Finds the smallest value in the bitmap.
    // Max index is BITMAP_WORD_COUNT/max trailing zeros is 64: no truncation.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn min(&self) -> Option<u16> {
        self.0.iter().enumerate().find(|&(_, word)| *word != 0).map(
            |(index, bit)| {
                let tail = (index as u16) * 64;
                let head = bit.trailing_zeros() as u16;

                tail + head
            },
        )
    }

    /// Finds the largest value in the bitmap.
    // Max index is BITMAP_WORD_COUNT/max leading zeros is 64: no truncation.
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn max(&self) -> Option<u16> {
        self.0
            .iter()
            .enumerate()
            .rev()
            .find(|&(_, word)| *word != 0)
            .map(|(index, bit)| {
                let tail = (index as u16) * 64;
                let head = 64 - 1 - (bit.leading_zeros() as u16);

                tail + head
            })
    }

    /// Returns an iterator over the bitmap values.
    pub(super) fn iter(&self) -> impl Iterator<Item = u16> + '_ {
        Iter::new(&self.0)
    }

    /// Tests the bit at `index`.
    fn tst(&self, index: &Index) -> bool {
        (self.0[index.word] >> index.bit) & 1 != 0
    }

    /// Sets the bit at `index`.
    fn set(&mut self, index: &Index) {
        self.0[index.word] |= 1 << index.bit;
    }

    /// Clears the bit at `index`.
    fn clr(&mut self, index: &Index) {
        self.0[index.word] &= !(1 << index.bit);
    }
}

impl FromIterator<u16> for Bitmap {
    fn from_iter<I: IntoIterator<Item = u16>>(iter: I) -> Self {
        let mut bitmap = Self::new();

        for value in iter {
            bitmap.set(&value.into());
        }

        bitmap
    }
}

impl From<&Array> for Bitmap {
    fn from(array: &Array) -> Self {
        array.iter().copied().collect()
    }
}

/// Bitmap index
struct Index {
    /// Selected word in the bitmap.
    word: usize,
    /// Selected bit in the word.
    bit: u16,
}

impl From<u16> for Index {
    fn from(value: u16) -> Self {
        Self {
            word: usize::from(value / 64),
            bit: value % 64,
        }
    }
}

struct Iter<'a> {
    bitmap: &'a [u64; BITMAP_WORD_COUNT],
    index: usize,
    word: u64,
}

impl<'a> Iter<'a> {
    fn new(bitmap: &'a [u64; BITMAP_WORD_COUNT]) -> Self {
        Self {
            bitmap,
            index: 0,
            word: bitmap[0],
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u16;

    // Max index is BITMAP_WORD_COUNT/max trailing zeros is 64: no truncation.
    #[allow(clippy::cast_possible_truncation)]
    fn next(&mut self) -> Option<u16> {
        while self.word == 0 {
            self.index += 1;
            if self.index == self.bitmap.len() {
                return None;
            }
            self.word = self.bitmap[self.index];
        }
        let value = (self.index as u32) * 64 + self.word.trailing_zeros();
        self.word &= self.word - 1;

        Some(value as u16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexing() {
        // Min index.
        let index = Index::from(0);
        assert_eq!(index.word, 0);
        assert_eq!(index.bit, 0);
        // Last bit of a word.
        let index = Index::from(63);
        assert_eq!(index.word, 0);
        assert_eq!(index.bit, 63);
        // First bit of a word.
        let index = Index::from(64);
        assert_eq!(index.word, 1);
        assert_eq!(index.bit, 0);
        // In the middle of a word.
        let index = Index::from(72);
        assert_eq!(index.word, 1);
        assert_eq!(index.bit, 8);
        // Max index.
        let index = Index::from(u16::MAX);
        assert_eq!(index.word, 1023);
        assert_eq!(index.bit, 63);
    }

    #[test]
    fn bit_twiddling() {
        let mut bitmap = Bitmap::new();

        for value in &[35470, 18777, 7, 12189, 45566] {
            let index = Index::from(*value);

            assert!(!bitmap.tst(&index), "default to unset");

            bitmap.set(&index);
            assert!(bitmap.tst(&index), "set a bit");

            bitmap.clr(&index);
            assert!(!bitmap.tst(&index), "unset a bit");
        }
    }

    #[test]
    fn min_max() {
        let mut bitmap = Bitmap::new();
        assert_eq!(bitmap.min(), None);
        assert_eq!(bitmap.max(), None);

        bitmap.insert(11);
        assert_eq!(bitmap.min(), Some(11));
        assert_eq!(bitmap.max(), Some(11));

        bitmap.insert(100);
        bitmap.insert(77);
        bitmap.insert(3);
        assert_eq!(bitmap.min(), Some(3));
        assert_eq!(bitmap.max(), Some(100));
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
    fn from_array() {
        let mut array = Array::new(11);
        array.insert(100);
        array.insert(77);
        array.insert(3);

        let bitmap = Bitmap::from(&array);
        assert_eq!(bitmap.iter().collect::<Vec<_>>(), vec![3u16, 11, 77, 100]);
    }
}
