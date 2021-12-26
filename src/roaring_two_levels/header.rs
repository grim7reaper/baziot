use crate::chunk;

/// Chunk header.
pub(super) struct Header {
    /// Header's data.
    ///
    /// Contains both the chunk's key (in the upper 48 bits) and the chunk's
    /// cardinality minus one (in the lower 16 bits) packed into a single
    /// 64-bit integer.
    ///
    /// Storing `cardinality - 1` allows to count up to 65536 while staying on
    /// 16-bit (that way it fits alongside the key), and it's safe because the
    /// minimum size is 1 (empty chunks are deallocated).
    data: u64,
}

impl Header {
    /// Initializes a new Chunk's header.
    pub(super) fn new(key: u64) -> Self {
        Self { data: key << 16 }
    }

    /// Extracts the cardinality from the packed data field.
    #[allow(clippy::cast_possible_truncation)] // We truncate on purpose here.
    fn unpack_cardinality(&self) -> u16 {
        (self.data & 0xFFFF) as u16
    }

    /// Packs a new cardinality value into the packed data field.
    fn pack_cardinality(&mut self, cardinality: u16) {
        const CARDINALITY_MASK: u64 = 0xFFFF_FFFF_FFFF_0000;
        self.data = (self.data & CARDINALITY_MASK) | u64::from(cardinality);
    }
}

impl chunk::Header for Header {
    type Key = u64;

    fn key(&self) -> Self::Key {
        self.data >> 16
    }

    fn cardinality(&self) -> usize {
        usize::from(self.unpack_cardinality()) + 1
    }

    fn increase_cardinality(&mut self) {
        let cardinality = self.unpack_cardinality() + 1;
        self.pack_cardinality(cardinality);
    }

    fn decrease_cardinality(&mut self) {
        let cardinality = self.unpack_cardinality().saturating_sub(1);
        self.pack_cardinality(cardinality);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::Header as HeaderTrait;

    #[test]
    fn header() {
        let mut header = Header::new(0xFEED_DEAD_BEEF);
        assert_eq!(header.data, 0xFEED_DEAD_BEEF_0000);
        assert_eq!(header.key(), 0xFEED_DEAD_BEEF);
        assert_eq!(header.unpack_cardinality(), 0);

        header.increase_cardinality();
        assert_eq!(header.key(), 0xFEED_DEAD_BEEF);
        assert_eq!(header.unpack_cardinality(), 1);

        header.decrease_cardinality();
        assert_eq!(header.key(), 0xFEED_DEAD_BEEF);
        assert_eq!(header.unpack_cardinality(), 0);
    }
}
