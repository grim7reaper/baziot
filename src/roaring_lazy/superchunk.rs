use super::Entry;
use crate::{
    chunk::Chunk,
    roaring::{ChunkIter, Entry as ChunkEntry, Header},
};

pub(super) struct SuperChunk {
    key: u32,
    chunks: Vec<Chunk<Header>>,
}

impl SuperChunk {
    pub(super) fn new(entry: &Entry) -> Self {
        let chunk_entry = ChunkEntry::from(entry.lo);
        let header = Header::new(chunk_entry.hi);

        Self {
            key: entry.hi,
            chunks: vec![Chunk::new(header, chunk_entry.lo)],
        }
    }

    /// Adds a value to the chunk.
    ///
    /// If the chunk did not have this value present, true is returned.
    /// If the chunk did have this value present, false is returned.
    pub(super) fn insert(&mut self, value: u32) -> bool {
        let entry = ChunkEntry::from(value);

        match self.chunks.binary_search_by_key(&entry.hi, Chunk::key) {
            Ok(index) => self.chunks[index].insert(entry.lo),
            Err(index) => {
                let header = Header::new(entry.hi);
                self.chunks.insert(index, Chunk::new(header, entry.lo));
                true
            },
        }
    }

    /// Removes a value from the chunk.
    ///
    /// Returns whether the value was present or not.
    pub(super) fn remove(&mut self, value: u32) -> bool {
        let entry = ChunkEntry::from(value);

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

    /// Returns true if the chunk contains the value.
    pub(super) fn contains(&self, value: u32) -> bool {
        let entry = ChunkEntry::from(value);

        self.chunks
            .binary_search_by_key(&entry.hi, Chunk::key)
            .map(|index| self.chunks[index].contains(entry.lo))
            .unwrap_or(false)
    }

    /// Returns the chunk key.
    pub(super) fn key(&self) -> u32 {
        self.key
    }

    /// Computes the chunk cardinality.
    pub(super) fn cardinality(&self) -> usize {
        self.chunks
            .iter()
            .fold(0, |acc, chunk| acc + chunk.cardinality())
    }

    /// Finds the smallest value in the chunk.
    pub(super) fn min(&self) -> Option<u32> {
        self.chunks
            .iter()
            .filter_map(|chunk| {
                chunk
                    .min()
                    .map(|min| ChunkEntry::from_parts(chunk.key(), min).into())
            })
            .min()
    }

    /// Finds the largest value in the chunk.
    pub(super) fn max(&self) -> Option<u32> {
        self.chunks
            .iter()
            .filter_map(|chunk| {
                chunk
                    .max()
                    .map(|max| ChunkEntry::from_parts(chunk.key(), max).into())
            })
            .max()
    }

    /// Gets an iterator that visits the values in the superchunk in ascending
    /// order.
    pub(super) fn iter(&self) -> Iter<'_> {
        Iter::new(self)
    }
}

type ChunkFlatIter<'a> = std::iter::FlatMap<
    std::slice::Iter<'a, Chunk<Header>>,
    ChunkIter<'a>,
    fn(&'a Chunk<Header>) -> ChunkIter<'a>,
>;

/// Super-chunk iterator wrapper, containing the associated key as well.
pub(super) struct Iter<'a> {
    key: u32,
    inner: ChunkFlatIter<'a>,
}

impl<'a> Iter<'a> {
    fn new(chunk: &'a SuperChunk) -> Self {
        Self {
            key: chunk.key,
            inner: chunk.chunks.iter().flat_map(Into::into),
        }
    }
}

impl<'a> Iterator for Iter<'a> {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        self.inner
            .next()
            .map(|value| Entry::from_parts(self.key, value).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insertion_deletion() {
        let entry = 1538809352.into();
        let mut chunk = SuperChunk::new(&entry);
        assert_eq!(chunk.cardinality(), 1);
        assert_eq!(chunk.chunks.len(), 1);
        assert_eq!(chunk.min(), Some(1538809352));
        assert_eq!(chunk.max(), Some(1538809352));

        // Chunks are created as needed.
        chunk.insert(370099062);
        assert_eq!(chunk.cardinality(), 2);
        assert_eq!(chunk.chunks.len(), 2);

        // Operation works accross chunks.
        assert_eq!(chunk.min(), Some(370099062));
        assert_eq!(chunk.max(), Some(1538809352));

        // Chunks are deleted when empty.
        chunk.remove(370099062);
        assert_eq!(chunk.cardinality(), 1);
        assert_eq!(chunk.chunks.len(), 1);
    }

    #[test]
    fn contains() {
        let entry = 0.into();
        let mut chunk = SuperChunk::new(&entry);
        assert_eq!(chunk.contains(42), false);

        chunk.insert(42);
        assert_eq!(chunk.contains(42), true);

        chunk.remove(42);
        assert_eq!(chunk.contains(42), false);
    }

    #[test]
    fn already_exists() {
        let entry = 0.into();
        let mut chunk = SuperChunk::new(&entry);

        assert_eq!(chunk.insert(42), true, "new entry");
        assert_eq!(chunk.insert(42), false, "already exists");
    }

    #[test]
    fn missing() {
        let entry = 0.into();
        let mut chunk = SuperChunk::new(&entry);

        chunk.insert(11);

        assert_eq!(chunk.remove(11), true, "found");
        assert_eq!(chunk.remove(11), false, "missing entry");
    }
}
