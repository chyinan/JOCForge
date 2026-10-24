// pattern: Functional Core
//! Selected frontend warmup gate for caller-visible output publication.
//!
//! Gate progress is not a source-sample or absolute ADM clock.

const PUBLICATION_THRESHOLD_SAMPLES: u32 = 3_072;
const PUBLICATION_HOP_SAMPLES: u32 = 1_536;
const PUBLICATION_PROGRESS_LIMIT: u32 = PUBLICATION_THRESHOLD_SAMPLES + PUBLICATION_HOP_SAMPLES;

/// Frontend invocation ordinal, one-based and distinct from an AU ordinal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrontendCallOrdinal(u32);

impl FrontendCallOrdinal {
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Caller-visible published AU ordinal, zero-based.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublishedAuOrdinal(u32);

impl PublishedAuOrdinal {
    pub const fn value(self) -> u32 {
        self.0
    }
}

/// Cold selected outer publication gate.
#[derive(Debug, Eq, PartialEq)]
pub struct CarrierPublicationGate {
    progress_samples: u32,
    frontend_calls: u32,
    published_aus: u32,
}

impl CarrierPublicationGate {
    pub const fn cold() -> Self {
        Self {
            progress_samples: 0,
            frontend_calls: 0,
            published_aus: 0,
        }
    }

    pub const fn progress_samples(&self) -> u32 {
        self.progress_samples
    }

    pub const fn published_au_count(&self) -> u32 {
        self.published_aus
    }

    /// Advances one frontend call and assigns an AU ordinal only when bytes
    /// are caller-visible after the strict threshold check.
    pub fn advance(&mut self, internal_output_bytes: u32) -> PublicationGateReceipt {
        self.frontend_calls += 1;
        let progress_before = self.progress_samples;
        if self.progress_samples < PUBLICATION_PROGRESS_LIMIT {
            self.progress_samples += PUBLICATION_HOP_SAMPLES;
        }
        let valid_output_bytes = if self.progress_samples > PUBLICATION_THRESHOLD_SAMPLES {
            internal_output_bytes
        } else {
            0
        };
        let published_au_ordinal = if valid_output_bytes > 0 {
            let ordinal = PublishedAuOrdinal(self.published_aus);
            self.published_aus += 1;
            Some(ordinal)
        } else {
            None
        };

        PublicationGateReceipt {
            frontend_call: FrontendCallOrdinal(self.frontend_calls),
            progress_before,
            progress_after: self.progress_samples,
            valid_output_bytes,
            published_au_ordinal,
        }
    }
}

impl Default for CarrierPublicationGate {
    fn default() -> Self {
        Self::cold()
    }
}

/// Immutable selected gate result for one frontend call.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicationGateReceipt {
    frontend_call: FrontendCallOrdinal,
    progress_before: u32,
    progress_after: u32,
    valid_output_bytes: u32,
    published_au_ordinal: Option<PublishedAuOrdinal>,
}

impl PublicationGateReceipt {
    pub const fn frontend_call(self) -> FrontendCallOrdinal {
        self.frontend_call
    }

    pub const fn progress_before(self) -> u32 {
        self.progress_before
    }

    pub const fn progress_after(self) -> u32 {
        self.progress_after
    }

    pub const fn valid_output_bytes(self) -> u32 {
        self.valid_output_bytes
    }

    pub const fn published_au_ordinal(self) -> Option<PublishedAuOrdinal> {
        self.published_au_ordinal
    }
}
