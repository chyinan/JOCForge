// pattern: Functional Core
//! Pure transaction identities and capability boundaries. No coordinator state is implemented.

/// A nonempty identity shared by callback-owner proposals and staged receipts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallbackTransactionId(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackTransactionIdError {
    Empty,
}

impl CallbackTransactionId {
    pub fn new(value: &str) -> Result<Self, CallbackTransactionIdError> {
        if value.is_empty() {
            Err(CallbackTransactionIdError::Empty)
        } else {
            Ok(Self(value.to_owned()))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
