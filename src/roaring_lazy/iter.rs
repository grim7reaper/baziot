use super::{superchunk, SuperChunk};

type SuperChunkFlatIter<'a> = std::iter::FlatMap<
    std::slice::Iter<'a, SuperChunk>,
    superchunk::Iter<'a>,
    fn(&'a SuperChunk) -> superchunk::Iter<'a>,
>;

/// Immutable Lazy Roaring bitmap iterator.
///
/// This struct is created by the `iter` method on Lazy Roaring bitmap.
pub struct Iter<'a> {
    inner: SuperChunkFlatIter<'a>,
    size: usize,
}

impl<'a> Iter<'a> {
    pub(super) fn new(chunks: std::slice::Iter<'a, SuperChunk>) -> Self {
        Self {
            inner: chunks.clone().flat_map(SuperChunk::iter),
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
