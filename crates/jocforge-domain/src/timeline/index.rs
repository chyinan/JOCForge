// pattern: Functional Core
//! Distinct clock-domain indices. Their numeric representation is private.

macro_rules! index_type {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(u64);
        impl $name {
            pub const fn new(value: u64) -> Self {
                Self(value)
            }
            pub const fn value(self) -> u64 {
                self.0
            }
        }
    };
}

index_type!(AdmSampleIndex);
index_type!(NormalCallbackCount);
index_type!(FrontendCallbackIndex);
index_type!(CodecSourceIndex);
index_type!(JocDispatchIndex);
index_type!(JocGeneratedWriterIndex);
index_type!(OrdinaryPayloadWriterIndex);
index_type!(JocPoppedWriterIndex);
index_type!(PublishedAuIndex);
index_type!(ReceiverCallbackCount);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AdmSampleCount(u64);

impl AdmSampleCount {
    pub fn new(value: u64) -> Result<Self, super::ClockError> {
        if value == 0 {
            Err(super::ClockError::ZeroSampleCount)
        } else {
            Ok(Self(value))
        }
    }
    pub(crate) const fn value(self) -> u64 {
        self.0
    }
}

/// Clock domains are intentionally not interchangeable.
///
/// ```compile_fail
/// use jocforge_domain::timeline::{CodecSourceIndex, FrontendCallbackIndex};
/// fn takes_source(_: CodecSourceIndex) {}
/// takes_source(FrontendCallbackIndex::new(2));
/// ```
/// ```compile_fail
/// use jocforge_domain::timeline::{JocGeneratedWriterIndex, JocPoppedWriterIndex};
/// fn takes_popped(_: JocPoppedWriterIndex) {}
/// takes_popped(JocGeneratedWriterIndex::new(0));
/// ```
/// ```compile_fail
/// use jocforge_domain::timeline::{CodecSourceIndex, PublishedAuIndex};
/// fn takes_source(_: CodecSourceIndex) {}
/// takes_source(PublishedAuIndex::new(0));
/// ```
/// ```compile_fail
/// use jocforge_domain::timeline::{JocPoppedWriterIndex, OrdinaryPayloadWriterIndex};
/// fn takes_joc(_: JocPoppedWriterIndex) {}
/// takes_joc(OrdinaryPayloadWriterIndex::new(0));
/// ```
/// ```compile_fail
/// use jocforge_domain::timeline::{FrontendCallbackIndex, ReceiverCallbackCount};
/// fn takes_receiver(_: ReceiverCallbackCount) {}
/// takes_receiver(FrontendCallbackIndex::new(0));
/// ```
pub struct ClockDomainTypeSeparation;
