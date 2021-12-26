use super::{Entry, Header};
use crate::{chunk, Chunk};

type ChunkFlatIter<'a> = std::iter::FlatMap<
    std::slice::Iter<'a, Chunk<Header>>,
    ChunkIter<'a>,
    fn(&'a Chunk<Header>) -> ChunkIter<'a>,
>;

/// Immutable Roaring Two-Levels bitmap iterator.
///
/// This struct is created by the `iter` method on Roaring Two-Levels bitmap.
pub struct Iter<'a> {
    inner: ChunkFlatIter<'a>,
    size: usize,
}

impl<'a> Iter<'a> {
    pub(super) fn new(chunks: std::slice::Iter<'a, Chunk<Header>>) -> Self {
        Self {
            inner: chunks.clone().flat_map(Into::into),
            size: chunks.fold(0, |acc, chunk| acc + chunk.cardinality()),
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

/// Chunk iterator wrapper, containing the associated key as well.
struct ChunkIter<'a> {
    key: u64,
    inner: chunk::Iter<'a>,
}

impl<'a> From<&'a Chunk<Header>> for ChunkIter<'a> {
    fn from(chunk: &'a Chunk<Header>) -> Self {
        Self {
            key: chunk.key(),
            inner: chunk.iter(),
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.inner
            .next()
            .map(|value| Entry::from_parts(self.key, value).into())
    }
}
