use crate::{containers::Container, roaring::Entry};

// Number of elements that defines the limit between a sparse and dense chunk.
const SPARSE_CHUNK_THRESHOLD: usize = 4_096;

/// Chunks of 2^16 integers, using containers adapted to the density.
pub(crate) struct Chunk {
    /// The 16 most significant bits.
    key: u16,
    /// Chunk's cardinality minus one.
    ///
    /// -1 allows to count up to 65536 while staying on 16-bit, and it's safe
    /// because the minimum size is 1 (empty chunks are deallocated).
    cardinality: u16,
    /// The 16 least significant bits.
    container: Container,
}

impl Chunk {
    /// Initializes a new chunk with the given value.
    pub(crate) fn new(entry: &Entry) -> Self {
        Self {
            key: entry.hi,
            cardinality: 0,
            container: Container::new(entry.lo),
        }
    }

    /// Adds a value to the chunk.
    ///
    /// If the chunk did not have this value present, true is returned.
    /// If the chunk did have this value present, false is returned.
    pub(crate) fn insert(&mut self, value: u16) -> bool {
        let added = self.container.insert(value);
        if added {
            self.cardinality += 1;
            self.optimize_container();
        }
        added
    }

    /// Removes a value from the chunk.
    ///
    /// Returns whether the value was present or not.
    pub(crate) fn remove(&mut self, value: u16) -> bool {
        let removed = self.container.remove(value);
        if removed {
            self.cardinality = self.cardinality.saturating_sub(1);
            self.optimize_container();
        }
        removed
    }

    /// Returns true if the chunk contains the value.
    pub(crate) fn contains(&self, value: u16) -> bool {
        self.container.contains(value)
    }

    /// Returns the chunk key.
    pub(crate) fn key(&self) -> u16 {
        self.key
    }

    /// Returns the chunk cardinality.
    pub(crate) fn cardinality(&self) -> usize {
        usize::from(self.cardinality) + 1
    }

    /// Finds the smallest value in the chunk.
    pub(crate) fn min(&self) -> Option<u32> {
        self.container
            .min()
            .map(|lo| Entry::from_parts(self.key, lo).into())
    }

    /// Finds the largest value in the chunk.
    pub(crate) fn max(&self) -> Option<u32> {
        self.container
            .max()
            .map(|lo| Entry::from_parts(self.key, lo).into())
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

    #[test]
    fn insertion_deletion() {
        let entry = Entry::from_parts(0, 0);
        let mut chunk = Chunk::new(&entry);

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
        let entry = Entry::from_parts(0, 42);
        let mut chunk = Chunk::new(&entry);
        assert_eq!(chunk.contains(11), false);

        chunk.insert(11);
        assert_eq!(chunk.contains(11), true);
    }

    #[test]
    fn already_exists() {
        let entry = Entry::from_parts(0, 42);
        let mut chunk = Chunk::new(&entry);

        assert_eq!(chunk.insert(42), false, "already exists");
        assert_eq!(chunk.cardinality(), 1);

        assert_eq!(chunk.insert(11), true, "new entry");
        assert_eq!(chunk.cardinality(), 2);
    }

    #[test]
    fn missing() {
        let entry = Entry::from_parts(0, 42);
        let mut chunk = Chunk::new(&entry);

        assert_eq!(chunk.remove(42), true, "found");
        assert_eq!(chunk.remove(11), false, "missing entry");
    }

    #[test]
    fn min_max() {
        let entry = Entry::from_parts(0, 42);
        let mut chunk = Chunk::new(&entry);
        assert_eq!(chunk.min(), Some(42));
        assert_eq!(chunk.max(), Some(42));

        chunk.insert(11);
        chunk.insert(100);
        chunk.insert(77);
        chunk.insert(3);
        assert_eq!(chunk.min(), Some(3));
        assert_eq!(chunk.max(), Some(100));
    }
}
