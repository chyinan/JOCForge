// pattern: Functional Core
//! Pure selected-profile object-render metadata, routing, and point-target values.
//!
//! This module owns no callback state and performs no mixing, I/O, or commit.
//! Unknown metadata padding is deliberately represented as undefined rather
//! than serialized as a native record identity.

const CONTROL_PACKET_BYTES: usize = 52;
const SOURCE_RECORD_BYTES: usize = 30;
const METADATA_BYTES: usize = 60;
const CANONICAL_RECORD_WORDS: usize = 15;
const OUTPUT_CHANNELS: usize = 6;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ObjectRenderValueError {
    InvalidControlLength { actual: usize },
    FrameSampleCountOutOfRange,
    NonFiniteControlValue { slot: u8 },
    NonFiniteMetadataValue { field: MetadataFloatField },
    GainOutOfRange,
    InvalidSourceLength { actual: usize },
    MissingDirectSelector,
    ExtensionPresenceUnbound,
    UnboundSpread,
    InvalidCodedMask,
    UnsupportedRoutingProfile,
    InvalidTaskIndex,
    RoutingClassificationMismatch,
    UnsupportedGeometry,
    UnsupportedRecordType,
    UnsupportedDirectSelector,
    UnsupportedComponent,
    UnsupportedObjectMode,
    UnsupportedSourceOffset,
    RecordExtensionPresent,
    HorizontalCoordinateOutOfRange,
    VerticalCoordinateOutOfRange,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataFloatField {
    Horizontal,
    Depth,
    Vertical,
    Width,
    Gain,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderSourceKind {
    Direct,
    Object,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MetadataAdmission {
    source_index: u32,
    direct_source_count: u32,
    frame_sample_count: i32,
}

impl MetadataAdmission {
    pub fn try_new(
        source_index: u32,
        direct_source_count: u32,
        frame_sample_count: u64,
    ) -> Result<Self, ObjectRenderValueError> {
        let frame_sample_count = i32::try_from(frame_sample_count)
            .map_err(|_| ObjectRenderValueError::FrameSampleCountOutOfRange)?;
        Ok(Self {
            source_index,
            direct_source_count,
            frame_sample_count,
        })
    }

    pub const fn source_kind(self) -> RenderSourceKind {
        if self.source_index < self.direct_source_count {
            RenderSourceKind::Direct
        } else {
            RenderSourceKind::Object
        }
    }

    pub const fn source_index(self) -> u32 {
        self.source_index
    }

    pub const fn frame_sample_count(self) -> i32 {
        self.frame_sample_count
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ControlPacket {
    bytes: [u8; CONTROL_PACKET_BYTES],
}

impl ControlPacket {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, ObjectRenderValueError> {
        let bytes: [u8; CONTROL_PACKET_BYTES] =
            bytes
                .try_into()
                .map_err(|_| ObjectRenderValueError::InvalidControlLength {
                    actual: bytes.len(),
                })?;
        Ok(Self { bytes })
    }

    fn u32_slot(&self, slot: usize) -> u32 {
        let offset = slot * 4;
        u32::from_le_bytes(
            self.bytes[offset..offset + 4]
                .try_into()
                .expect("validated control packet has every four-byte slot"),
        )
    }

    fn f32_slot(&self, slot: usize) -> Result<f32, ObjectRenderValueError> {
        let value = f32::from_bits(self.u32_slot(slot));
        if value.is_finite() {
            Ok(value)
        } else {
            Err(ObjectRenderValueError::NonFiniteControlValue {
                slot: u8::try_from(slot).expect("control slot fits u8"),
            })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceMetadataValues {
    pub horizontal: f32,
    pub depth: f32,
    pub vertical: f32,
    pub vertical_enabled: u8,
    pub component: i32,
    pub flag: u8,
    pub width: f32,
    pub mode: i32,
    pub first_enumeration: i32,
    pub second_enumeration: i32,
    pub enabled: u8,
    pub sample_count: i32,
    pub extra: f32,
    pub trailing_flag: u8,
    pub gain: f32,
}

impl Default for SourceMetadataValues {
    fn default() -> Self {
        Self {
            horizontal: 0.5,
            depth: 0.5,
            vertical: 0.0,
            vertical_enabled: 1,
            component: 0,
            flag: 0,
            width: 0.0,
            mode: 1,
            first_enumeration: -1,
            second_enumeration: -1,
            enabled: 1,
            sample_count: 0,
            extra: -1.0,
            trailing_flag: 0,
            gain: 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SourceMetadata {
    source_kind: RenderSourceKind,
    values: SourceMetadataValues,
}

impl SourceMetadata {
    pub const fn from_values(source_kind: RenderSourceKind, values: SourceMetadataValues) -> Self {
        Self {
            source_kind,
            values,
        }
    }

    pub const fn source_kind(self) -> RenderSourceKind {
        self.source_kind
    }

    pub const fn values(self) -> SourceMetadataValues {
        self.values
    }

    pub fn defined_bytes(self) -> DefinedMetadataBytes {
        let mut projection = DefinedMetadataBytes::new();
        projection.set(0, &self.values.horizontal.to_le_bytes());
        projection.set(4, &self.values.depth.to_le_bytes());
        projection.set(8, &self.values.vertical.to_le_bytes());
        projection.set(12, &[self.values.vertical_enabled]);
        projection.set(16, &self.values.component.to_le_bytes());
        projection.set(20, &[self.values.flag]);
        projection.set(24, &self.values.width.to_le_bytes());
        projection.set(28, &self.values.mode.to_le_bytes());
        projection.set(32, &self.values.first_enumeration.to_le_bytes());
        projection.set(36, &self.values.second_enumeration.to_le_bytes());
        projection.set(40, &[self.values.enabled]);
        projection.set(44, &self.values.sample_count.to_le_bytes());
        projection.set(48, &self.values.extra.to_le_bytes());
        projection.set(52, &[self.values.trailing_flag]);
        projection.set(56, &self.values.gain.to_le_bytes());
        projection
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinedMetadataBytes {
    bytes: [u8; METADATA_BYTES],
    defined: [bool; METADATA_BYTES],
}

impl DefinedMetadataBytes {
    fn new() -> Self {
        Self {
            bytes: [0; METADATA_BYTES],
            defined: [false; METADATA_BYTES],
        }
    }

    fn set(&mut self, offset: usize, value: &[u8]) {
        for (index, byte) in value.iter().copied().enumerate() {
            self.bytes[offset + index] = byte;
            self.defined[offset + index] = true;
        }
    }

    pub fn byte(&self, offset: u8) -> Option<u8> {
        let offset = usize::from(offset);
        self.defined
            .get(offset)
            .copied()
            .filter(|defined| *defined)
            .map(|_| self.bytes[offset])
    }

    pub fn as_options(&self) -> [Option<u8>; METADATA_BYTES] {
        std::array::from_fn(|index| self.defined[index].then_some(self.bytes[index]))
    }

    #[cfg(test)]
    fn defined_count(&self) -> usize {
        self.defined.iter().filter(|defined| **defined).count()
    }
}

pub fn metadata_from_control(
    control: &ControlPacket,
    admission: MetadataAdmission,
) -> Result<SourceMetadata, ObjectRenderValueError> {
    let source_kind = admission.source_kind();
    let mut values = SourceMetadataValues::default();
    if source_kind == RenderSourceKind::Direct {
        values.second_enumeration = enumeration(control.u32_slot(2));
        return Ok(SourceMetadata::from_values(source_kind, values));
    }

    let horizontal = control.f32_slot(5)?;
    let depth = control.f32_slot(6)?;
    let vertical = control.f32_slot(7)?;
    let width = control.f32_slot(9)?;
    values.enabled = u8::from(control.u32_slot(0) == 0);
    values.mode = i32::from(control.u32_slot(8) == 0);
    values.sample_count = admission.frame_sample_count;
    values.component = match control.u32_slot(11) {
        1 => 48,
        2 => 12,
        3 => 3,
        4 => 60,
        5 => 51,
        _ => 0,
    };
    values.first_enumeration = match control.u32_slot(3) {
        0 => 1,
        1 => 0,
        _ => -1,
    };
    values.second_enumeration = enumeration(control.u32_slot(2));
    values.flag = u8::from(control.u32_slot(10) != 0);
    values.vertical_enabled = u8::from(control.u32_slot(12) != 0);
    values.horizontal = clip(horizontal, 0.0, 1.0);
    values.depth = clip(depth, 0.0, 1.0);
    values.vertical = clip(vertical, -1.0, 1.0);
    values.width = clip(width, 0.0, 1.0);
    Ok(SourceMetadata::from_values(source_kind, values))
}

fn enumeration(value: u32) -> i32 {
    match value {
        0..=2 => i32::try_from(value).expect("bounded enumeration fits i32"),
        _ => -1,
    }
}

fn clip(value: f32, lower: f32, upper: f32) -> f32 {
    if value < lower {
        lower
    } else if value > upper {
        upper
    } else {
        value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSourceRecord {
    source_kind: RenderSourceKind,
    bytes: [u8; SOURCE_RECORD_BYTES],
}

impl CanonicalSourceRecord {
    pub fn from_bytes(
        source_kind: RenderSourceKind,
        bytes: &[u8],
    ) -> Result<Self, ObjectRenderValueError> {
        let bytes: [u8; SOURCE_RECORD_BYTES] =
            bytes
                .try_into()
                .map_err(|_| ObjectRenderValueError::InvalidSourceLength {
                    actual: bytes.len(),
                })?;
        Ok(Self { source_kind, bytes })
    }

    pub const fn source_kind(&self) -> RenderSourceKind {
        self.source_kind
    }

    pub const fn bytes(&self) -> &[u8; SOURCE_RECORD_BYTES] {
        &self.bytes
    }
}

pub fn source_record_from_metadata(
    metadata: SourceMetadata,
) -> Result<CanonicalSourceRecord, ObjectRenderValueError> {
    let values = metadata.values;
    validate_finite(values.horizontal, MetadataFloatField::Horizontal)?;
    validate_finite(values.depth, MetadataFloatField::Depth)?;
    validate_finite(values.gain, MetadataFloatField::Gain)?;
    if !(0.0..8.0).contains(&values.gain) {
        return Err(ObjectRenderValueError::GainOutOfRange);
    }
    if metadata.source_kind == RenderSourceKind::Object {
        validate_finite(values.vertical, MetadataFloatField::Vertical)?;
        validate_finite(values.width, MetadataFloatField::Width)?;
    }

    let sentinel_word = (-1000.0_f32).to_bits();
    let sentinel =
        values.horizontal.to_bits() == sentinel_word && values.depth.to_bits() == sentinel_word;
    let mut bytes = [0_u8; SOURCE_RECORD_BYTES];
    write_i16(&mut bytes, 2, 4096);
    write_u16(&mut bytes, 4, 32767);
    if metadata.source_kind == RenderSourceKind::Object {
        write_u16(&mut bytes, 12, 256);
    }
    bytes[0] = u8::from(values.enabled == 0 || sentinel);
    let gain_code = ((f64::from(values.gain) * 4096.0).round() as i32).min(23033);
    let gain_code = i16::try_from(gain_code).map_err(|_| ObjectRenderValueError::GainOutOfRange)?;
    write_i16(&mut bytes, 2, gain_code);

    if metadata.source_kind == RenderSourceKind::Direct || sentinel {
        return Ok(CanonicalSourceRecord {
            source_kind: metadata.source_kind,
            bytes,
        });
    }

    write_i16(
        &mut bytes,
        6,
        horizontal_coordinate_code(values.horizontal)?,
    );
    write_i16(&mut bytes, 8, horizontal_coordinate_code(values.depth)?);
    write_i16(&mut bytes, 10, vertical_coordinate_code(values.vertical)?);
    let width_code = width_code(values.width)?;
    write_i16(&mut bytes, 16, width_code);
    bytes[12] = match values.component {
        3 => 3,
        12 => 2,
        48 => 1,
        51 => 5,
        60 => 4,
        _ => 0,
    };
    bytes[13] = values.vertical_enabled;
    bytes[14] = u8::from(width_code != 0);
    bytes[22] = 0;
    write_u16(&mut bytes, 24, 0);
    bytes[26] = 2;
    bytes[27] = u8::from(values.mode == 0);
    write_u16(&mut bytes, 28, 0);
    Ok(CanonicalSourceRecord {
        source_kind: metadata.source_kind,
        bytes,
    })
}

fn validate_finite(value: f32, field: MetadataFloatField) -> Result<(), ObjectRenderValueError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ObjectRenderValueError::NonFiniteMetadataValue { field })
    }
}

#[derive(Clone, Copy)]
enum CoordinateAxis {
    Horizontal,
    Vertical,
}

pub fn horizontal_coordinate_code(value: f32) -> Result<i16, ObjectRenderValueError> {
    validate_finite(value, MetadataFloatField::Horizontal)?;
    Ok(quantize_coordinate(value, CoordinateAxis::Horizontal))
}

pub fn vertical_coordinate_code(value: f32) -> Result<i16, ObjectRenderValueError> {
    validate_finite(value, MetadataFloatField::Vertical)?;
    Ok(quantize_coordinate(value, CoordinateAxis::Vertical))
}

fn quantize_coordinate(value: f32, axis: CoordinateAxis) -> i16 {
    let (extent, lower) = match axis {
        CoordinateAxis::Horizontal => (310, 0.0),
        CoordinateAxis::Vertical => (75, -75.0),
    };
    let extent_f64 = f64::from(extent);
    let level = (f64::from(value) * extent_f64)
        .round()
        .clamp(lower, extent_f64) as i32;
    if level == extent {
        return i16::MAX;
    }
    let encoded = (level * 32768) / extent;
    i16::try_from(encoded).expect("clipped coordinate code fits i16")
}

pub fn width_code(value: f32) -> Result<i16, ObjectRenderValueError> {
    validate_finite(value, MetadataFloatField::Width)?;
    Ok(quantize_width(value))
}

fn quantize_width(value: f32) -> i16 {
    let mut width = f64::from(clip(value, 0.0, 1.0));
    let base = (width * 31.0).trunc() as i32;
    let reciprocal = 1.0_f64 / 31.0;
    let mut distance = f64::INFINITY;
    for delta in [-1, 0, 1] {
        let candidate = f64::from(base + delta) * reciprocal;
        distance = distance.min((width - candidate).abs());
    }
    if distance > 0.01 {
        width = (width * 31.0).round() / 31.0;
    }
    let code = (width * 32768.0).round().clamp(0.0, 32767.0) as i32;
    i16::try_from(code).expect("clipped width code fits i16")
}

fn write_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn read_i16(bytes: &[u8], offset: usize) -> i16 {
    i16::from_le_bytes(
        bytes[offset..offset + 2]
            .try_into()
            .expect("validated source record has requested i16"),
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalRenderRecord {
    words: [i32; CANONICAL_RECORD_WORDS],
}

impl CanonicalRenderRecord {
    pub const fn from_words(words: [i32; CANONICAL_RECORD_WORDS]) -> Self {
        Self { words }
    }

    pub const fn words(&self) -> &[i32; CANONICAL_RECORD_WORDS] {
        &self.words
    }

    pub fn to_le_bytes(&self) -> [u8; CANONICAL_RECORD_WORDS * 4] {
        let mut bytes = [0; CANONICAL_RECORD_WORDS * 4];
        for (index, word) in self.words.iter().copied().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes
    }
}

pub fn canonical_record_from_source(
    source: &CanonicalSourceRecord,
    direct_selector: Option<i32>,
    extension_absent: bool,
) -> Result<CanonicalRenderRecord, ObjectRenderValueError> {
    if !extension_absent {
        return Err(ObjectRenderValueError::ExtensionPresenceUnbound);
    }
    let mut words = [0_i32; CANONICAL_RECORD_WORDS];
    words[10] = 1;
    words[11] = 1;
    words[12] = i32::from(source.bytes[0] == 0);
    words[13] = i32::from(read_i16(&source.bytes, 2));

    if source.source_kind == RenderSourceKind::Direct {
        words[0] = 1;
        words[1] = direct_selector.ok_or(ObjectRenderValueError::MissingDirectSelector)?;
        return Ok(CanonicalRenderRecord::from_words(words));
    }

    let source_offset = read_i16(&source.bytes, 28);
    if source.bytes[14] != 0 && source.bytes[27] != 1 && source_offset <= 0 {
        return Err(ObjectRenderValueError::UnboundSpread);
    }
    words[0] = 2;
    words[1] = i32::from(read_i16(&source.bytes, 6));
    words[2] = i32::from(read_i16(&source.bytes, 8));
    words[3] = if source.bytes[13] != 0 {
        i32::from(read_i16(&source.bytes, 10))
    } else {
        0
    };
    words[4] = match source.bytes[12] {
        1 => 3,
        2 => 2,
        3 => 4,
        4 => 1,
        5 => 5,
        6 => 6,
        _ => 0,
    };
    words[5] = i32::from(source.bytes[27]);
    words[6] = i32::from(source.bytes[13]);
    words[7] = i32::from(source_offset);
    Ok(CanonicalRenderRecord::from_words(words))
}

pub fn direct_selector_from_coded_mask(
    selected_bit_mask: u64,
) -> Result<i32, ObjectRenderValueError> {
    if selected_bit_mask == 0
        || selected_bit_mask > (1_u64 << 16)
        || !selected_bit_mask.is_power_of_two()
    {
        return Err(ObjectRenderValueError::InvalidCodedMask);
    }
    let mut bit = 0_i32;
    let mut remaining = selected_bit_mask;
    while remaining != 1 {
        remaining /= 2;
        bit += 1;
    }
    Ok(bit + 1)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SourceDescriptor {
    tag: i32,
    kind: i32,
}

impl SourceDescriptor {
    pub const fn new(tag: i32, kind: i32) -> Self {
        Self { tag, kind }
    }

    pub const fn tag(self) -> i32 {
        self.tag
    }

    pub const fn kind(self) -> i32 {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SelectedTaskRoutingRequest {
    pub task_index: i64,
    pub global_source_count: i64,
    pub global_direct_count: i64,
    pub worker_cpu_count: i64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaskSourceRoute {
    local_source_index: u8,
    global_source_index: u8,
    source_kind: RenderSourceKind,
    descriptor: SourceDescriptor,
}

impl TaskSourceRoute {
    pub const fn local_source_index(self) -> u8 {
        self.local_source_index
    }

    pub const fn global_source_index(self) -> u8 {
        self.global_source_index
    }

    pub const fn pcm_row(self) -> u8 {
        self.global_source_index
    }

    pub const fn control_row(self) -> u8 {
        self.global_source_index
    }

    pub const fn reset_map_index(self) -> u8 {
        self.local_source_index
    }

    pub const fn source_kind(self) -> RenderSourceKind {
        self.source_kind
    }

    pub const fn descriptor(self) -> SourceDescriptor {
        self.descriptor
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskControlCall {
    MetadataGet {
        global_source_index: u8,
    },
    SourceSet {
        local_source_index: u8,
        metadata_global_source_index: u8,
        setter_context: u32,
    },
    Process,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedTaskRouting {
    task_index: u8,
    source_begin: u8,
    source_end: u8,
    local_direct_source_count: u8,
    routes: [TaskSourceRoute; 2],
    control_calls: [TaskControlCall; 5],
}

impl SelectedTaskRouting {
    pub const fn task_index(&self) -> u8 {
        self.task_index
    }

    pub const fn source_begin(&self) -> u8 {
        self.source_begin
    }

    pub const fn source_end(&self) -> u8 {
        self.source_end
    }

    pub const fn local_direct_source_count(&self) -> u8 {
        self.local_direct_source_count
    }

    pub const fn routes(&self) -> &[TaskSourceRoute; 2] {
        &self.routes
    }

    pub const fn control_calls(&self) -> &[TaskControlCall; 5] {
        &self.control_calls
    }
}

pub fn route_selected_task(
    request: SelectedTaskRoutingRequest,
    descriptors: Option<&[SourceDescriptor]>,
) -> Result<SelectedTaskRouting, ObjectRenderValueError> {
    if request.global_source_count != 16
        || request.global_direct_count != 1
        || request.worker_cpu_count != 16
    {
        return Err(ObjectRenderValueError::UnsupportedRoutingProfile);
    }
    if !(0..=7).contains(&request.task_index) {
        return Err(ObjectRenderValueError::InvalidTaskIndex);
    }
    let task_index =
        u8::try_from(request.task_index).map_err(|_| ObjectRenderValueError::InvalidTaskIndex)?;
    let source_begin = task_index * 2;
    let source_end = source_begin + 2;
    let routes = [
        selected_source_route(source_begin, 0),
        selected_source_route(source_begin, 1),
    ];
    if let Some(descriptors) = descriptors {
        if descriptors != routes.map(|route| route.descriptor) {
            return Err(ObjectRenderValueError::RoutingClassificationMismatch);
        }
    }
    let control_calls = [
        TaskControlCall::MetadataGet {
            global_source_index: routes[0].global_source_index,
        },
        TaskControlCall::SourceSet {
            local_source_index: routes[0].local_source_index,
            metadata_global_source_index: routes[0].global_source_index,
            setter_context: 0,
        },
        TaskControlCall::MetadataGet {
            global_source_index: routes[1].global_source_index,
        },
        TaskControlCall::SourceSet {
            local_source_index: routes[1].local_source_index,
            metadata_global_source_index: routes[1].global_source_index,
            setter_context: 0,
        },
        TaskControlCall::Process,
    ];
    Ok(SelectedTaskRouting {
        task_index,
        source_begin,
        source_end,
        local_direct_source_count: u8::from(task_index == 0),
        routes,
        control_calls,
    })
}

fn selected_source_route(source_begin: u8, local_source_index: u8) -> TaskSourceRoute {
    let global_source_index = source_begin + local_source_index;
    let source_kind = if global_source_index == 0 {
        RenderSourceKind::Direct
    } else {
        RenderSourceKind::Object
    };
    let descriptor = match source_kind {
        RenderSourceKind::Direct => SourceDescriptor::new(0, 3),
        RenderSourceKind::Object => SourceDescriptor::new(2, 0),
    };
    TaskSourceRoute {
        local_source_index,
        global_source_index,
        source_kind,
        descriptor,
    }
}

const SELECTED_GEOMETRY_ROWS: [[i32; 4]; 8] = [
    [0, 0, 0, 35],
    [32767, 0, 0, 35],
    [16384, 0, 0, 35],
    [-1, -1, -1, 35],
    [0, 16384, 0, 6],
    [32767, 16384, 0, 7],
    [0, 32767, 0, 35],
    [32767, 32767, 0, 35],
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecondaryGeometry {
    rows: [[i32; 4]; 8],
}

impl SecondaryGeometry {
    pub const fn selected() -> Self {
        Self {
            rows: SELECTED_GEOMETRY_ROWS,
        }
    }

    pub fn try_from_rows(rows: [[i32; 4]; 8]) -> Result<Self, ObjectRenderValueError> {
        if rows != SELECTED_GEOMETRY_ROWS {
            return Err(ObjectRenderValueError::UnsupportedGeometry);
        }
        Ok(Self { rows })
    }

    pub const fn rows(&self) -> &[[i32; 4]; 8] {
        &self.rows
    }

    pub fn canonical_i32le_bytes(&self) -> [u8; 128] {
        let mut bytes = [0; 128];
        for (index, value) in self.rows.iter().flatten().copied().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenderTarget {
    words: [u32; OUTPUT_CHANNELS],
}

impl RenderTarget {
    pub const fn words(&self) -> &[u32; OUTPUT_CHANNELS] {
        &self.words
    }

    pub fn values(&self) -> [f32; OUTPUT_CHANNELS] {
        self.words.map(f32::from_bits)
    }
}

pub fn point_render_target(
    record: &CanonicalRenderRecord,
    geometry: &SecondaryGeometry,
) -> Result<RenderTarget, ObjectRenderValueError> {
    if geometry.rows != SELECTED_GEOMETRY_ROWS {
        return Err(ObjectRenderValueError::UnsupportedGeometry);
    }
    let words = record.words;
    if words[12] == 0 {
        return Ok(RenderTarget {
            words: [0; OUTPUT_CHANNELS],
        });
    }

    let gain = words[13] as f32 / 32768.0_f32;
    if words[0] == 1 {
        if words[1] != 4 {
            return Err(ObjectRenderValueError::UnsupportedDirectSelector);
        }
        let mut target = [0.0_f32; OUTPUT_CHANNELS];
        target[3] = gain * 8.0_f32;
        return Ok(RenderTarget {
            words: target.map(f32::to_bits),
        });
    }
    if words[0] != 2 {
        return Err(ObjectRenderValueError::UnsupportedRecordType);
    }
    if words[4] != 0 {
        return Err(ObjectRenderValueError::UnsupportedComponent);
    }
    if words[5] != 0 {
        return Err(ObjectRenderValueError::UnsupportedObjectMode);
    }
    if words[7] != 0 {
        return Err(ObjectRenderValueError::UnsupportedSourceOffset);
    }
    if words[14] != 0 {
        return Err(ObjectRenderValueError::RecordExtensionPresent);
    }
    if !(0..=32767).contains(&words[1]) || !(0..=32767).contains(&words[2]) {
        return Err(ObjectRenderValueError::HorizontalCoordinateOutOfRange);
    }
    if !(-32768..=32767).contains(&words[3]) {
        return Err(ObjectRenderValueError::VerticalCoordinateOutOfRange);
    }

    let horizontal = words[1] as f32 / 32768.0_f32;
    let depth = words[2] as f32 / 32768.0_f32;
    let raw_vertical = words[3] as f32 / 32768.0_f32;
    let vertical = if raw_vertical < 0.0 {
        0.0
    } else {
        raw_vertical
    };
    let lower = clip(depth * 1.0_f32, 0.0, 1.0);
    let upper = clip(depth * 1.0_f32, 0.0, 1.0);
    let lower_contribution = (1.0_f32 - vertical) * lower;
    let upper_contribution = upper * vertical;
    let warped_depth = lower_contribution + upper_contribution;

    let output_codes = [0_usize, 1, 2, 4, 5, 6, 7];
    let compact_horizontal = output_codes.map(|code| geometry.rows[code][0] as f32 / 32768.0);
    let compact_depth = output_codes.map(|code| geometry.rows[code][1] as f32 / 32768.0);
    let horizontal_groups: [&[usize]; 3] = [&[0, 2, 1], &[3, 4], &[5, 6]];
    let horizontal_weights =
        grouped_axis_weights(horizontal, &compact_horizontal, &horizontal_groups);
    let depth_levels = [compact_depth[0], compact_depth[3], compact_depth[5]];
    let depth_weights = axis_pair_weights(warped_depth, &depth_levels);
    let point_depth_weights = [
        depth_weights[0],
        depth_weights[0],
        depth_weights[0],
        depth_weights[1],
        depth_weights[1],
        depth_weights[2],
        depth_weights[2],
    ];

    let mut result = [0.0_f32; 8];
    for point in 0..7 {
        let coefficient = horizontal_weights[point] * point_depth_weights[point];
        result[output_codes[point]] = (gain * coefficient) * 8.0_f32;
    }
    result[4] += result[6];
    result[5] += result[7];
    Ok(RenderTarget {
        words: std::array::from_fn(|index| result[index].to_bits()),
    })
}

fn grouped_axis_weights(value: f32, coordinates: &[f32; 7], groups: &[&[usize]; 3]) -> [f32; 7] {
    let mut weights = [0.0_f32; 7];
    for group in groups {
        let first = group[0];
        let last = group[group.len() - 1];
        if value <= coordinates[first] {
            weights[first] = 1.0;
            continue;
        }
        if value >= coordinates[last] {
            weights[last] = 1.0;
            continue;
        }
        for pair in group.windows(2) {
            let left = pair[0];
            let right = pair[1];
            if coordinates[left] < value && value <= coordinates[right] {
                let numerator = value - coordinates[left];
                let denominator = coordinates[right] - coordinates[left];
                let fraction = numerator / denominator;
                let (left_weight, right_weight) = equal_power_pair(fraction);
                weights[left] = left_weight;
                weights[right] = right_weight;
                break;
            }
        }
    }
    weights
}

fn axis_pair_weights(value: f32, coordinates: &[f32; 3]) -> [f32; 3] {
    let mut weights = [0.0_f32; 3];
    if value <= coordinates[0] {
        weights[0] = 1.0;
        return weights;
    }
    if value >= coordinates[2] {
        weights[2] = 1.0;
        return weights;
    }
    for pair in 0..2 {
        if coordinates[pair] < value && value <= coordinates[pair + 1] {
            let numerator = value - coordinates[pair];
            let denominator = coordinates[pair + 1] - coordinates[pair];
            let fraction = numerator / denominator;
            let (lower, upper) = equal_power_pair(fraction);
            weights[pair] = lower;
            weights[pair + 1] = upper;
            break;
        }
    }
    weights
}

fn equal_power_pair(fraction: f32) -> (f32, f32) {
    const QUARTER_PI: f32 = f32::from_bits(0x3f49_0fdb);
    let reduced = if fraction <= 0.5 {
        fraction
    } else {
        1.0_f32 - fraction
    };
    let first_product = reduced * QUARTER_PI;
    let second_product = first_product * 0.5_f32;
    let angle = second_product * 4.0_f32;
    let cosine = f64::from(angle).cos() as f32;
    let sine = f64::from(angle).sin() as f32;
    if fraction <= 0.5 {
        (cosine, sine)
    } else {
        (sine, cosine)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn control_packet(unsigned: &[(usize, u32)], floats: &[(usize, f32)]) -> ControlPacket {
        let mut bytes = [0_u8; CONTROL_PACKET_BYTES];
        for (slot, value) in unsigned {
            bytes[slot * 4..slot * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        for (slot, value) in floats {
            bytes[slot * 4..slot * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        ControlPacket::from_bytes(&bytes).unwrap()
    }

    fn default_control() -> ControlPacket {
        control_packet(
            &[(0, 0), (2, 0), (3, 0), (8, 0), (10, 0), (11, 0), (12, 1)],
            &[(5, 0.5), (6, 0.5), (7, 0.0), (9, 0.0)],
        )
    }

    fn decode_hex<const N: usize>(text: &str) -> [u8; N] {
        assert_eq!(text.len(), N * 2);
        std::array::from_fn(|index| {
            u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap()
        })
    }

    #[test]
    fn approved_direct_and_object_inputs_materialize_without_padding_claims() {
        let direct = metadata_from_control(
            &default_control(),
            MetadataAdmission::try_new(0, 1, 1536).unwrap(),
        )
        .unwrap();
        assert_eq!(direct.source_kind(), RenderSourceKind::Direct);
        assert_eq!(direct.defined_bytes().defined_count(), 48);
        assert_eq!(direct.defined_bytes().byte(13), None);
        let direct_source = source_record_from_metadata(direct).unwrap();
        assert_eq!(
            direct_source.bytes(),
            &decode_hex::<30>("00000010ff7f000000000000000000000000000000000000000000000000")
        );
        let selector = direct_selector_from_coded_mask(8).unwrap();
        assert_eq!(selector, 4);
        assert_eq!(
            canonical_record_from_source(&direct_source, Some(selector), true)
                .unwrap()
                .to_le_bytes(),
            decode_hex::<60>(
                "0100000004000000000000000000000000000000000000000000000000000000\
                 00000000000000000100000001000000010000000010000000000000"
                    .replace(' ', "")
                    .as_str()
            )
        );

        let object = metadata_from_control(
            &default_control(),
            MetadataAdmission::try_new(1, 1, 1536).unwrap(),
        )
        .unwrap();
        assert_eq!(object.values().sample_count, 1536);
        let object_source = source_record_from_metadata(object).unwrap();
        assert_eq!(
            object_source.bytes(),
            &decode_hex::<30>("00000010ff7f004000400000000100000000000000000000000002000000")
        );
        assert_eq!(
            canonical_record_from_source(&object_source, None, true)
                .unwrap()
                .to_le_bytes(),
            decode_hex::<60>(
                "0200000000400000004000000000000000000000000000000100000000000000\
                 00000000000000000100000001000000010000000010000000000000"
                    .replace(' ', "")
                    .as_str()
            )
        );
    }

    #[test]
    fn admission_and_source_prevalidation_keep_error_causes_distinct() {
        assert_eq!(
            ControlPacket::from_bytes(&[0; 51]),
            Err(ObjectRenderValueError::InvalidControlLength { actual: 51 })
        );
        assert_eq!(
            MetadataAdmission::try_new(0, 1, i32::MAX as u64 + 1),
            Err(ObjectRenderValueError::FrameSampleCountOutOfRange)
        );
        let nonfinite = control_packet(
            &[(0, 0), (2, 0), (3, 0), (8, 0), (10, 0), (11, 0), (12, 1)],
            &[(5, f32::NAN), (6, 0.5), (7, 0.0), (9, 0.0)],
        );
        assert_eq!(
            metadata_from_control(&nonfinite, MetadataAdmission::try_new(1, 1, 1536).unwrap()),
            Err(ObjectRenderValueError::NonFiniteControlValue { slot: 5 })
        );
        assert!(
            metadata_from_control(&nonfinite, MetadataAdmission::try_new(0, 1, 1536).unwrap())
                .is_ok()
        );

        let invalid_gain = SourceMetadataValues {
            gain: 8.0,
            ..SourceMetadataValues::default()
        };
        assert_eq!(
            source_record_from_metadata(SourceMetadata::from_values(
                RenderSourceKind::Object,
                invalid_gain
            )),
            Err(ObjectRenderValueError::GainOutOfRange)
        );

        let spread = SourceMetadataValues {
            width: 0.5,
            ..SourceMetadataValues::default()
        };
        let spread = source_record_from_metadata(SourceMetadata::from_values(
            RenderSourceKind::Object,
            spread,
        ))
        .unwrap();
        assert_eq!(
            canonical_record_from_source(&spread, None, true),
            Err(ObjectRenderValueError::UnboundSpread)
        );
        assert_eq!(
            canonical_record_from_source(&spread, None, false),
            Err(ObjectRenderValueError::ExtensionPresenceUnbound)
        );
    }

    #[test]
    fn approved_quantizer_and_named_setter_cases_are_recomputed() {
        assert_eq!(horizontal_coordinate_code(-0.001_612_903_2).unwrap(), 0);
        assert_eq!(horizontal_coordinate_code(0.106_451_616).unwrap(), 3488);
        assert_eq!(horizontal_coordinate_code(1.0).unwrap(), 32767);
        assert_eq!(vertical_coordinate_code(-1.0).unwrap(), -32768);
        assert_eq!(width_code(0.967_741_9).unwrap(), 31711);
        assert_eq!(width_code(0.983_871).unwrap(), 32767);

        let values = SourceMetadataValues {
            sample_count: 1536,
            gain: 0.5,
            ..SourceMetadataValues::default()
        };
        assert_eq!(
            source_record_from_metadata(SourceMetadata::from_values(
                RenderSourceKind::Object,
                values
            ))
            .unwrap()
            .bytes(),
            &decode_hex::<30>("00000008ff7f004000400000000100000000000000000000000002000000")
        );
        let sentinel = SourceMetadataValues {
            horizontal: -1000.0,
            depth: -1000.0,
            sample_count: 1536,
            ..SourceMetadataValues::default()
        };
        assert_eq!(
            source_record_from_metadata(SourceMetadata::from_values(
                RenderSourceKind::Object,
                sentinel
            ))
            .unwrap()
            .bytes(),
            &decode_hex::<30>("01000010ff7f000000000000000100000000000000000000000000000000")
        );
    }

    #[test]
    fn selected_task_routes_cover_each_global_source_once() {
        let mut globals = Vec::new();
        for task_index in 0..8 {
            let task_index_u8 = u8::try_from(task_index).unwrap();
            let routing = route_selected_task(
                SelectedTaskRoutingRequest {
                    task_index,
                    global_source_count: 16,
                    global_direct_count: 1,
                    worker_cpu_count: 16,
                },
                None,
            )
            .unwrap();
            assert_eq!(routing.source_begin(), task_index_u8 * 2);
            assert_eq!(routing.source_end(), task_index_u8 * 2 + 2);
            assert_eq!(routing.control_calls().len(), 5);
            globals.extend(routing.routes().map(TaskSourceRoute::global_source_index));
        }
        assert_eq!(globals, (0_u8..16).collect::<Vec<_>>());

        let incorrect = [SourceDescriptor::new(2, 0), SourceDescriptor::new(2, 0)];
        assert_eq!(
            route_selected_task(
                SelectedTaskRoutingRequest {
                    task_index: 0,
                    global_source_count: 16,
                    global_direct_count: 1,
                    worker_cpu_count: 16,
                },
                Some(&incorrect),
            ),
            Err(ObjectRenderValueError::RoutingClassificationMismatch)
        );
        assert_eq!(
            route_selected_task(
                SelectedTaskRoutingRequest {
                    task_index: 8,
                    global_source_count: 16,
                    global_direct_count: 1,
                    worker_cpu_count: 8,
                },
                None,
            ),
            Err(ObjectRenderValueError::UnsupportedRoutingProfile)
        );
    }

    #[test]
    fn selected_geometry_and_all_coded_bits_are_bounded() {
        let geometry = SecondaryGeometry::selected();
        assert_eq!(geometry.rows(), &SELECTED_GEOMETRY_ROWS);
        assert_eq!(geometry.canonical_i32le_bytes().len(), 128);
        for bit in 0..=16 {
            assert_eq!(
                direct_selector_from_coded_mask(1_u64 << bit).unwrap(),
                bit + 1
            );
        }
        for invalid in [0, 3, 65537, 131072, u64::from(u32::MAX)] {
            assert_eq!(
                direct_selector_from_coded_mask(invalid),
                Err(ObjectRenderValueError::InvalidCodedMask)
            );
        }
        let mut wrong = SELECTED_GEOMETRY_ROWS;
        wrong[0][0] = 1;
        assert_eq!(
            SecondaryGeometry::try_from_rows(wrong),
            Err(ObjectRenderValueError::UnsupportedGeometry)
        );
    }

    #[test]
    fn approved_point_targets_and_branch_order_are_recomputed() {
        let geometry = SecondaryGeometry::selected();
        let cases = [
            (
                [2, 0, 0, -32768, 0, 0, 1, 0, 0, 0, 1, 1, 1, 4096, 0],
                [1065353216, 0, 0, 0, 0, 0],
            ),
            (
                [2, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 4096, 0],
                [1065353216, 0, 952700891, 0, 952700891, 832432931],
            ),
            (
                [2, 1, 16383, 16384, 0, 0, 1, 0, 0, 0, 1, 1, 1, 4096, 0],
                [952700891, 0, 840821223, 0, 1065353216, 944312685],
            ),
            (
                [1, 4, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 23033, 0],
                [0, 0, 0, 1085534720, 0, 0],
            ),
        ];
        for (input, expected) in cases {
            assert_eq!(
                point_render_target(&CanonicalRenderRecord::from_words(input), &geometry)
                    .unwrap()
                    .words(),
                &expected
            );
        }

        let inactive =
            CanonicalRenderRecord::from_words([99, -1, -1, 0, 9, 9, 0, 9, 0, 0, 1, 1, 0, 4096, 9]);
        assert_eq!(
            point_render_target(&inactive, &geometry).unwrap().words(),
            &[0; 6]
        );
        let invalid_type = CanonicalRenderRecord::from_words([
            3, 16384, 16384, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 4096, 0,
        ]);
        assert_eq!(
            point_render_target(&invalid_type, &geometry),
            Err(ObjectRenderValueError::UnsupportedRecordType)
        );
        let invalid_component = CanonicalRenderRecord::from_words([
            2, 16384, 16384, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 4096, 0,
        ]);
        assert_eq!(
            point_render_target(&invalid_component, &geometry),
            Err(ObjectRenderValueError::UnsupportedComponent)
        );
    }
}
