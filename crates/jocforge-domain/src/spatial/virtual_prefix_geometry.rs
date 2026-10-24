// pattern: Functional Core
//! Candidate selected eleven-point Q15 virtual-prefix geometry values.

const POINTS_Q15: [[i32; 3]; 11] = [
    [0, 0, 0],
    [32_767, 0, 0],
    [16_384, 0, 0],
    [0, 16_384, 0],
    [32_767, 16_384, 0],
    [0, 32_767, 0],
    [32_767, 32_767, 0],
    [0, 0, 32_767],
    [32_767, 0, 32_767],
    [0, 32_767, 32_767],
    [32_767, 32_767, 32_767],
];
const ATTRIBUTES: [u8; 5] = [0, 0, 0, 3, 2];
const ATTRIBUTE_SHIFTS: [u32; 5] = [0, 1, 4, 8, 10];
const Q15_DENOMINATOR: f32 = 32_768.0;

/// Prefix slot, distinct from physical host and global codec source indices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VirtualPrefixIndex(u8);

impl VirtualPrefixIndex {
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// One candidate prefix position and its metadata words 4..55.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VirtualPrefixRow {
    index: VirtualPrefixIndex,
    q15_xyz: [i32; 3],
    xyz_words: [u32; 3],
    attribute_components: [u8; 5],
    packed_attribute: u32,
    defined_metadata_words: [u32; 13],
}

impl VirtualPrefixRow {
    pub const fn index(self) -> VirtualPrefixIndex {
        self.index
    }

    pub const fn q15_xyz(self) -> [i32; 3] {
        self.q15_xyz
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

/// Candidate numeric values only; no table identity, shared owner, or audio state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateVirtualPrefix([VirtualPrefixRow; 11]);

impl CandidateVirtualPrefix {
    pub const fn rows(&self) -> &[VirtualPrefixRow; 11] {
        &self.0
    }

    pub const fn source_count(self) -> u8 {
        11
    }

    pub const fn total_source_count(self) -> u8 {
        48
    }
}

/// Generates the washed candidate Q15 point recipe and one-time F32 conversion.
pub fn build_candidate_virtual_prefix() -> CandidateVirtualPrefix {
    CandidateVirtualPrefix(std::array::from_fn(|ordinal| {
        let q15_xyz = POINTS_Q15[ordinal];
        let xyz_words = q15_xyz.map(|coordinate| (coordinate as f32 / Q15_DENOMINATOR).to_bits());
        let packed_attribute = ATTRIBUTES
            .iter()
            .zip(ATTRIBUTE_SHIFTS)
            .fold(0_u32, |packed, (value, shift)| {
                packed | (u32::from(*value) << shift)
            });
        let [x, y, z] = xyz_words;
        let defined_metadata_words = [0, 0, 3, 2, 0, x, y, z, 0, 0, 0, 0, 1];
        VirtualPrefixRow {
            index: VirtualPrefixIndex(u8::try_from(ordinal).unwrap_or(u8::MAX)),
            q15_xyz,
            xyz_words,
            attribute_components: ATTRIBUTES,
            packed_attribute,
            defined_metadata_words,
        }
    }))
}
