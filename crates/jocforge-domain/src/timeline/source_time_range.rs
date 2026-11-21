// pattern: Functional Core
//! Bounded file-position ranges for the selected active JOC writer.

const SELECTED_SAMPLE_RATE_HZ: u32 = 48_000;
const RATE_SCALE_NUMERATOR: u128 = 1_001;
const RATE_SCALE_DENOMINATOR: u128 = 1_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceTimeRangeError {
    UnsupportedTimeBase,
    UnsupportedSampleRate,
    UnsupportedFrameRate,
    InvalidFrameToken,
    InvalidSilenceDuration,
    ZeroInclusiveEndUnsupported,
    EndBeforeStart,
    EmptyValidatedRange,
    SourceBounds,
    ArithmeticOverflow,
}

impl std::fmt::Display for SourceTimeRangeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::UnsupportedTimeBase => "start and end require file_position",
            Self::UnsupportedSampleRate => "expected a 48 kHz source",
            Self::UnsupportedFrameRate => "frame positions require a supported timecode_frame_rate",
            Self::InvalidFrameToken => "expected HH:MM:SS.xx, HH:MM:SS:FF or a frame number",
            Self::InvalidSilenceDuration => {
                "silence duration must be a nonnegative number of seconds"
            }
            Self::ZeroInclusiveEndUnsupported => "zero is not an admitted inclusive end here",
            Self::EndBeforeStart => "end precedes start",
            Self::EmptyValidatedRange => "start and end select an empty interval",
            Self::SourceBounds => "the selected interval exceeds the source",
            Self::ArithmeticOverflow => "the selected timeline exceeds the supported length",
        })
    }
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

/// Input to the bounded selected file-position conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedFilePositionRangeInput<'a> {
    time_base: &'a str,
    frame_rate_override: &'a str,
    start_frame_token: &'a str,
    end_frame_token: &'a str,
    sample_rate_hz: u32,
    source_samples: u64,
}

impl<'a> SelectedFilePositionRangeInput<'a> {
    pub const fn new(
        time_base: &'a str,
        frame_rate_override: &'a str,
        start_frame_token: &'a str,
        end_frame_token: &'a str,
        sample_rate_hz: u32,
        source_samples: u64,
    ) -> Self {
        Self {
            time_base,
            frame_rate_override,
            start_frame_token,
            end_frame_token,
            sample_rate_hz,
            source_samples,
        }
    }
}

/// Resolves selected numeric and approved 29.97 colon/drop-frame file-position tokens.
///
/// Numeric tokens cover 23.976, 29.97, and 59.94; colon and lowercase drop-frame forms are
/// limited to 29.97 at 48 kHz. Drop-frame conversion follows the approved static label-correction
/// rule and the exact native-confirmed run-79 vector.
/// `SampleRange` rounds positive sample products for bounds/nonempty validation; the active JOC
/// writer truncates the same positive products toward zero. Numeric end tokens advance by one;
/// colon and drop-frame end positions are used directly.
/// Embedded-timecode offset precedence is a separate rule and is not handled here.
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
    let start_token = parse_frame_token(input.start_frame_token, frame_rate)?;
    let end_token = parse_frame_token(input.end_frame_token, frame_rate)?;
    let (start_frame, end_frame_exclusive) = match (start_token, end_token) {
        (ParsedFrameToken::Numeric(start), ParsedFrameToken::Numeric(inclusive_end)) => {
            if inclusive_end == 0 {
                return Err(SourceTimeRangeError::ZeroInclusiveEndUnsupported);
            }
            if inclusive_end < start {
                return Err(SourceTimeRangeError::EndBeforeStart);
            }
            let end_exclusive = inclusive_end
                .checked_add(1)
                .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
            (start, end_exclusive)
        }
        (ParsedFrameToken::NonDropColon(start), ParsedFrameToken::NonDropColon(end))
        | (ParsedFrameToken::DropFrame(start), ParsedFrameToken::DropFrame(end)) => {
            if end < start {
                return Err(SourceTimeRangeError::EndBeforeStart);
            }
            (start, end)
        }
        _ => return Err(SourceTimeRangeError::InvalidFrameToken),
    };

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ParsedFrameToken {
    Numeric(u64),
    NonDropColon(u64),
    DropFrame(u64),
}

fn parse_frame_token(
    value: &str,
    frame_rate: SelectedNumericFrameRate,
) -> Result<ParsedFrameToken, SourceTimeRangeError> {
    if !value.is_empty() && value.is_ascii() && value.bytes().all(|byte| byte.is_ascii_digit()) {
        return value
            .parse::<u64>()
            .map(ParsedFrameToken::Numeric)
            .map_err(|_| SourceTimeRangeError::InvalidFrameToken);
    }

    if let Some(timecode) = value.strip_suffix("df") {
        return parse_drop_frame_timecode(timecode, frame_rate).map(ParsedFrameToken::DropFrame);
    }

    parse_non_drop_colon_timecode(value, frame_rate).map(ParsedFrameToken::NonDropColon)
}

fn parse_non_drop_colon_timecode(
    value: &str,
    frame_rate: SelectedNumericFrameRate,
) -> Result<u64, SourceTimeRangeError> {
    if frame_rate != SelectedNumericFrameRate::Fps2997 {
        return Err(SourceTimeRangeError::UnsupportedFrameRate);
    }

    let (hours, minutes, seconds, frames) = parse_timecode_components(value)?;
    let nominal_frames_per_second = frame_rate.nominal_frames_per_second();
    if minutes >= 60 || seconds >= 60 || frames >= nominal_frames_per_second {
        return Err(SourceTimeRangeError::InvalidFrameToken);
    }

    let total_seconds = hours
        .checked_mul(60)
        .and_then(|value| value.checked_add(minutes))
        .and_then(|value| value.checked_mul(60))
        .and_then(|value| value.checked_add(seconds))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    total_seconds
        .checked_mul(nominal_frames_per_second)
        .and_then(|value| value.checked_add(frames))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)
}

fn parse_drop_frame_timecode(
    value: &str,
    frame_rate: SelectedNumericFrameRate,
) -> Result<u64, SourceTimeRangeError> {
    if frame_rate != SelectedNumericFrameRate::Fps2997 {
        return Err(SourceTimeRangeError::UnsupportedFrameRate);
    }

    let (hours, minutes, seconds, frames) = parse_timecode_components(value)?;
    let nominal_frames_per_second = frame_rate.nominal_frames_per_second();
    if minutes >= 60 || seconds >= 60 || frames >= nominal_frames_per_second {
        return Err(SourceTimeRangeError::InvalidFrameToken);
    }

    let total_minutes = hours
        .checked_mul(60)
        .and_then(|value| value.checked_add(minutes))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    if seconds == 0 && total_minutes % 10 != 0 && frames < 2 {
        return Err(SourceTimeRangeError::InvalidFrameToken);
    }

    let elapsed_seconds = total_minutes
        .checked_mul(60)
        .and_then(|value| value.checked_add(seconds))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    let nominal_frame_index = elapsed_seconds
        .checked_mul(nominal_frames_per_second)
        .and_then(|value| value.checked_add(frames))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    let dropped_labels = total_minutes
        .checked_sub(total_minutes / 10)
        .and_then(|value| value.checked_mul(2))
        .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
    nominal_frame_index
        .checked_sub(dropped_labels)
        .ok_or(SourceTimeRangeError::InvalidFrameToken)
}

fn parse_timecode_components(value: &str) -> Result<(u64, u64, u64, u64), SourceTimeRangeError> {
    let fields = value.split(':').collect::<Vec<_>>();
    if fields.len() != 4
        || fields.iter().any(|field| {
            field.len() != 2 || !field.is_ascii() || !field.bytes().all(|b| b.is_ascii_digit())
        })
    {
        return Err(SourceTimeRangeError::InvalidFrameToken);
    }

    Ok((
        parse_timecode_field(fields[0])?,
        parse_timecode_field(fields[1])?,
        parse_timecode_field(fields[2])?,
        parse_timecode_field(fields[3])?,
    ))
}

fn parse_timecode_field(value: &str) -> Result<u64, SourceTimeRangeError> {
    value
        .parse::<u64>()
        .map_err(|_| SourceTimeRangeError::InvalidFrameToken)
}

/// The sample mapping shared by the PCM reader and the ADM object sampler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderTimeline {
    file_samples: u64,
    start: u64,
    end: u64,
    leading: u64,
    output_samples: u64,
}

impl RenderTimeline {
    pub const fn full(file_samples: u64) -> Self {
        Self {
            file_samples,
            start: 0,
            end: file_samples,
            leading: 0,
            output_samples: file_samples,
        }
    }

    pub fn from_file_position(
        file_samples: u64,
        start: &str,
        end: &str,
        rate: &str,
        leading: u64,
        trailing: u64,
    ) -> Result<Self, SourceTimeRangeError> {
        let (start, end) = Self::endpoints(start, end, rate)?;
        let end = end.unwrap_or(file_samples);
        if start >= file_samples {
            return Err(SourceTimeRangeError::SourceBounds);
        }
        if end > file_samples {
            return Err(SourceTimeRangeError::SourceBounds);
        }
        let output_samples = (end - start)
            .checked_add(leading)
            .and_then(|n| n.checked_add(trailing))
            .ok_or(SourceTimeRangeError::ArithmeticOverflow)?;
        // Frontend callback indices include four drain callbacks and fit in a u32.
        if output_samples.div_ceil(1536) > u64::from(u32::MAX - 4) {
            return Err(SourceTimeRangeError::ArithmeticOverflow);
        }
        Ok(Self {
            file_samples,
            start,
            end,
            leading,
            output_samples,
        })
    }
    pub(crate) fn endpoints(
        start: &str,
        end: &str,
        rate: &str,
    ) -> Result<(u64, Option<u64>), SourceTimeRangeError> {
        let start = if start.is_empty() {
            0
        } else {
            endpoint_samples(start, rate, false)?
        };
        let end = if end.is_empty() || end == "end_of_file" {
            None
        } else {
            Some(endpoint_samples(end, rate, true)?)
        };
        if end.is_some_and(|end| end < start) {
            return Err(SourceTimeRangeError::EndBeforeStart);
        }
        if end == Some(start) {
            return Err(SourceTimeRangeError::EmptyValidatedRange);
        }
        Ok((start, end))
    }
    pub const fn source_range(self) -> std::ops::Range<u64> {
        self.start..self.end
    }

    pub const fn file_samples(self) -> u64 {
        self.file_samples
    }
    pub const fn output_samples(self) -> u64 {
        self.output_samples
    }
    pub const fn is_edited(self) -> bool {
        self.start != 0
            || self.end != self.file_samples
            || self.leading != 0
            || self.output_samples != self.file_samples
    }
    /// Holds the first/last retained object position during the adjacent silence.
    pub fn object_sample(self, output: u64) -> u64 {
        self.start
            + output
                .saturating_sub(self.leading)
                .min((self.end - self.start).saturating_sub(1))
    }
    /// Returns (leading silent samples in this read, source offset, source sample count).
    pub fn pcm_span(self, output: u64, count: u64) -> (u64, u64, u64) {
        let skip = self.leading.saturating_sub(output).min(count);
        let offset = output
            .saturating_sub(self.leading)
            .min(self.end - self.start);
        (
            skip,
            self.start + offset,
            (count - skip).min(self.end - self.start - offset),
        )
    }
}

fn endpoint_samples(
    token: &str,
    rate: &str,
    inclusive_numeric_end: bool,
) -> Result<u64, SourceTimeRangeError> {
    use SourceTimeRangeError::{ArithmeticOverflow, InvalidFrameToken, UnsupportedFrameRate};
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    if token.matches(':').count() == 2 {
        let (clock, fraction) = token.split_once('.').unwrap_or((token, "0"));
        if !digits(fraction) || fraction.len() > 9 {
            return Err(InvalidFrameToken);
        }
        let (h, m, s, _) = parse_timecode_components(&format!("{clock}:00"))?;
        if m >= 60 || s >= 60 {
            return Err(InvalidFrameToken);
        }
        let decimal = parse_timecode_field(fraction)?;
        let scale = 10_u64.pow(u32::try_from(fraction.len()).map_err(|_| InvalidFrameToken)?);
        return Ok(((h * 60 + m) * 60 + s) * 48000 + decimal * 48000 / scale);
    }
    let (nominal, numerator, denominator) = match rate {
        "23.976" => (24, 24000, 1001),
        "29.97" => (30, 30000, 1001),
        "59.94" => (60, 60000, 1001),
        "24" | "25" | "30" | "48" | "50" | "60" => {
            let n = parse_timecode_field(rate)?;
            (n, n, 1)
        }
        _ => return Err(UnsupportedFrameRate),
    };
    let frame = if digits(token) {
        parse_timecode_field(token)?
            .checked_add(u64::from(inclusive_numeric_end))
            .ok_or(ArithmeticOverflow)?
    } else if let Some(clock) = token.strip_suffix("df") {
        parse_drop_frame_timecode(clock, SelectedNumericFrameRate::parse(rate)?)?
    } else {
        let (h, m, s, f) = parse_timecode_components(token)?;
        if m >= 60 || s >= 60 || f >= nominal {
            return Err(InvalidFrameToken);
        }
        ((h * 60 + m) * 60 + s) * nominal + f
    };
    let samples = u128::from(frame) * 48000 * denominator / u128::from(numerator);
    u64::try_from(samples).map_err(|_| ArithmeticOverflow)
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

    #[test]
    fn edited_timeline_maps_pcm_and_objects_across_partial_frames() {
        use super::RenderTimeline;
        let edit =
            RenderTimeline::from_file_position(96000, "00:00:00.125", "00:00:01:00", "25", 7, 11)
                .unwrap();
        assert_eq!(edit.output_samples(), 42018);
        assert_eq!(edit.pcm_span(0, 12), (7, 6000, 5));
        assert_eq!(edit.pcm_span(42005, 10), (0, 47998, 2));
        assert_eq!(edit.pcm_span(42008, 10), (0, 48000, 0));
        assert_eq!(edit.object_sample(0), 6000);
        assert_eq!(edit.object_sample(8), 6001);
        assert_eq!(edit.object_sample(42008), 47999);
        let frames =
            RenderTimeline::from_file_position(96000, "00:00:00:01", "", "23.976", 0, 0).unwrap();
        assert_eq!(frames.object_sample(0), 2002);
        let end_only =
            RenderTimeline::from_file_position(96000, "", "00:00:00.25", "not_indicated", 0, 0)
                .unwrap();
        assert_eq!(end_only.output_samples(), 12000);
        for (start, end, rate) in [
            ("00:00:00:01", "", "not_indicated"),
            ("00:00:00:25", "", "25"),
            ("00:00:00.10", "00:00:00.09", "25"),
            ("00:00:00", "00:00:00", "25"),
            ("", "00:00:03.0", "25"),
            ("00:60:00.0", "", "25"),
        ] {
            assert!(RenderTimeline::from_file_position(96000, start, end, rate, 0, 0).is_err());
        }
    }

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
    fn selected_non_drop_colon_timecode_2997_uses_end_frame_directly() {
        let range = resolve("29.97", "00:01:30:00", "00:01:30:10");
        assert_eq!(range.validation_start().value(), 4_324_320);
        assert_eq!(range.validation_end_exclusive().value(), 4_340_336);
        assert_eq!(range.writer_start().value(), 4_324_320);
        assert_eq!(range.writer_end_exclusive().value(), 4_340_336);
    }

    #[test]
    fn colon_timecode_parser_rejects_unapproved_rates_and_syntax() {
        let input = |rate, start, end| {
            SelectedFilePositionRangeInput::new(
                "file_position",
                rate,
                start,
                end,
                48_000,
                SOURCE_SAMPLES,
            )
        };
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "23.976",
                "00:01:30:00",
                "00:01:30:10",
            )),
            Err(SourceTimeRangeError::UnsupportedFrameRate),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "29.97",
                "00;01;30;00",
                "00;01;30;10",
            )),
            Err(SourceTimeRangeError::InvalidFrameToken),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input("29.97", "2160", "00:01:30:10",)),
            Err(SourceTimeRangeError::InvalidFrameToken),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "29.97",
                "00:01:30:30",
                "00:01:31:00",
            )),
            Err(SourceTimeRangeError::InvalidFrameToken),
        );
    }

    #[test]
    fn selected_lowercase_drop_frame_2997_matches_native_run79_range() {
        let range = resolve("29.97", "00:01:00:02df", "00:01:00:12df");
        assert_eq!(range.validation_start().value(), 2_882_880);
        assert_eq!(range.validation_end_exclusive().value(), 2_898_896);
        assert_eq!(range.writer_start().value(), 2_882_880);
        assert_eq!(range.writer_end_exclusive().value(), 2_898_896);
    }

    #[test]
    fn lowercase_drop_frame_parser_enforces_its_static_29_97_contract() {
        assert_eq!(
            super::parse_drop_frame_timecode(
                "00:10:00:00",
                super::SelectedNumericFrameRate::Fps2997
            ),
            Ok(17_982),
        );
        assert_eq!(
            super::parse_drop_frame_timecode(
                "00:10:00:01",
                super::SelectedNumericFrameRate::Fps2997
            ),
            Ok(17_983),
        );

        let input = |time_base, rate, start, end| {
            SelectedFilePositionRangeInput::new(time_base, rate, start, end, 48_000, SOURCE_SAMPLES)
        };
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "file_position",
                "23.976",
                "00:01:00:02df",
                "00:01:00:12df",
            )),
            Err(SourceTimeRangeError::UnsupportedFrameRate),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "file_position",
                "29.97",
                "00:01:00:02DF",
                "00:01:00:12DF",
            )),
            Err(SourceTimeRangeError::InvalidFrameToken),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "file_position",
                "29.97",
                "00:01:00:00df",
                "00:01:00:02df",
            )),
            Err(SourceTimeRangeError::InvalidFrameToken),
        );
        assert_eq!(
            resolve_selected_file_position_sample_range(input(
                "embedded_timecode",
                "29.97",
                "00:01:00:02df",
                "00:01:00:12df",
            )),
            Err(SourceTimeRangeError::UnsupportedTimeBase),
        );
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
