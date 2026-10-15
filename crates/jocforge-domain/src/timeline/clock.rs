// pattern: Functional Core
use super::{
    AdmSampleCount, CodecSourceIndex, FrontendCallbackIndex, JocDispatchIndex,
    JocGeneratedWriterIndex, JocPoppedWriterIndex, NormalCallbackCount, OrdinaryPayloadWriterIndex,
    PublishedAuIndex,
};

const SAMPLES_PER_CALLBACK: u64 = 1536;
const CACHED_DRAINS: u64 = 4;
const PRIMING_CALLBACKS: u64 = 2;
const SUPPRESSED_SOURCES: u64 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClockError {
    ZeroSampleCount,
    ArithmeticUnderflow,
    ArithmeticOverflow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgrammeCounts {
    normal: NormalCallbackCount,
    padding: u64,
    total: u64,
    sources: u64,
    published: u64,
    last_real: u64,
}
impl ProgrammeCounts {
    pub const fn as_tuple(self) -> (u64, u64, u64, u64, u64, u64) {
        (
            self.normal.value(),
            self.padding,
            self.total,
            self.sources,
            self.published,
            self.last_real,
        )
    }
    pub fn ordinary_pending(self) -> Result<[OrdinaryPayloadWriterIndex; 2], ClockError> {
        let next = self
            .published
            .checked_add(1)
            .ok_or(ClockError::ArithmeticOverflow)?;
        Ok([
            OrdinaryPayloadWriterIndex::new(self.published),
            OrdinaryPayloadWriterIndex::new(next),
        ])
    }
    pub fn joc_pending(self) -> Result<JocGeneratedWriterIndex, ClockError> {
        self.sources
            .checked_sub(1)
            .map(JocGeneratedWriterIndex::new)
            .ok_or(ClockError::ArithmeticUnderflow)
    }
}

pub fn programme_counts(samples: AdmSampleCount) -> Result<ProgrammeCounts, ClockError> {
    let n = samples.value();
    let rounded = n
        .checked_add(SAMPLES_PER_CALLBACK - 1)
        .ok_or(ClockError::ArithmeticOverflow)?;
    let normal = rounded / SAMPLES_PER_CALLBACK;
    let total = normal
        .checked_add(CACHED_DRAINS)
        .ok_or(ClockError::ArithmeticOverflow)?;
    let sources = total
        .checked_sub(PRIMING_CALLBACKS)
        .ok_or(ClockError::ArithmeticUnderflow)?;
    let published = sources
        .checked_sub(SUPPRESSED_SOURCES)
        .ok_or(ClockError::ArithmeticUnderflow)?;
    let rem = n % SAMPLES_PER_CALLBACK;
    let last_real = if rem == 0 { SAMPLES_PER_CALLBACK } else { rem };
    let padding = SAMPLES_PER_CALLBACK
        .checked_sub(last_real)
        .ok_or(ClockError::ArithmeticUnderflow)?;
    Ok(ProgrammeCounts {
        normal: NormalCallbackCount::new(normal),
        padding,
        total,
        sources,
        published,
        last_real,
    })
}
pub fn source_for_callback(g: FrontendCallbackIndex) -> Option<CodecSourceIndex> {
    g.value()
        .checked_sub(PRIMING_CALLBACKS)
        .map(CodecSourceIndex::new)
}
impl CodecSourceIndex {
    pub fn dispatch(self) -> Result<JocDispatchIndex, ClockError> {
        self.value()
            .checked_add(1)
            .map(JocDispatchIndex::new)
            .ok_or(ClockError::ArithmeticOverflow)
    }
}
impl JocDispatchIndex {
    pub fn generated_writer(self) -> Result<JocGeneratedWriterIndex, ClockError> {
        self.value()
            .checked_sub(1)
            .map(JocGeneratedWriterIndex::new)
            .ok_or(ClockError::ArithmeticUnderflow)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublishedConsumption {
    pub au: PublishedAuIndex,
    pub ordinary: OrdinaryPayloadWriterIndex,
    pub joc_popped: JocPoppedWriterIndex,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationDisposition {
    Suppressed {
        joc_popped: Option<JocPoppedWriterIndex>,
    },
    Published(PublishedConsumption),
}
pub fn publication_for_source(s: CodecSourceIndex) -> Result<PublicationDisposition, ClockError> {
    match s.value() {
        0 => Ok(PublicationDisposition::Suppressed { joc_popped: None }),
        1 => Ok(PublicationDisposition::Suppressed {
            joc_popped: Some(JocPoppedWriterIndex::new(0)),
        }),
        v => {
            let j = v.checked_sub(2).ok_or(ClockError::ArithmeticUnderflow)?;
            let popped = v.checked_sub(1).ok_or(ClockError::ArithmeticUnderflow)?;
            Ok(PublicationDisposition::Published(PublishedConsumption {
                au: PublishedAuIndex::new(j),
                ordinary: OrdinaryPayloadWriterIndex::new(j),
                joc_popped: JocPoppedWriterIndex::new(popped),
            }))
        }
    }
}
