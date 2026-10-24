// pattern: Functional Core
//! Candidate normalized bed source points keyed by selected bed role.

const BED_SOURCE_POINTS: [(u8, [u32; 3]); 9] = [
    (0, [0x0000_0000, 0x0000_0000, 0x0000_0000]),
    (1, [0x3f80_0000, 0x0000_0000, 0x0000_0000]),
    (2, [0x3f00_0000, 0x0000_0000, 0x0000_0000]),
    (4, [0x0000_0000, 0x3f00_0000, 0x0000_0000]),
    (5, [0x3f80_0000, 0x3f00_0000, 0x0000_0000]),
    (6, [0x0000_0000, 0x3f80_0000, 0x0000_0000]),
    (7, [0x3f80_0000, 0x3f80_0000, 0x0000_0000]),
    (8, [0x0000_0000, 0x3f00_0000, 0x3f80_0000]),
    (9, [0x3f80_0000, 0x3f00_0000, 0x3f80_0000]),
];
const ATTRIBUTES: [u8; 5] = [0, 0, 0, 3, 2];
const ATTRIBUTE_SHIFTS: [u32; 5] = [0, 1, 4, 8, 10];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BedSourcePoint {
    bed_id: u8,
    xyz_words: [u32; 3],
    attribute_components: [u8; 5],
    packed_attribute: u32,
    defined_metadata_words: [u32; 13],
}

impl BedSourcePoint {
    pub const fn bed_id(self) -> u8 {
        self.bed_id
    }

    pub const fn xyz_words(self) -> [u32; 3] {
        self.xyz_words
    }

    pub const fn attribute_components(self) -> [u8; 5] {
        self.attribute_components
    }

    pub const fn packed_attribute(self) -> u32 {
        self.packed_attribute
    }

    pub const fn defined_metadata_words(self) -> [u32; 13] {
        self.defined_metadata_words
    }
}

/// Generates the nine regular bed points; bed id3 is the direct LFE and has no regular source point.
pub fn selected_bed_source_points() -> [BedSourcePoint; 9] {
    std::array::from_fn(|index| {
        let (bed_id, xyz_words) = BED_SOURCE_POINTS[index];
        let packed_attribute = ATTRIBUTES
            .iter()
            .zip(ATTRIBUTE_SHIFTS)
            .fold(0_u32, |packed, (value, shift)| {
                packed | (u32::from(*value) << shift)
            });
        let [x, y, z] = xyz_words;
        BedSourcePoint {
            bed_id,
            xyz_words,
            attribute_components: ATTRIBUTES,
            packed_attribute,
            defined_metadata_words: [0, 0, 3, 2, 0, x, y, z, 0, 0, 0, 0, 1],
        }
    })
}
