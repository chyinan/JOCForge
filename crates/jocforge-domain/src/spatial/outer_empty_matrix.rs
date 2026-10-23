// pattern: Functional Core
//! Stateless F03S.1 zero-target outer matrix skip.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EmptyOuterMatrixError {
    UnsupportedNonemptyTargetSet,
    InvalidSourceCallOrder,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EmptyOuterMatrixResult {
    matrix: Vec<Vec<u32>>,
    shape: [u8; 2],
    solver_calls: u8,
    empty_targets_skipped: bool,
}

impl EmptyOuterMatrixResult {
    pub fn matrix_u32(&self) -> &[Vec<u32>] {
        &self.matrix
    }

    pub const fn shape(&self) -> [u8; 2] {
        self.shape
    }

    pub const fn solver_calls(&self) -> u8 {
        self.solver_calls
    }

    pub const fn empty_targets_skipped(&self) -> bool {
        self.empty_targets_skipped
    }
}

/// Returns the selected zero-target matrix shape without consuming response or XYZ inputs.
pub fn empty_target_matrix(
    target_count: u8,
    source_call_order: &[u8],
) -> Result<EmptyOuterMatrixResult, EmptyOuterMatrixError> {
    if target_count != 0 {
        return Err(EmptyOuterMatrixError::UnsupportedNonemptyTargetSet);
    }
    let mut seen = [false; 48];
    for source in source_call_order {
        let slot = seen
            .get_mut(usize::from(*source))
            .ok_or(EmptyOuterMatrixError::InvalidSourceCallOrder)?;
        if *slot {
            return Err(EmptyOuterMatrixError::InvalidSourceCallOrder);
        }
        *slot = true;
    }
    Ok(EmptyOuterMatrixResult {
        matrix: Vec::new(),
        shape: [0, 48],
        solver_calls: 0,
        empty_targets_skipped: true,
    })
}
