// pattern: Functional Core
//! Bounded typed transform workspace initializer; does not model a native memory layout.

/// Selected controlled descriptor extent in bytes.
pub const TRANSFORM_DESCRIPTOR_BYTES: usize = 1320;

/// Selected controlled arena extent in bytes.
pub const TRANSFORM_ARENA_BYTES: usize = 65_536;

/// Number of full-band POWER rows initialized by the selected workspace constructor.
pub const WORKSPACE_POWER_ROWS: usize = 30;

/// Number of full-band bias rows initialized by the selected workspace constructor.
pub const WORKSPACE_BIAS_ROWS: usize = 5;

/// Number of full-band scratch exponent rows initialized by the selected constructor.
pub const WORKSPACE_SCRATCH_ROWS: usize = 30;

/// Caller-supplied evidence for the untouched canary suffixes of the controlled buffers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkspaceTailCanaries {
    descriptor_tail: [u8; 24],
    arena_tail: [u8; 16],
}

impl WorkspaceTailCanaries {
    pub const fn new(descriptor_tail: [u8; 24], arena_tail: [u8; 16]) -> Self {
        Self {
            descriptor_tail,
            arena_tail,
        }
    }

    pub const fn descriptor_tail(&self) -> &[u8; 24] {
        &self.descriptor_tail
    }

    pub const fn arena_tail(&self) -> &[u8; 16] {
        &self.arena_tail
    }
}

/// Typed selected-profile workspace state returned by the bounded initializer.
///
/// Current-row binding state is represented only as bound/unbound booleans. This does not
/// reproduce native pointer values, offsets, or ABI structure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransformWorkspace {
    power_rows: Vec<[u32; 253]>,
    bias_rows: [[i16; 6]; WORKSPACE_BIAS_ROWS],
    scratch_exponent_rows: Vec<[i16; 253]>,
    full_band_current_row_bound: [bool; WORKSPACE_SCRATCH_ROWS],
    lfe_scratch_rows: [[i16; 7]; 6],
    lfe_current_row_bound: [bool; 6],
    shared_work_row: [i16; 253],
    tail_canaries: WorkspaceTailCanaries,
}

impl TransformWorkspace {
    pub const fn return_value(&self) -> i32 {
        0
    }

    pub fn power_rows(&self) -> &[[u32; 253]] {
        &self.power_rows
    }

    pub const fn bias_rows(&self) -> &[[i16; 6]; WORKSPACE_BIAS_ROWS] {
        &self.bias_rows
    }

    pub fn scratch_exponent_rows(&self) -> &[[i16; 253]] {
        &self.scratch_exponent_rows
    }

    pub const fn full_band_current_row_bound(&self) -> &[bool; WORKSPACE_SCRATCH_ROWS] {
        &self.full_band_current_row_bound
    }

    pub const fn lfe_scratch_rows(&self) -> &[[i16; 7]; 6] {
        &self.lfe_scratch_rows
    }

    pub const fn lfe_current_row_bound(&self) -> &[bool; 6] {
        &self.lfe_current_row_bound
    }

    pub const fn shared_work_row(&self) -> &[i16; 253] {
        &self.shared_work_row
    }

    pub const fn tail_canaries(&self) -> &WorkspaceTailCanaries {
        &self.tail_canaries
    }
}

/// Creates the selected typed zero workspace while returning the untouched canary tails.
///
/// The controlled package binds field counts and zero/null status but does not deliver raw
/// descriptor or arena offsets. This function therefore operates only on typed values and
/// retains canary bytes supplied by the caller; it does not write a native buffer layout.
pub fn initialize_transform_workspace(tail_canaries: WorkspaceTailCanaries) -> TransformWorkspace {
    TransformWorkspace {
        power_rows: vec![[0; 253]; WORKSPACE_POWER_ROWS],
        bias_rows: [[0; 6]; WORKSPACE_BIAS_ROWS],
        scratch_exponent_rows: vec![[0; 253]; WORKSPACE_SCRATCH_ROWS],
        full_band_current_row_bound: [false; WORKSPACE_SCRATCH_ROWS],
        lfe_scratch_rows: [[0; 7]; 6],
        lfe_current_row_bound: [false; 6],
        shared_work_row: [0; 253],
        tail_canaries,
    }
}
