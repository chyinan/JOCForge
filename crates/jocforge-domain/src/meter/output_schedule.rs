// pattern: Functional Core
//! F06W.1/.2 synthetic output-sample and complete-1024 accounting ledger.

const CALLBACK_SAMPLES: u64 = 1_536;
const LATENCY_SAMPLES: u64 = 3_104;
const DRAIN_CALLBACKS: u64 = 3;
const PCM_BLOCK_SAMPLES: u64 = 1_024;
const ENERGY_WINDOW_SAMPLES: u64 = 960;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterOutputScheduleError {
    InvalidSourceSampleCount,
    ArithmeticOverflow,
    SpanCapacityUnavailable,
    AccountingInvariant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeterOutputPhase {
    NormalSource,
    DplmZeroInputDrain,
}

impl MeterOutputPhase {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NormalSource => "NORMAL_SOURCE",
            Self::DplmZeroInputDrain => "DPLM_ZERO_INPUT_DRAIN",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MeterOutputSpan {
    callback_index: u64,
    phase: MeterOutputPhase,
    raw_frame_sample_begin: u64,
    samples: u64,
    valid_programme_sample_begin: u64,
}

impl MeterOutputSpan {
    pub const fn callback_index(&self) -> u64 {
        self.callback_index
    }

    pub const fn phase_name(&self) -> &'static str {
        self.phase.as_str()
    }

    pub const fn raw_frame_sample_begin(&self) -> u64 {
        self.raw_frame_sample_begin
    }

    pub const fn samples(&self) -> u64 {
        self.samples
    }

    pub const fn valid_programme_sample_begin(&self) -> u64 {
        self.valid_programme_sample_begin
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MeterOutputSchedule {
    normal_callbacks: u64,
    drain_callbacks: u64,
    published_samples: u64,
    source_padding_samples: u64,
    consumed1024_samples: u64,
    complete960_windows: u64,
    uncommitted_pcm_samples: u64,
    uncommitted_energy_samples: u64,
    spans: Vec<MeterOutputSpan>,
}

impl MeterOutputSchedule {
    pub const fn normal_callbacks(&self) -> u64 {
        self.normal_callbacks
    }

    pub const fn drain_callbacks(&self) -> u64 {
        self.drain_callbacks
    }

    pub const fn published_samples(&self) -> u64 {
        self.published_samples
    }

    pub const fn source_padding_samples(&self) -> u64 {
        self.source_padding_samples
    }

    pub const fn consumed1024_samples(&self) -> u64 {
        self.consumed1024_samples
    }

    pub const fn complete960_windows(&self) -> u64 {
        self.complete960_windows
    }

    pub const fn uncommitted_pcm_samples(&self) -> u64 {
        self.uncommitted_pcm_samples
    }

    pub const fn uncommitted_energy_samples(&self) -> u64 {
        self.uncommitted_energy_samples
    }

    pub fn spans(&self) -> &[MeterOutputSpan] {
        &self.spans
    }
}

/// Computes the F06W sample-accounting ledger for one positive source length.
///
/// This function emits only spans and integer sample counts. It does not read
/// PCM, run a filter, update a six-lane 960-window state, or emit energy pairs.
pub fn calculate_meter_output_schedule(
    input_source_samples: u64,
) -> Result<MeterOutputSchedule, MeterOutputScheduleError> {
    if input_source_samples == 0 {
        return Err(MeterOutputScheduleError::InvalidSourceSampleCount);
    }

    let normal_callbacks = (input_source_samples - 1)
        .checked_div(CALLBACK_SAMPLES)
        .and_then(|count| count.checked_add(1))
        .ok_or(MeterOutputScheduleError::ArithmeticOverflow)?;
    let published_samples = normal_callbacks
        .checked_mul(CALLBACK_SAMPLES)
        .ok_or(MeterOutputScheduleError::ArithmeticOverflow)?;
    let callback_count = normal_callbacks
        .checked_add(DRAIN_CALLBACKS)
        .ok_or(MeterOutputScheduleError::ArithmeticOverflow)?;
    let span_capacity = usize::try_from(callback_count)
        .map_err(|_| MeterOutputScheduleError::SpanCapacityUnavailable)?;
    let mut spans = Vec::new();
    spans
        .try_reserve_exact(span_capacity)
        .map_err(|_| MeterOutputScheduleError::SpanCapacityUnavailable)?;

    let mut startup = LATENCY_SAMPLES;
    let mut pending = 0_u64;
    let mut processed = 0_u64;
    for callback_index in 0..callback_count {
        let phase = if callback_index < normal_callbacks {
            pending = pending
                .checked_add(CALLBACK_SAMPLES)
                .ok_or(MeterOutputScheduleError::ArithmeticOverflow)?;
            MeterOutputPhase::NormalSource
        } else {
            MeterOutputPhase::DplmZeroInputDrain
        };

        let skipped = startup.min(CALLBACK_SAMPLES);
        startup -= skipped;
        let available = CALLBACK_SAMPLES - skipped;
        let valid = pending.min(available);
        if valid > 0 {
            spans.push(MeterOutputSpan {
                callback_index,
                phase,
                raw_frame_sample_begin: skipped,
                samples: valid,
                valid_programme_sample_begin: processed,
            });
            processed = processed
                .checked_add(valid)
                .ok_or(MeterOutputScheduleError::ArithmeticOverflow)?;
            pending -= valid;
        }
    }

    if startup != 0 || pending != 0 || processed != published_samples {
        return Err(MeterOutputScheduleError::AccountingInvariant);
    }

    let consumed1024_samples = (processed / PCM_BLOCK_SAMPLES) * PCM_BLOCK_SAMPLES;
    let complete960_windows = consumed1024_samples / ENERGY_WINDOW_SAMPLES;
    let windowed_energy_samples = complete960_windows
        .checked_mul(ENERGY_WINDOW_SAMPLES)
        .ok_or(MeterOutputScheduleError::ArithmeticOverflow)?;
    Ok(MeterOutputSchedule {
        normal_callbacks,
        drain_callbacks: DRAIN_CALLBACKS,
        published_samples,
        source_padding_samples: published_samples - input_source_samples,
        consumed1024_samples,
        complete960_windows,
        uncommitted_pcm_samples: processed - consumed1024_samples,
        uncommitted_energy_samples: consumed1024_samples - windowed_energy_samples,
        spans,
    })
}
