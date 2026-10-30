// pattern: Functional Core
//! Bounded numeric file-position ranges for the selected active JOC writer.

const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
const RATE_SCALE_NUMERATOR: u128 = 1_001;
const RATE_SCALE_DENOMINATOR: u128 = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceTimeRangeError {
    UnsupportedTimeBase,
    UnsupportedSampleRate,
    UnsupportedFrameRate,
    InvalidFrameToken,
    ZeroInclusiveEndUnsupported,
    EndBeforeStart,
    EmptyValidatedRange,
    SourceBounds,
    ArithmeticOverflow,
}

/// A selected numeric filter-rate override supported by the washed writer vectors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedNumericFrameRate {
    Fps23976,
    Fps2997,
    Fps5994,
}

impl SelectedNumericFrameRate {
    pub const fn parse(value: &str) -> Result<Self, SourceTimeRangeError> {
        match value.as_bytes() {
            b"23.976" => Ok(Self::Fps23976),
            b"29.97" => Ok(Self::Fps2997),
            b"59.94" => Ok(Self::Fps5994),
            _ => Err(SourceTimeRangeError::UnsupportedFrameRate),
        }
    }

    const fn nominal_frames_per_second(self) -> u64 {
        match self {
            Self::Fps23976 => 24,
            Self::Fps2997 => 30,
            Self::Fps5994 => 60,
        }
    }
}

/// A validation-only sample boundary produced by the rounded SampleRange rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ValidatedSampleIndex(u64);

impl ValidatedSampleIndex {
    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A final active-writer sample boundary produced by truncating toward zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActiveWriterSampleIndex(i64);

impl ActiveWriterSampleIndex {
    pub const fn value(self) -> i64 {
        self.0
    }
}

/// Validation bounds and final writer bounds for one selected numeric file-position range.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedFilePositionSampleRange {
    validation_start: ValidatedSampleIndex,
    validation_end_exclusive: ValidatedSampleIndex,
    writer_start: ActiveWriterSampleIndex,
    writer_end_exclusive: ActiveWriterSampleIndex,
}

impl SelectedFilePositionSampleRange {
    pub const fn validation_start(self) -> ValidatedSampleIndex {
        self.validation_start
    }

    pub const fn validation_end_exclusive(self) -> ValidatedSampleIndex {
        self.validation_end_exclusive
    }

    pub const fn writer_start(self) -> ActiveWriterSampleIndex {
        self.writer_start
    }

    pub const fn writer_end_exclusive(self) -> ActiveWriterSampleIndex {
        self.writer_end_exclusive
    }
}

/// Input to the bounded selected numeric file-position conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedFilePositionRangeInput<'a> {
    time_base: &'a str,
    frame_rate_override: &'a str,
    start_frame_token: &'a str,
    inclusive_end_frame_token: &'a str,
    sample_rate_hz: u32,
    source_samples: u64,
}

impl<'a> SelectedFilePositionRangeInput<'a> {
    pub const fn new(
        time_base: &'a str,
        frame_rate_override: &'a str,
        start_frame_token: &'a str,
        inclusive_end_frame_token: &'a str,
        sample_rate_hz: u32,
        source_samples: u64,
    ) -> Self {
        Self {
            time_base,
            frame_rate_override,
            start_frame_token,
            inclusive_end_frame_token,
            sample_rate_hz,
            source_samples,
        }
    }
}

/// Resolves numeric file-position frame tokens into distinct validation and writer clocks.
///
/// The selected rule covers only 23.976, 29.97, and 59.94 filter-rate overrides at 48 kHz.
/// `SampleRange` rounds positive sample products for bounds/nonempty validation; the active JOC
/// writer truncates the same positive products toward zero. Embedded-timecode offset precedence
/// is a separate rule and is not handled here.
pub fn resolve_selected_file_position_sample_range(
    input: SelectedFilePositionRangeInput<'_>,
) -> Result<SelectedFilePositionSampleRange, SourceTimeRangeError> {
    if input.time_base != "file_position" {
        return Err(SourceTimeRangeError::UnsupportedTimeBase);
    }
    if input.sample_rate_hz != SELECTED_SAMPLE_RATE_HZ || input.source_samples == 0 {
        return Err(SourceTimeRangeError::UnsupportedSampleRate);
    }

    let frame_rate = SelectedNumericFrameRate::parse(input.frame_rate_override)?;
    let start_frame = parse_frame_token(input.start_frame_token)?;
    let inclusive_end_frame = parse_frame_token(input.inclusive_end_frame_token)?;
    if inclusive_end_frame == 0 {
        return Err(SourceTimeRangeError::ZeroInclusiveEndUnsupported);
    }
    if inclusive_end_frame < start_frame {
        return Err(SourceTimeRangeError::EndBeforeStart);
    }
    let end_frame_exclusive = inclusive_end_frame
        .checked_add(1)
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;

    let start_product = sample_product(start_frame, frame_rate)?;
    let end_product = sample_product(end_frame_exclusive, frame_rate)?;
    let validation_start = round_positive_product(start_product)?;
    let validation_end_exclusive = round_positive_product(end_product)?;
    if validation_end_exclusive <= validation_start {
        return Err(SourceTimeRangeError::EmptyValidatedRange);
    }
    if validation_start > input.source_samples || validation_end_exclusive > input.source_samples {
        return Err(SourceTimeRangeError::SourceBounds);
    }

    Ok(SelectedFilePositionSampleRange {
        validation_start: ValidatedSampleIndex(validation_start),
        validation_end_exclusive: ValidatedSampleIndex(validation_end_exclusive),
        writer_start: ActiveWriterSampleIndex(truncate_positive_product(start_product)?),
        writer_end_exclusive: ActiveWriterSampleIndex(truncate_positive_product(end_product)?),
    })
}

fn parse_frame_token(value: &str) -> Result<u64, SourceTimeRangeError> {
    if value.is_empty() || !value.is_ascii() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(SourceTimeRangeError::InvalidFrameToken);
    }
    value
        .parse::<u64>()
        .map_err(|_| SourceTimeRangeError::InvalidFrameToken)
}

fn sample_product(
    frame: u64,
    frame_rate: SelectedNumericFrameRate,
) -> Result<(u128, u128), SourceTimeRangeError> {
    let numerator = u128::from(frame)
        .checked_mul(RATE_SCALE_NUMERATOR)
        .and_then(|value| value.checked_mul(u128::from(SELECTED_SAMPLE_RATE_HZ)))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    let denominator = u128::from(frame_rate.nominal_frames_per_second())
        .checked_mul(RATE_SCALE_DENOMINATOR)
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    Ok((numerator, denominator))
}

fn round_positive_product(
    (numerator, denominator): (u128, u128),
) -> Result<u64, SourceTimeRangeError> {
    let rounded = numerator
        .checked_add(denominator / 2)
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?
        / denominator;
    u64::try_from(rounded).map_err(|_| SourceTimeRangeError::ArithmeticOverflow)
}

fn truncate_positive_product(
    (numerator, denominator): (u128, u128),
) -> Result<i64, SourceTimeRangeError> {
    let truncated = numerator / denominator;
    i64::try_from(truncated).map_err(|_| SourceTimeRangeError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
    use super::{
        SelectedFilePositionRangeInput, SourceTimeRangeError,
        resolve_selected_file_position_sample_range,
    };

    const SOURCE_SAMPLES: u64 = 6_522_906;

    fn resolve(
        rate: &str,
        start: &str,
        inclusive_end: &str,
    ) -> super::SelectedFilePositionSampleRange {
        resolve_selected_file_position_sample_range(SelectedFilePositionRangeInput::new(
            "file_position",
            rate,
            start,
            inclusive_end,
            48_000,
            SOURCE_SAMPLES,
        ))
        .unwrap()
    }

    #[test]
    fn active_writer_truncation_is_distinct_from_sample_range_validation_rounding() {
        let range = resolve("29.97", "2160", "2162");
        assert_eq!(range.validation_start().value(), 3_459_456);
        assert_eq!(range.validation_end_exclusive().value(), 3_464_261);
        assert_eq!(range.writer_start().value(), 3_459_456);
        assert_eq!(range.writer_end_exclusive().value(), 3_464_260);
    }

    #[test]
    fn selected_5994_and_23976_file_position_vectors_match_washed_ranges() {
        let rate_5994 = resolve("59.94", "2160", "2172");
        assert_eq!(rate_5994.writer_start().value(), 1_729_728);
        assert_eq!(rate_5994.writer_end_exclusive().value(), 1_740_138);
        assert_eq!(rate_5994.validation_end_exclusive().value(), 1_740_138);

        let rate_23976 = resolve("23.976", "2160", "2172");
        assert_eq!(rate_23976.writer_start().value(), 4_324_320);
        assert_eq!(rate_23976.writer_end_exclusive().value(), 4_350_346);
        assert_eq!(rate_23976.validation_end_exclusive().value(), 4_350_346);
    }

    #[test]
    fn unsupported_timebases_rates_and_tokens_reject() {
        let input = |time_base, rate, start, end, sample_rate| {
            SelectedFilePositionRangeInput::new(
                time_base,
                rate,
                start,
                end,
                sample_rate,
                SOURCE_SAMPLES,
            )
        };
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "embedded_timecode",
                "29.97",
                "2160",
                "2162",
                48_000,
            )),
            Err(SourceTimeRangeError::UnsupportedTimeBase),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "file_position",
                "not_indicated",
                "2160",
                "2162",
                48_000,
            )),
            Err(SourceTimeRangeError::UnsupportedFrameRate),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "file_position",
                "29.97",
                "21:60",
                "2162",
                48_000,
            )),
            Err(SourceTimeRangeError::InvalidFrameToken),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "file_position",
                "29.97",
                "2160",
                "2162",
                44_100,
            )),
            Err(SourceTimeRangeError::UnsupportedSampleRate),
        );
    }

    #[test]
    fn rounded_sample_range_must_be_nonempty_and_within_the_source() {
        assert_eq!(
            resolve_selected_file_position_sample_range(SelectedFilePositionRangeInput::new(
                "file_position",
                "29.97",
                "2163",
                "2162",
                48_000,
                SOURCE_SAMPLES,
            )),
            Err(SourceTimeRangeError::EndBeforeStart),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(SelectedFilePositionRangeInput::new(
                "file_position",
                "29.97",
                "2160",
                "2162",
                48_000,
                1_000,
            )),
            Err(SourceTimeRangeError::SourceBounds),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(SelectedFilePositionRangeInput::new(
                "file_position",
                "29.97",
                "0",
                "0",
                48_000,
                SOURCE_SAMPLES,
            )),
            Err(SourceTimeRangeError::ZeroInclusiveEndUnsupported),
        );
    }
}
