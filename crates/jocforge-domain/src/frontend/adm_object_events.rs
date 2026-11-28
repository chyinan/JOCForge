// pattern: Functional Core
//! Selected F01G/F01E object timeline extraction and admission.

use roxmltree::Node;

use crate::frontend::object_control::SelectedObjectEvent;
use crate::timeline::{AdmSampleCount, AdmSampleIndex};

const SAMPLE_RATE_HZ: u32 = 48_000;
const MAX_SIGNED_SAMPLE_EXCLUSIVE: f64 = 9_223_372_036_854_775_808.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectedAdmObjectEventError {
    MissingAttribute,
    MissingBlockField,
    InvalidTimestamp,
    TimestampOutOfRange,
    NonFiniteCoordinate,
    InvalidCartesianMode,
    InvalidJumpPosition,
    InvalidInterpolationLength,
    DurationMismatch,
    ProgrammeNotCovered,
    SpreadMismatch,
}

impl SelectedAdmObjectEventError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::MissingAttribute => "ADM_EVENT_ATTRIBUTE",
            Self::MissingBlockField => "ADM_EVENT_FIELD",
            Self::InvalidTimestamp => "ADM_EVENT_TIME_SYNTAX",
            Self::TimestampOutOfRange => "ADM_EVENT_TIME_RANGE",
            Self::NonFiniteCoordinate => "ADM_EVENT_NONFINITE_COORDINATE",
            Self::InvalidCartesianMode => "ADM_EVENT_CARTESIAN",
            Self::InvalidJumpPosition => "ADM_EVENT_JUMP_POSITION",
            Self::InvalidInterpolationLength => "ADM_EVENT_INTERPOLATION",
            Self::DurationMismatch => "ADM_EVENT_DURATION_MISMATCH",
            Self::ProgrammeNotCovered => "ADM_EVENT_PROGRAMME_DURATION",
            Self::SpreadMismatch => "ADM_EVENT_SPREAD_MISMATCH",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct RawAdmObjectEvent {
    pub(crate) physical_host: u8,
    pub(crate) track_uid: String,
    pub(crate) object_id: String,
    object_start: String,
    object_duration: String,
    block_rtime: String,
    block_duration: String,
    cartesian: Option<String>,
    jump_position: Option<String>,
    interpolation_length: Option<String>,
    coordinates: [Option<f64>; 3],
    width: Option<f64>,
    depth: Option<f64>,
    height: Option<f64>,
    diffuse: Option<f64>,
}

impl RawAdmObjectEvent {
    pub(crate) fn from_xml(
        physical_host: u8,
        track_uid: String,
        object_id: String,
        object: Node<'_, '_>,
        block: Node<'_, '_>,
        namespace: Option<&str>,
    ) -> Result<Self, SelectedAdmObjectEventError> {
        let object_start = required_attribute(object, "start")?;
        let object_duration = required_attribute(object, "duration")?;
        let block_rtime = required_attribute(block, "rtime")?;
        let block_duration = required_attribute(block, "duration")?;
        let jump_position_node = first_direct_child(block, "jumpPosition", namespace);
        let jump_position = jump_position_node
            .and_then(|node| node.text())
            .map(str::trim)
            .map(str::to_owned);
        let interpolation_length = jump_position_node
            .and_then(|node| node.attribute("interpolationLength"))
            .map(str::to_owned);

        Ok(Self {
            physical_host,
            track_uid,
            object_id,
            object_start,
            object_duration,
            block_rtime,
            block_duration,
            cartesian: first_direct_child_text(block, "cartesian", namespace),
            jump_position,
            interpolation_length,
            coordinates: read_coordinates(block, namespace)?,
            width: optional_binary64_child(block, "width", namespace)?,
            depth: optional_binary64_child(block, "depth", namespace)?,
            height: optional_binary64_child(block, "height", namespace)?,
            diffuse: optional_binary64_child(block, "diffuse", namespace)?,
        })
    }
}

/// Admitted event retains graph identity and sample-domain facts alongside values.
#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedSelectedObjectEvent {
    physical_host: u8,
    track_uid: String,
    object_id: String,
    object_start: AdmSampleIndex,
    block_rtime: AdmSampleIndex,
    interpolation_length: AdmSampleIndex,
    object_duration: AdmSampleIndex,
    block_duration: AdmSampleIndex,
    pub(crate) timeline: std::sync::Arc<[ObjectBlock]>,
    pub(crate) render_timeline: Option<crate::timeline::RenderTimeline>,
    event: SelectedObjectEvent,
}

impl AdmittedSelectedObjectEvent {
    pub const fn physical_host(&self) -> u8 {
        self.physical_host
    }

    pub fn track_uid(&self) -> &str {
        &self.track_uid
    }

    pub fn object_id(&self) -> &str {
        &self.object_id
    }

    pub const fn object_start(&self) -> AdmSampleIndex {
        self.object_start
    }

    pub const fn block_rtime(&self) -> AdmSampleIndex {
        self.block_rtime
    }

    pub const fn interpolation_length(&self) -> AdmSampleIndex {
        self.interpolation_length
    }

    pub const fn object_duration(&self) -> AdmSampleIndex {
        self.object_duration
    }

    pub const fn block_duration(&self) -> AdmSampleIndex {
        self.block_duration
    }

    pub const fn control_event(&self) -> &SelectedObjectEvent {
        &self.event
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AdmittedSelectedObjectEvents(Vec<AdmittedSelectedObjectEvent>);

impl AdmittedSelectedObjectEvents {
    pub fn events(&self) -> &[AdmittedSelectedObjectEvent] {
        &self.0
    }

    pub fn control_events(&self) -> Vec<SelectedObjectEvent> {
        self.0
            .iter()
            .map(|event| event.control_event().to_owned())
            .collect()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ObjectBlock {
    start: u64,
    ramp_samples: u64,
    target: SelectedObjectEvent,
}

pub(crate) fn sample_object_blocks(
    initial: SelectedObjectEvent,
    blocks: &[ObjectBlock],
    sample: u64,
) -> SelectedObjectEvent {
    let count = blocks.partition_point(|b| b.start <= sample);
    if count == 0 {
        return initial;
    }
    let block = &blocks[count - 1];
    let elapsed = sample - block.start;
    if elapsed >= block.ramp_samples {
        return block.target;
    }
    let previous = if count == 1 {
        initial
    } else {
        blocks[count - 2].target
    };
    block
        .target
        .interpolate_from(previous, elapsed as f64 / block.ramp_samples as f64)
}

fn interpolation_samples(
    raw: &RawAdmObjectEvent,
    duration: u64,
) -> Result<u64, SelectedAdmObjectEventError> {
    match raw.jump_position.as_deref() {
        None | Some("0") => Ok(duration),
        Some("1") => {
            let seconds = raw.interpolation_length.as_deref().unwrap_or("0");
            let value = trim_xml_number(seconds)
                .parse::<f64>()
                .map_err(|_| SelectedAdmObjectEventError::InvalidInterpolationLength)?;
            if !value.is_finite() || value < 0.0 {
                return Err(SelectedAdmObjectEventError::InvalidInterpolationLength);
            }
            // A transition cannot extend into the following block.
            #[allow(clippy::cast_sign_loss)] // validated nonnegative above
            let samples = (value * f64::from(SAMPLE_RATE_HZ)).round() as u64;
            Ok(samples.min(duration))
        }
        _ => Err(SelectedAdmObjectEventError::InvalidJumpPosition),
    }
}

pub(crate) fn admit_selected_object_events(
    tracks: &[Vec<RawAdmObjectEvent>],
    source_samples: AdmSampleCount,
    edit: Option<crate::timeline::RenderTimeline>,
) -> Result<AdmittedSelectedObjectEvents, SelectedAdmObjectEventError> {
    let mut admitted = Vec::with_capacity(tracks.len());
    for blocks in tracks {
        let first = blocks
            .first()
            .ok_or(SelectedAdmObjectEventError::MissingBlockField)?;
        let object_start = timestamp_to_sample_index(&first.object_start, SAMPLE_RATE_HZ)?;
        let object_end = parse_timestamp(&first.object_duration, SAMPLE_RATE_HZ)?;
        let object_duration = object_end.sample;
        if object_start.value() != 0 {
            return Err(SelectedAdmObjectEventError::InvalidTimestamp);
        }
        if object_duration.value()
            < edit.map_or(
                source_samples.value(),
                crate::timeline::RenderTimeline::file_samples,
            )
        {
            return Err(SelectedAdmObjectEventError::ProgrammeNotCovered);
        }
        let mut timeline = Vec::with_capacity(blocks.len());
        let mut start_time = parse_timestamp(&first.block_rtime, SAMPLE_RATE_HZ)?;
        let boundaries = blocks
            .iter()
            .skip(1)
            .map(|b| parse_timestamp(&b.block_rtime, SAMPLE_RATE_HZ))
            .chain(std::iter::once(Ok(object_end)));
        // Preserve timelines already contiguous in samples. For decimal video-frame
        // times, rounding each duration separately can lose/gain a sample: accept
        // those only when the entire authored timeline meets exactly in microseconds.
        let mut sample_contiguous = start_time.sample.value() == 0;
        let mut authored_contiguous = start_time.micros == 0;
        for (block, next) in blocks.iter().zip(boundaries) {
            let next = next?;
            let length = parse_timestamp(&block.block_duration, SAMPLE_RATE_HZ)?;
            let start = start_time.sample.value();
            let duration = next
                .sample
                .value()
                .checked_sub(start)
                .filter(|&width| width != 0)
                .ok_or(SelectedAdmObjectEventError::DurationMismatch)?;
            sample_contiguous &= duration == length.sample.value();
            authored_contiguous &= start_time.micros + length.micros == next.micros;
            start_time = next;
            if !sample_contiguous && !authored_contiguous {
                return Err(SelectedAdmObjectEventError::DurationMismatch);
            }
            if block.cartesian.as_deref() != Some("1") {
                return Err(SelectedAdmObjectEventError::InvalidCartesianMode);
            }
            if !spread_values_agree([block.width, block.depth, block.height]) {
                return Err(SelectedAdmObjectEventError::SpreadMismatch);
            }
            if block.coordinates.iter().flatten().any(|x| !x.is_finite()) {
                return Err(SelectedAdmObjectEventError::NonFiniteCoordinate);
            }
            let target = SelectedObjectEvent::new(
                block.physical_host,
                object_start.value(),
                edit.map_or(
                    object_duration.value(),
                    crate::timeline::RenderTimeline::output_samples,
                ),
                source_samples.value(),
                block.coordinates[0],
                block.coordinates[1],
                block.coordinates[2],
                block.width,
                block.depth,
                block.height,
                block.diffuse.map(|v| i32::from(v != 0.0)),
            );
            timeline.push(ObjectBlock {
                start,
                ramp_samples: interpolation_samples(block, duration)?,
                target,
            });
        }
        let initial = timeline[0];
        let control = if let Some(mapping) = edit {
            sample_object_blocks(initial.target, &timeline, mapping.object_sample(0))
        } else {
            timeline.remove(0);
            initial.target
        };
        admitted.push(AdmittedSelectedObjectEvent {
            physical_host: first.physical_host,
            track_uid: first.track_uid.clone(),
            object_id: first.object_id.clone(),
            object_start,
            block_rtime: AdmSampleIndex::new(0),
            object_duration: edit
                .map_or(object_duration, |m| AdmSampleIndex::new(m.output_samples())),
            block_duration: timestamp_to_sample_index(&first.block_duration, SAMPLE_RATE_HZ)?,
            interpolation_length: AdmSampleIndex::new(initial.ramp_samples),
            event: control,
            render_timeline: edit,
            timeline: timeline.into(),
        });
    }
    Ok(AdmittedSelectedObjectEvents(admitted))
}

#[derive(Clone, Copy)]
struct AdmTimestamp {
    sample: AdmSampleIndex,
    micros: u128,
}

/// Implements F01G.4's binary64 operation sequence and returns an ADM sample index.
pub fn timestamp_to_sample_index(
    timestamp: &str,
    sample_rate: u32,
) -> Result<AdmSampleIndex, SelectedAdmObjectEventError> {
    Ok(parse_timestamp(timestamp, sample_rate)?.sample)
}

fn parse_timestamp(
    timestamp: &str,
    sample_rate: u32,
) -> Result<AdmTimestamp, SelectedAdmObjectEventError> {
    if sample_rate == 0 {
        return Err(SelectedAdmObjectEventError::TimestampOutOfRange);
    }
    let mut components = timestamp.split(':');
    let hours_text = components
        .next()
        .ok_or(SelectedAdmObjectEventError::InvalidTimestamp)?;
    let minutes_text = components
        .next()
        .ok_or(SelectedAdmObjectEventError::InvalidTimestamp)?;
    let seconds_text = components
        .next()
        .ok_or(SelectedAdmObjectEventError::InvalidTimestamp)?;
    if components.next().is_some() || !is_ascii_digits(hours_text) || minutes_text.len() != 2 {
        return Err(SelectedAdmObjectEventError::InvalidTimestamp);
    }
    let (seconds_digits, fractional_digits) = match seconds_text.split_once('.') {
        Some((seconds, fraction)) => (seconds, Some(fraction)),
        None => (seconds_text, None),
    };
    if seconds_text.matches('.').count() > 1
        || seconds_digits.len() != 2
        || !is_ascii_digits(minutes_text)
        || !is_ascii_digits(seconds_digits)
        || fractional_digits.is_some_and(|fraction| {
            fraction.is_empty() || fraction.len() > 6 || !is_ascii_digits(fraction)
        })
    {
        return Err(SelectedAdmObjectEventError::InvalidTimestamp);
    }
    let hours = hours_text
        .parse::<u64>()
        .map_err(|_| SelectedAdmObjectEventError::TimestampOutOfRange)?;
    let minutes = minutes_text
        .parse::<u64>()
        .map_err(|_| SelectedAdmObjectEventError::InvalidTimestamp)?;
    let seconds = seconds_digits
        .parse::<u64>()
        .map_err(|_| SelectedAdmObjectEventError::InvalidTimestamp)?;
    if minutes >= 60 || seconds >= 60 {
        return Err(SelectedAdmObjectEventError::InvalidTimestamp);
    }
    let fractional_microseconds = match fractional_digits {
        None => 0,
        Some(fraction) => {
            let parsed = fraction
                .parse::<u64>()
                .map_err(|_| SelectedAdmObjectEventError::InvalidTimestamp)?;
            parsed
                .checked_mul(10_u64.pow(u32::try_from(6 - fraction.len()).unwrap_or(0)))
                .ok_or(SelectedAdmObjectEventError::TimestampOutOfRange)?
        }
    };
    let whole_seconds = hours
        .checked_mul(3_600)
        .and_then(|hours_seconds| {
            minutes
                .checked_mul(60)
                .and_then(|m| hours_seconds.checked_add(m))
        })
        .ok_or(SelectedAdmObjectEventError::TimestampOutOfRange)?;

    let fraction_seconds = (fractional_microseconds as f64) / 1_000_000.0;
    let seconds_with_fraction = fraction_seconds + (seconds as f64);
    let whole_seconds_f64 = whole_seconds as f64;
    let timestamp_seconds = whole_seconds_f64 + seconds_with_fraction;
    let scaled_samples = timestamp_seconds * f64::from(sample_rate);
    let rounded_samples = scaled_samples + 0.5;
    if !rounded_samples.is_finite()
        || !(0.0..MAX_SIGNED_SAMPLE_EXCLUSIVE).contains(&rounded_samples)
    {
        return Err(SelectedAdmObjectEventError::TimestampOutOfRange);
    }
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let sample = rounded_samples.trunc() as u64;
    Ok(AdmTimestamp {
        sample: AdmSampleIndex::new(sample),
        micros: (u128::from(whole_seconds) + u128::from(seconds)) * 1_000_000
            + u128::from(fractional_microseconds),
    })
}

fn is_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit())
}

#[allow(clippy::float_cmp)]
fn spread_values_agree(values: [Option<f64>; 3]) -> bool {
    let mut values = values.into_iter().flatten();
    let Some(first) = values.next() else {
        return true;
    };
    values.all(|value| value == first)
}

fn required_attribute(
    node: Node<'_, '_>,
    name: &str,
) -> Result<String, SelectedAdmObjectEventError> {
    node.attribute(name)
        .map(str::to_owned)
        .ok_or(SelectedAdmObjectEventError::MissingAttribute)
}

fn first_direct_child<'document, 'input>(
    parent: Node<'document, 'input>,
    name: &str,
    namespace: Option<&str>,
) -> Option<Node<'document, 'input>> {
    parent.children().find(|node| {
        node.is_element()
            && node.tag_name().namespace() == namespace
            && node.tag_name().name() == name
    })
}

fn first_direct_child_text(
    parent: Node<'_, '_>,
    name: &str,
    namespace: Option<&str>,
) -> Option<String> {
    first_direct_child(parent, name, namespace)
        .and_then(|node| node.text())
        .map(str::trim)
        .map(str::to_owned)
}

// XML Schema numeric tokens collapse XML whitespace, not arbitrary Unicode spaces.
fn trim_xml_number(text: &str) -> &str {
    text.trim_matches([' ', '\t', '\r', '\n'])
}

fn optional_binary64_child(
    parent: Node<'_, '_>,
    name: &str,
    namespace: Option<&str>,
) -> Result<Option<f64>, SelectedAdmObjectEventError> {
    let Some(node) = first_direct_child(parent, name, namespace) else {
        return Ok(None);
    };
    let text = node.text().unwrap_or("");
    if text.is_empty() {
        return Ok(Some(0.0));
    }
    trim_xml_number(text)
        .parse::<f64>()
        .map(Some)
        .map_err(|_| SelectedAdmObjectEventError::MissingBlockField)
}

fn read_coordinates(
    block: Node<'_, '_>,
    namespace: Option<&str>,
) -> Result<[Option<f64>; 3], SelectedAdmObjectEventError> {
    let mut coordinates = [None; 3];
    for position in block.descendants().filter(|node| {
        node.is_element()
            && node.tag_name().namespace() == namespace
            && node.tag_name().name() == "position"
    }) {
        let name = position
            .attribute("coordinate")
            .unwrap_or("")
            .to_ascii_uppercase();
        let index = match name.as_str() {
            "X" => 0,
            "Y" => 1,
            "Z" => 2,
            _ => continue,
        };
        let text = position.text().unwrap_or("");
        let value = if text.is_empty() {
            0.0
        } else {
            trim_xml_number(text)
                .parse::<f64>()
                .map_err(|_| SelectedAdmObjectEventError::MissingBlockField)?
        };
        coordinates[index] = Some(value);
    }
    Ok(coordinates)
}

#[cfg(test)]
mod tests {
    use super::{RawAdmObjectEvent, admit_selected_object_events};
    use crate::timeline::AdmSampleCount;

    #[test]
    fn samples_cartesian_motion_at_block_boundaries_and_inside_transitions() {
        let document = roxmltree::Document::parse(r#"<audioObject start="00:00:00.00000" duration="00:00:00.09600">
            <audioBlockFormat rtime="00:00:00.00000" duration="00:00:00.03200"><cartesian>1</cartesian><position coordinate="X">-1</position><jumpPosition interpolationLength="0.000000">1</jumpPosition></audioBlockFormat>
            <audioBlockFormat rtime="00:00:00.03200" duration="00:00:00.03200"><cartesian>1</cartesian><position coordinate="X">1</position><jumpPosition interpolationLength="0.005208">1</jumpPosition></audioBlockFormat>
            <audioBlockFormat rtime="00:00:00.06400" duration="00:00:00.03200"><cartesian>1</cartesian><position coordinate="X">0</position><jumpPosition>1</jumpPosition></audioBlockFormat>
        </audioObject>"#).unwrap();
        let object = document.root_element();
        let blocks = object
            .children()
            .filter(roxmltree::Node::is_element)
            .map(|block| {
                RawAdmObjectEvent::from_xml(
                    10,
                    "track".into(),
                    "object".into(),
                    object,
                    block,
                    None,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let samples = AdmSampleCount::new(4608).unwrap();
        let mut tracks = [blocks];
        let admitted = admit_selected_object_events(&tracks, samples, None).unwrap();
        let track = &admitted.events()[0];
        for (sample, x) in [
            (1535, -1.0),
            (1536, -1.0),
            (1661, 0.0),
            (1786, 1.0),
            (3071, 1.0),
            (3072, 0.0),
        ] {
            let event = super::sample_object_blocks(track.event, &track.timeline, sample);
            assert_eq!(event.authored_coordinates()[0], Some(x));
        }
        tracks[0][1].block_rtime = "00:00:00.03100".into();
        assert_eq!(
            admit_selected_object_events(&tracks, samples, None).unwrap_err(),
            super::SelectedAdmObjectEventError::DurationMismatch
        );
    }

    #[test]
    fn admits_contiguous_fractional_blocks_without_accumulating_rounding() {
        fn timeline(xml: &str) -> Result<Vec<(u64, u64)>, super::SelectedAdmObjectEventError> {
            let document = roxmltree::Document::parse(xml).unwrap();
            let object = document.root_element();
            let blocks = object
                .children()
                .filter(roxmltree::Node::is_element)
                .map(|block| {
                    RawAdmObjectEvent::from_xml(
                        10,
                        "track".into(),
                        "object".into(),
                        object,
                        block,
                        None,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let result =
                admit_selected_object_events(&[blocks], AdmSampleCount::new(4805).unwrap(), None)?;
            let event = &result.events()[0];
            let mut blocks = vec![(0, event.block_duration().value())];
            blocks.extend(event.timeline.iter().map(|b| (b.start, b.ramp_samples)));
            Ok(blocks)
        }
        let xml = r#"<audioObject start="00:00:00" duration="00:00:00.100100">
            <audioBlockFormat rtime="00:00:00" duration="00:00:00.033367"><cartesian>1</cartesian></audioBlockFormat>
            <audioBlockFormat rtime="00:00:00.033367" duration="00:00:00.033366"><cartesian>1</cartesian></audioBlockFormat>
            <audioBlockFormat rtime="00:00:00.066733" duration="00:00:00.033367"><cartesian>1</cartesian></audioBlockFormat>
        </audioObject>"#;
        assert_eq!(
            timeline(xml).unwrap(),
            [(0, 1602), (1602, 1601), (3203, 1602)]
        );
        // Preserve an existing sample-contiguous timeline even when the lexical
        // endpoints disagree by a microsecond.
        let legacy = xml
            .replace(".033367", ".033375")
            .replace(".033366", ".033353")
            .replace(".066733", ".066729");
        assert_eq!(timeline(&legacy).unwrap(), timeline(xml).unwrap());
        // A real authored gap/overlap is not a rounding fix.
        for duration in [".033365", ".033368"] {
            assert_eq!(
                timeline(&xml.replace(".033366", duration)),
                Err(super::SelectedAdmObjectEventError::DurationMismatch)
            );
        }
    }

    #[test]
    fn admitted_event_accessors_preserve_authored_optional_control_values() {
        // Synthetic accessor recipe only; this does not exercise graph parsing or snapshots.
        let raw = RawAdmObjectEvent {
            physical_host: 12,
            track_uid: "synthetic-track".to_owned(),
            object_id: "synthetic-object".to_owned(),
            object_start: "00:00:00.000000".to_owned(),
            object_duration: "00:00:00.032021".to_owned(),
            block_rtime: "00:00:00.000000".to_owned(),
            block_duration: "00:00:00.032021".to_owned(),
            cartesian: Some("1".to_owned()),
            jump_position: Some("1".to_owned()),
            interpolation_length: Some("0".to_owned()),
            coordinates: [None, Some(-1000.0), Some(-0.0)],
            width: Some(-0.0),
            depth: None,
            height: Some(-0.0),
            diffuse: Some(0.0),
        };
        let raw_events: Vec<_> = (0..28)
            .map(|index| {
                let mut event = raw.clone();
                if index == 1 {
                    event.diffuse = None;
                }
                vec![event]
            })
            .collect();
        let admitted =
            admit_selected_object_events(&raw_events, AdmSampleCount::new(1_537).unwrap(), None)
                .unwrap();
        let event = &admitted.events()[0];
        let control = event.control_event();

        assert_eq!(event.physical_host(), 12);
        assert_eq!(control.start_sample(), 0);
        assert_eq!(control.duration_samples(), 1_537);
        assert_eq!(control.programme_samples(), 1_537);
        let coordinates = control.authored_coordinates();
        assert_eq!(coordinates[0], None);
        assert_eq!(coordinates[1], Some(-1000.0));
        assert_eq!(coordinates[2].map(f64::to_bits), Some((-0.0_f64).to_bits()));
        let spread = control.authored_spread();
        assert_eq!(spread[0].map(f64::to_bits), Some((-0.0_f64).to_bits()));
        assert_eq!(spread[1], None);
        assert_eq!(spread[2].map(f64::to_bits), Some((-0.0_f64).to_bits()));
        assert_eq!(control.diffuse(), Some(0));
        assert_eq!(admitted.events()[1].control_event().diffuse(), None);
    }
}
