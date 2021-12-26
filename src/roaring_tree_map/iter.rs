use super::Entry;
use crate::{roaring, Roaring};
use std::collections::btree_map;

type RoaringFlatIter<'a> = std::iter::FlatMap<
    btree_map::Iter<'a, u32, Roaring>,
    BitmapIter<'a>,
    fn((&'a u32, &'a Roaring)) -> BitmapIter<'a>,
>;

/// Immutable Roaring Tree-Map bitmap iterator.
///
/// This struct is created by the `iter` method on Roaring Tree-Map bitmap.
pub struct Iter<'a> {
    inner: RoaringFlatIter<'a>,
    size: usize,
}

impl<'a> Iter<'a> {
    pub(super) fn new(bitmaps: btree_map::Iter<'a, u32, Roaring>) -> Self {
        Self {
            inner: bitmaps.clone().flat_map(Into::into),
            size: bitmaps.fold(0, |acc, bitmap| acc + bitmap.1.cardinality()),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.size = self.size.saturating_sub(1);
        self.inner.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.size, Some(self.size))
    }
}

/// Roaring bitmap iterator wrapper, containing the associated key as well.
struct BitmapIter<'a> {
    key: u32,
    inner: roaring::Iter<'a>,
}

impl<'a> From<(&'a u32, &'a Roaring)> for BitmapIter<'a> {
    fn from(entry: (&'a u32, &'a Roaring)) -> Self {
        Self {
            key: *entry.0,
            inner: entry.1.iter(),
        }
    }
}

impl<'a> Iterator for BitmapIter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.inner
            .next()
            .map(|value| Entry::from_parts(self.key, value).into())
    }
}
