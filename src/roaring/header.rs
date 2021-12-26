use crate::chunk;

/// Chunk header.
pub(crate) struct Header {
    /// The 16 most significant bits.
    key: u16,
    /// Chunk's cardinality minus one.
    ///
    /// -1 allows to count up to 65536 while staying on 16-bit, and it's
    /// safe because the minimum size is 1 (empty chunks are deallocated).
    cardinality: u16,
}

impl Header {
    /// Initializes a new Chunk's header.
    pub(crate) fn new(key: u16) -> Self {
        Self {
            key,
            cardinality: 0,
        }
    }
}

impl chunk::Header for Header {
    type Key = u16;

    fn key(&self) -> Self::Key {
        self.key
    }

    fn cardinality(&self) -> usize {
        usize::from(self.cardinality) + 1
    }

    fn increase_cardinality(&mut self) {
        self.cardinality += 1;
    }

    fn decrease_cardinality(&mut self) {
        self.cardinality = self.cardinality.saturating_sub(1);
    }
}
