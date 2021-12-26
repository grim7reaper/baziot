use super::{Entry, Header};
use crate::{chunk, Chunk};

/// Immutable Roaring Two-Levels bitmap iterator.
///
/// This struct is created by the `iter` method on Roaring Two-Levels bitmap.
pub struct Iter<'a> {
    inner: std::slice::Iter<'a, Chunk<Header>>,
    iter: Option<ChunkIter<'a>>,
    size: usize,
}

impl<'a> Iter<'a> {
    pub(super) fn new(mut chunks: std::slice::Iter<'a, Chunk<Header>>) -> Self {
        let size = chunks
            .clone()
            .fold(0, |acc, chunk| acc + chunk.cardinality());
        let iter = chunks.next().map(Into::into);

        Self {
            inner: chunks,
            iter,
            size,
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.size = self.size.saturating_sub(1);
        let iter = self.iter.as_mut()?;

        iter.next().or_else(|| {
            self.iter = self.inner.next().map(Into::into);
            self.iter.as_mut().and_then(std::iter::Iterator::next)
        })
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
