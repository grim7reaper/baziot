use crate::containers::{self, Container};
use std::mem;

// Number of elements that defines the limit between a sparse and dense chunk.
const SPARSE_CHUNK_THRESHOLD: usize = 4_096;

/// A chunk header, providing key and cardinality handling.
pub(super) trait Header {
    type Key;

    /// Returns the chunk's key.
    fn key(&self) -> Self::Key;
    /// Returns the chunk's cardinality.
    fn cardinality(&self) -> usize;

    /// Increases by 1 the chunk's cardinality.
    fn increase_cardinality(&mut self);
    /// Decreases by 1 the chunk's cardinality.
    fn decrease_cardinality(&mut self);
}

/// Chunks of 2¹⁶ integers, using containers adapted to the density.
pub(super) struct Chunk<H> {
    /// Chunk header, holding the chunk's key and cardinality.
    header: H,
    /// The 16 least significant bits.
    container: Container,
}

pub(super) type Iter<'a> = containers::Iter<'a>;

impl<H: Header> Chunk<H> {
    /// Initializes a new chunk with the given value.
    pub(super) fn new(header: H, value: u16) -> Self {
        Self {
            header,
            container: Container::new(value),
        }
    }

    /// Adds a value to the chunk.
    ///
    /// If the chunk did not have this value present, true is returned.
    /// If the chunk did have this value present, false is returned.
    pub(super) fn insert(&mut self, value: u16) -> bool {
        let added = self.container.insert(value);
        if added {
            self.header.increase_cardinality();
            self.optimize_container();
        }
        added
    }

    /// Removes a value from the chunk.
    ///
    /// Returns whether the value was present or not.
    pub(super) fn remove(&mut self, value: u16) -> bool {
        let removed = self.container.remove(value);
        if removed {
            self.header.decrease_cardinality();
            self.optimize_container();
        }
        removed
    }

    /// Returns true if the chunk contains the value.
    pub(super) fn contains(&self, value: u16) -> bool {
        self.container.contains(value)
    }

    /// Returns the chunk key.
    pub(super) fn key(&self) -> H::Key {
        self.header.key()
    }

    /// Returns the chunk container.
    pub(super) fn container(&self) -> &Container {
        &self.container
    }

    /// Returns the chunk cardinality.
    pub(super) fn cardinality(&self) -> usize {
        self.header.cardinality()
    }

    /// Finds the smallest value in the chunk.
    pub(super) fn min(&self) -> Option<u16> {
        self.container.min()
    }

    /// Finds the largest value in the chunk.
    pub(super) fn max(&self) -> Option<u16> {
        self.container.max()
    }

    /// Gets an iterator that visits the values in the chunk in ascending
    /// order.
    pub(super) fn iter(&self) -> Iter<'_> {
        self.container.iter()
    }

    /// Returns the approximate in-memory size of the chunk, in bytes.
    pub(super) fn mem_size(&self) -> usize {
        mem::size_of_val(&self.header) + self.container.mem_size()
    }

    /// Ensures that the container is adapted to the chunk's cardinality.
    fn optimize_container(&mut self) {
        let better_container = match (&self.container, self.cardinality()) {
            (&Container::Array(ref array), cardinality)
                if cardinality > SPARSE_CHUNK_THRESHOLD =>
            {
                Some(Container::Bitmap(array.into()))
            },
            (&Container::Bitmap(ref bitmap), cardinality)
                if cardinality <= SPARSE_CHUNK_THRESHOLD =>
            {
                Some(Container::Array(bitmap.into()))
            },
            _ => None,
        };

        if let Some(container) = better_container {
            self.container = container;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::roaring::Header;

    #[test]
    fn insertion_deletion() {
        let header = Header::new(0);
        let mut chunk = Chunk::new(header, 0);

        // Chunks start with an array container.
        assert!(matches!(chunk.container, Container::Array(_)));
        assert_eq!(chunk.cardinality(), 1);

        // They keep using an array until they hit the density threshold.
        for value in 1..SPARSE_CHUNK_THRESHOLD {
            chunk.insert(value as u16);
            assert!(chunk.cardinality() <= SPARSE_CHUNK_THRESHOLD);
            assert!(matches!(chunk.container, Container::Array(_)));
        }

        // From there, they migrate the values into a bitmap container.
        chunk.insert(4242);
        chunk.insert(8888);
        assert!(chunk.cardinality() > SPARSE_CHUNK_THRESHOLD);
        assert!(matches!(chunk.container, Container::Bitmap(_)));

        // Original data (min) and new ones (max) are both here.
        assert_eq!(chunk.min(), Some(0));
        assert_eq!(chunk.max(), Some(8888));
        assert!(chunk.contains(4242));

        // Move values back into an array when the density is below the
        // threshold.
        chunk.remove(42);
        chunk.remove(1000);
        assert!(chunk.cardinality() <= SPARSE_CHUNK_THRESHOLD);
        assert!(matches!(chunk.container, Container::Array(_)));
    }

    #[test]
    fn contains() {
        let header = Header::new(0);
        let mut chunk = Chunk::new(header, 42);
        assert_eq!(chunk.contains(11), false);

        chunk.insert(11);
        assert_eq!(chunk.contains(11), true);

        chunk.remove(11);
        assert_eq!(chunk.contains(11), false);
    }

    #[test]
    fn already_exists() {
        let header = Header::new(0);
        let mut chunk = Chunk::new(header, 42);

        assert_eq!(chunk.insert(42), false, "already exists");
        assert_eq!(chunk.cardinality(), 1);

        assert_eq!(chunk.insert(11), true, "new entry");
        assert_eq!(chunk.cardinality(), 2);
    }

    #[test]
    fn missing() {
        let header = Header::new(0);
        let mut chunk = Chunk::new(header, 42);

        assert_eq!(chunk.remove(42), true, "found");
        assert_eq!(chunk.remove(11), false, "missing entry");
    }

    #[test]
    fn min_max() {
        let header = Header::new(0);
        let mut chunk = Chunk::new(header, 42);
        assert_eq!(chunk.min(), Some(42));
        assert_eq!(chunk.max(), Some(42));

        chunk.insert(11);
        chunk.insert(100);
        chunk.insert(77);
        chunk.insert(3);
        assert_eq!(chunk.min(), Some(3));
        assert_eq!(chunk.max(), Some(100));
    }

    #[test]
    fn mem_size() {
        let header = Header::new(0);
        let chunk = Chunk::new(header, 42);
        let container_size = chunk.container.mem_size();

        // Ensure we don't forget to account for the header overhead.
        assert!(chunk.mem_size() > container_size);
    }
}
