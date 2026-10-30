// pattern: Functional Core
//! Partial parser for the selected protected F13 EMDF container.
//!
//! This boundary validates the outer frame, selected payload order and
//! configurations, exact payload extents, terminal padding, F14K checksums,
//! selected ID11 OAMD semantics, and the washed selected ID14 coefficient grammar.
//! Its result retains producer identity but is not yet bound to the current
//! consumer transaction/source/AU lineage required by the F12 publication adapter.
//! This is the strict selected local parser; it does not emulate OpenJOC's
//! reserved-warp3 opaque branch or claim deployed-player parity.

use crate::integrity::keyless_checksum::{crc8_smbus, crc32_iso_hdlc};
use crate::joc::emdf::configuration_writer::Configuration;
use crate::joc::huffman::{
    HuffmanBitCount, HuffmanBitReader, HuffmanCodebookId, HuffmanCodebooks, HuffmanError,
    HuffmanTable,
};
use crate::joc::id14_body::Id14ProducerProvenance;
use crate::joc::oamd::{
    body_assembler::OamdBodyAssemblyError,
    body_reader::{OamdRowdataDecodeError, decode_rowdata_payload_with_bits},
    trim_section::{TrimSectionError, decode_trim_section_payload},
};
use std::fmt;

const MAX_CONTAINER_BYTES: usize = 511;
// F13.5 caps: ID11=160, ID1=10, ID2=8, ID14 FIFO=543; parser ID14 cap=372.
const MAX_OAMD_PAYLOAD_BYTES: u32 = 160;
const MAX_ID14_PAYLOAD_BYTES: u32 = 372;
const MAX_ID2_PAYLOAD_BYTES: u32 = 8;
const MAX_ID1_PAYLOAD_BYTES: u32 = 10;
const EXPECTED_PAYLOAD_IDS: [CurrentPayloadId; 4] = [
    CurrentPayloadId::Oamd,
    CurrentPayloadId::Joc,
    CurrentPayloadId::AdditionalBsi,
    CurrentPayloadId::Programme,
];

/// The carriage mode selected by the caller's fixed programme plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentEmdfCarriageMode {
    /// The selected deployed carrier configuration.
    DeployedCompat,
    /// The selected ETSI carriage configuration.
    EtsiEmdf,
}

/// A checked EMDF carrier block in the selected 0 through 5 domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarrierBlock(u8);

impl CarrierBlock {
    /// Constructs a carrier block when it is in the selected domain.
    pub const fn new(value: u8) -> Result<Self, CurrentEmdfParserError> {
        if value <= 5 {
            Ok(Self(value))
        } else {
            Err(CurrentEmdfParserError::CarrierBlockOutOfRange)
        }
    }

    /// Returns the selected carrier block value.
    pub const fn value(self) -> u8 {
        self.0
    }
}

/// A body ID admitted by this selected production route.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CurrentPayloadId {
    /// Selected OAMD payload.
    Oamd,
    /// Selected JOC payload.
    Joc,
    /// Selected additional BSI payload.
    AdditionalBsi,
    /// Selected programme payload.
    Programme,
}

impl CurrentPayloadId {
    /// Returns the wire ID for this selected body.
    pub const fn wire_value(self) -> u8 {
        match self {
            Self::Oamd => 11,
            Self::Joc => 14,
            Self::AdditionalBsi => 2,
            Self::Programme => 1,
        }
    }

    const fn from_wire(value: u8) -> Option<Self> {
        match value {
            11 => Some(Self::Oamd),
            14 => Some(Self::Joc),
            2 => Some(Self::AdditionalBsi),
            1 => Some(Self::Programme),
            _ => None,
        }
    }
}

/// Presence-preserving configuration fields parsed from one payload header.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PayloadConfiguration {
    sample_offset: Option<u16>,
    duration: Option<u32>,
    group_id: Option<u32>,
    codec_data_reserved: Option<u8>,
    discard_unknown: bool,
    frame_aligned: Option<bool>,
    create_duplicate: Option<bool>,
    remove_duplicate: Option<bool>,
    priority: Option<u8>,
    processing_allowed: Option<u8>,
}

impl PayloadConfiguration {
    /// Returns the optional sample offset without inventing an absent value.
    pub const fn sample_offset(self) -> Option<u16> {
        self.sample_offset
    }

    /// Returns the optional duration without inventing an absent value.
    pub const fn duration(self) -> Option<u32> {
        self.duration
    }

    /// Returns the optional group ID without inventing an absent value.
    pub const fn group_id(self) -> Option<u32> {
        self.group_id
    }

    /// Returns the optional codec-data reserved octet.
    pub const fn codec_data_reserved(self) -> Option<u8> {
        self.codec_data_reserved
    }

    /// Returns whether unknown payload content is discarded.
    pub const fn discard_unknown(self) -> bool {
        self.discard_unknown
    }

    /// Returns the optional frame-aligned flag.
    pub const fn frame_aligned(self) -> Option<bool> {
        self.frame_aligned
    }

    /// Returns the optional create-duplicate flag.
    pub const fn create_duplicate(self) -> Option<bool> {
        self.create_duplicate
    }

    /// Returns the optional remove-duplicate flag.
    pub const fn remove_duplicate(self) -> Option<bool> {
        self.remove_duplicate
    }

    /// Returns the optional payload priority.
    pub const fn priority(self) -> Option<u8> {
        self.priority
    }

    /// Returns the optional processing-allowed value.
    pub const fn processing_allowed(self) -> Option<u8> {
        self.processing_allowed
    }
}

/// One selected payload with its parsed config and independently bounded bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProtectedPayload {
    id: CurrentPayloadId,
    configuration: PayloadConfiguration,
    bytes: Vec<u8>,
    oamd_validation: Option<OamdValidationSummary>,
    joc_validation: Option<JocValidationSummary>,
}

impl ProtectedPayload {
    /// Returns this selected payload's wire ID.
    pub const fn id(&self) -> CurrentPayloadId {
        self.id
    }

    /// Returns the fields parsed from this payload's conditional config.
    pub const fn configuration(&self) -> PayloadConfiguration {
        self.configuration
    }

    /// Returns the exact payload octets, independent of their outer bit offset.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns bounded ID11 semantic facts when its complete selected body decoded.
    pub const fn oamd_validation(&self) -> Option<OamdValidationSummary> {
        self.oamd_validation
    }

    /// Returns ID14 facts when the complete selected coefficient body decoded.
    pub const fn joc_validation(&self) -> Option<&JocValidationSummary> {
        self.joc_validation.as_ref()
    }
}

/// ID11 facts established by full selected rowdata and optional trim-body decode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OamdValidationSummary {
    row_count: u8,
    leading_lfe: bool,
    section_count: u8,
}

impl OamdValidationSummary {
    /// Returns the number of decoded OAMD slots.
    pub const fn row_count(self) -> u8 {
        self.row_count
    }

    /// Returns whether the first decoded slot is the leading LFE.
    pub const fn leading_lfe(self) -> bool {
        self.leading_lfe
    }

    /// Returns the number of selected sections decoded.
    pub const fn section_count(self) -> u8 {
        self.section_count
    }
}

/// One selected ID14 descriptor and its point-selection fields.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JocObjectDescriptorSummary {
    band_enum: u8,
    band_count: u8,
    mode: JocCodingMode,
    steep: bool,
    point_offsets: Vec<u8>,
}

impl JocObjectDescriptorSummary {
    /// Returns the three-bit band enum.
    pub const fn band_enum(&self) -> u8 {
        self.band_enum
    }

    /// Returns the number of coefficient bands for this object.
    pub const fn band_count(&self) -> u8 {
        self.band_count
    }

    /// Returns the selected full/sparse and coarse/fine mode.
    pub const fn mode(&self) -> JocCodingMode {
        self.mode
    }

    /// Returns whether this object carries a steep-point offset.
    pub const fn steep(&self) -> bool {
        self.steep
    }

    /// Returns selected zero-based point slot offsets.
    pub fn point_offsets(&self) -> &[u8] {
        &self.point_offsets
    }
}

/// Selected F05Q coding modes with their washed table mapping.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum JocCodingMode {
    /// Full coarse coefficients.
    FullCoarse,
    /// Full fine coefficients.
    FullFine,
    /// Sparse coarse coefficients.
    SparseCoarse,
    /// Sparse fine coefficients.
    SparseFine,
}

/// Full selected ID14 header, descriptor, and coded-data validation summary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct JocValidationSummary {
    downmix_index: u8,
    object_count: u8,
    extension_config: u8,
    clip_gain_index: u8,
    sequence_count: u16,
    descriptors: Vec<JocObjectDescriptorSummary>,
    coefficient_symbol_count: u32,
    position_symbol_count: u32,
    padding_bits: u8,
}

impl JocValidationSummary {
    /// Returns the selected downmix index.
    pub const fn downmix_index(&self) -> u8 {
        self.downmix_index
    }

    /// Returns the declared ID14 object count.
    pub const fn object_count(&self) -> u8 {
        self.object_count
    }

    /// Returns the extension config field.
    pub const fn extension_config(&self) -> u8 {
        self.extension_config
    }

    /// Returns the clip-gain index field.
    pub const fn clip_gain_index(&self) -> u8 {
        self.clip_gain_index
    }

    /// Returns the sequence field without asserting a writer/source lineage join.
    pub const fn sequence_count(&self) -> u16 {
        self.sequence_count
    }

    /// Returns the fifteen selected semantic descriptor records.
    pub fn descriptors(&self) -> &[JocObjectDescriptorSummary] {
        &self.descriptors
    }

    /// Returns the number of successfully decoded coefficient symbols.
    pub const fn coefficient_symbol_count(&self) -> u32 {
        self.coefficient_symbol_count
    }

    /// Returns the number of successfully decoded sparse position symbols.
    pub const fn position_symbol_count(&self) -> u32 {
        self.position_symbol_count
    }

    /// Returns the number of verified zero byte-padding bits.
    pub const fn padding_bits(&self) -> u8 {
        self.padding_bits
    }
}

/// Parsed selected EMDF, with producer identity preserved for a later consumer join.
///
/// The owned bytes, selected mode/carrier, and producer provenance stay bound
/// together. This establishes neither the current consumer transaction nor the
/// source/AU queue lineage required by the separate F12 publication adapter.
/// ID1/ID2 configs are compared with their selected writer constructors, while
/// their body semantics and actual producer/pop receipt binding remain the F13
/// wrapper's responsibility. The producer transaction token may be absent; the
/// wrapper must reject that case before minting any F12 publication capability,
/// and must keep its consumer token separate. This type exposes no F12 capability
/// or complexity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParsedCurrentEmdfContainer {
    bytes: Vec<u8>,
    mode: CurrentEmdfCarriageMode,
    carrier_block: CarrierBlock,
    id14_provenance: Id14ProducerProvenance,
    payloads: Vec<ProtectedPayload>,
}

impl ParsedCurrentEmdfContainer {
    /// Returns the exact protected container bytes supplied to the parser.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the selected programme carriage mode used during validation.
    pub const fn mode(&self) -> CurrentEmdfCarriageMode {
        self.mode
    }

    /// Returns the selected carrier block used during validation.
    pub const fn carrier_block(&self) -> CarrierBlock {
        self.carrier_block
    }

    /// Returns the selected ID14 producer identity retained from the popped source.
    pub const fn id14_provenance(&self) -> &Id14ProducerProvenance {
        &self.id14_provenance
    }

    /// Returns the four selected payloads in their required production order.
    pub fn payloads(&self) -> &[ProtectedPayload] {
        &self.payloads
    }
}

/// Exact local parser outcomes for the selected framing/configuration boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CurrentEmdfParserError {
    /// Carrier block is outside the selected 0 through 5 range.
    CarrierBlockOutOfRange,
    /// The supplied container exceeds the selected 511-byte bound.
    ContainerTooLarge,
    /// Fewer than four bytes were supplied for the outer header.
    TruncatedHeader,
    /// The two-byte selected sync word does not match.
    InvalidSyncWord,
    /// Declared body length does not consume the complete supplied container.
    OuterLengthMismatch,
    /// A selected bit field ends before all of its bits are available.
    TruncatedBitstream,
    /// Container version is not the selected version zero.
    UnsupportedVersion,
    /// Container key ID is not the selected key ID zero.
    UnsupportedKeyId,
    /// The next production payload ID differs from the required ID.
    PayloadOrder { expected: u8, actual: u8 },
    /// The production route terminates before a required selected payload.
    MissingPayload(u8),
    /// A selected payload appears more than once.
    DuplicatePayload(u8),
    /// A payload outside the selected production set appears in the container.
    UnboundPayload(u8),
    /// A selected varuint exceeds its maximum group count.
    VaruintTooManyGroups,
    /// A selected varuint arithmetic operation overflows its bounded value.
    VaruintOverflow,
    /// The current selected production route requires carrier block two.
    UnsupportedCarrierBlock,
    /// Sample-offset extension bit is nonzero.
    SampleOffsetExtension,
    /// Codec-data reserved octet is nonzero.
    CodecDataReserved,
    /// ID11/ID14 config is inconsistent with the selected fixed mode.
    SelectedConfiguration(u8),
    /// A payload exceeds its selected same-container producer/parser limit.
    PayloadCapacityExceeded {
        payload_id: u8,
        actual_bytes: u32,
        maximum_bytes: u16,
    },
    /// Primary or secondary protection length code is not selected.
    ProtectionLengthCode,
    /// More than seven terminal padding bits remain.
    ExcessTerminalPadding,
    /// A terminal padding bit is nonzero.
    NonzeroTerminalPadding,
    /// The F14K CRC32/ISO-HDLC value does not match.
    PrimaryChecksumMismatch,
    /// The F14K CRC8/SMBUS value does not match.
    SecondaryChecksumMismatch,
    /// ID11 body section count is outside the selected one or two sections.
    OamdSectionCount(u8),
    /// The selected OAMD outer or rowdata grammar was rejected.
    OamdBodyAssembly(OamdBodyAssemblyError),
    /// The selected OAMD trim grammar was rejected.
    OamdTrim(TrimSectionError),
    /// ID11 did not decode exactly 16 slots with a leading LFE row.
    OamdSelectedSlots,
    /// ID14 payload exceeds the selected 372-byte capacity.
    JocBodyCapacity,
    /// ID14 fixed header does not match selected downmix/count/extension.
    JocSelectedHeader,
    /// The encoded sequence does not match the popped producer receipt.
    JocSequenceMismatch { expected: u16, actual: u16 },
    /// Parsed ID14 body bytes do not match the producer receipt identity.
    JocProducerBodyMismatch,
    /// ID14 selected production does not mark every object descriptor present.
    JocDescriptorAbsent(u8),
    /// ID14 selected band enum is outside the retry-reachable 0..=5 range.
    JocBandEnum(u8),
    /// The selected producer supports one point, not the generic two-point form.
    JocTwoPointProfile(u8),
    /// A steep point offset is outside the selected 0..=23 slot domain.
    JocPointOffset(u8),
    /// ID14 selected channel index is outside 0..=4.
    JocChannelIndex(u8),
    /// The selected codebook table or bounded Huffman body is invalid.
    Huffman(HuffmanError),
    /// More than seven ID14 final padding bits remain.
    JocExcessPadding,
    /// An ID14 final padding bit is nonzero.
    JocNonzeroPadding,
}

impl CurrentEmdfParserError {
    /// Returns a stable machine-readable local outcome label.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::CarrierBlockOutOfRange => "EMDF_CARRIER_BLOCK_OUT_OF_RANGE",
            Self::ContainerTooLarge => "EMDF_CONTAINER_TOO_LARGE",
            Self::TruncatedHeader => "EMDF_TRUNCATED_HEADER",
            Self::InvalidSyncWord => "EMDF_INVALID_SYNC_WORD",
            Self::OuterLengthMismatch => "EMDF_OUTER_LENGTH_MISMATCH",
            Self::TruncatedBitstream => "EMDF_TRUNCATED_BITSTREAM",
            Self::UnsupportedVersion => "EMDF_UNSUPPORTED_VERSION",
            Self::UnsupportedKeyId => "EMDF_UNSUPPORTED_KEY_ID",
            Self::PayloadOrder { .. } => "EMDF_PAYLOAD_ORDER",
            Self::MissingPayload(_) => "EMDF_MISSING_PAYLOAD",
            Self::DuplicatePayload(_) => "EMDF_DUPLICATE_PAYLOAD",
            Self::UnboundPayload(_) => "EMDF_UNBOUND_PAYLOAD",
            Self::VaruintTooManyGroups => "EMDF_VARUINT_TOO_MANY_GROUPS",
            Self::VaruintOverflow => "EMDF_VARUINT_OVERFLOW",
            Self::UnsupportedCarrierBlock => "EMDF_UNSUPPORTED_CARRIER_BLOCK",
            Self::SampleOffsetExtension => "EMDF_SAMPLE_OFFSET_EXTENSION",
            Self::CodecDataReserved => "EMDF_CODEC_DATA_RESERVED",
            Self::SelectedConfiguration(_) => "EMDF_SELECTED_CONFIGURATION",
            Self::PayloadCapacityExceeded { .. } => "EMDF_PAYLOAD_CAPACITY_EXCEEDED",
            Self::ProtectionLengthCode => "EMDF_PROTECTION_LENGTH_CODE",
            Self::ExcessTerminalPadding => "EMDF_EXCESS_TERMINAL_PADDING",
            Self::NonzeroTerminalPadding => "EMDF_NONZERO_TERMINAL_PADDING",
            Self::PrimaryChecksumMismatch => "EMDF_PRIMARY_CHECKSUM_MISMATCH",
            Self::SecondaryChecksumMismatch => "EMDF_SECONDARY_CHECKSUM_MISMATCH",
            Self::OamdSectionCount(_) => "EMDF_OAMD_SECTION_COUNT",
            Self::OamdBodyAssembly(_) => "EMDF_OAMD_BODY_ASSEMBLY",
            Self::OamdTrim(_) => "EMDF_OAMD_TRIM",
            Self::OamdSelectedSlots => "EMDF_OAMD_SELECTED_SLOTS",
            Self::JocBodyCapacity => "EMDF_JOC_BODY_CAPACITY",
            Self::JocSelectedHeader => "EMDF_JOC_SELECTED_HEADER",
            Self::JocSequenceMismatch { .. } => "EMDF_JOC_SEQUENCE_MISMATCH",
            Self::JocProducerBodyMismatch => "EMDF_JOC_PRODUCER_BODY_MISMATCH",
            Self::JocDescriptorAbsent(_) => "EMDF_JOC_DESCRIPTOR_ABSENT",
            Self::JocBandEnum(_) => "EMDF_JOC_BAND_ENUM",
            Self::JocTwoPointProfile(_) => "EMDF_JOC_TWO_POINT_PROFILE",
            Self::JocPointOffset(_) => "EMDF_JOC_POINT_OFFSET",
            Self::JocChannelIndex(_) => "EMDF_JOC_CHANNEL_INDEX",
            Self::Huffman(error) => error.code(),
            Self::JocExcessPadding => "EMDF_JOC_EXCESS_PADDING",
            Self::JocNonzeroPadding => "EMDF_JOC_NONZERO_PADDING",
        }
    }
}

impl fmt::Display for CurrentEmdfParserError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for CurrentEmdfParserError {}

const fn selected_xy_table() -> [u32; 63] {
    let mut values = [0_u32; 63];
    let mut index = 0_usize;
    while index < values.len() {
        let numerator = 32_768_u32 * index as u32 + 31;
        let value = numerator / 62;
        values[index] = if value > 32_767 { 32_767 } else { value };
        index += 1;
    }
    values
}

const fn selected_z_table() -> [u32; 16] {
    let mut values = [0_u32; 16];
    let mut index = 0_usize;
    while index < values.len() {
        let numerator = 32_768_u32 * index as u32 + 7;
        let value = numerator / 15;
        values[index] = if value > 32_767 { 32_767 } else { value };
        index += 1;
    }
    values
}

const SELECTED_XY_TABLE: [u32; 63] = selected_xy_table();
const SELECTED_Z_TABLE: [u32; 16] = selected_z_table();

fn validate_outer_header(bytes: &[u8]) -> Result<&[u8], CurrentEmdfParserError> {
    if bytes.len() > MAX_CONTAINER_BYTES {
        return Err(CurrentEmdfParserError::ContainerTooLarge);
    }
    if bytes.len() < 4 {
        return Err(CurrentEmdfParserError::TruncatedHeader);
    }
    if bytes[..2] != [0x58, 0x38] {
        return Err(CurrentEmdfParserError::InvalidSyncWord);
    }
    let body_length = usize::from(u16::from_be_bytes([bytes[2], bytes[3]]));
    let total_length = 4_usize
        .checked_add(body_length)
        .ok_or(CurrentEmdfParserError::OuterLengthMismatch)?;
    if total_length != bytes.len() {
        return Err(CurrentEmdfParserError::OuterLengthMismatch);
    }
    Ok(&bytes[4..])
}

fn validate_oamd_body(bytes: &[u8]) -> Result<OamdValidationSummary, CurrentEmdfParserError> {
    let mut header = BitReader::new(bytes);
    let version = header.read_bits(2)?;
    let count_code = header.read_bits(5)?;
    let row_count = if count_code == 31 {
        32_usize
            .checked_add(
                usize::try_from(header.read_bits(7)?)
                    .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?,
            )
            .ok_or(CurrentEmdfParserError::TruncatedBitstream)?
    } else {
        usize::try_from(count_code + 1).map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?
    };
    let shortcut = header.read_bits(1)? != 0;
    let leading_lfe = header.read_bits(1)? != 0;
    let protected_sections = header.read_bits(1)? != 0;
    let section_count = u8::try_from(header.read_bits(4)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    if version != 0 || !shortcut || protected_sections {
        return Err(CurrentEmdfParserError::OamdBodyAssembly(
            OamdBodyAssemblyError::RowDomain,
        ));
    }
    if row_count != 16 || !leading_lfe {
        return Err(CurrentEmdfParserError::OamdSelectedSlots);
    }

    if !(1..=2).contains(&section_count) {
        return Err(CurrentEmdfParserError::OamdSectionCount(section_count));
    }
    let mut rowdata_payload = None;
    let mut trim_payload = None;
    for section_index in 0..section_count {
        let kind = u8::try_from(header.read_bits(4)?)
            .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
        match (section_index, kind) {
            (0, 1) | (1, 2) => {}
            (0, 2) => {
                return Err(CurrentEmdfParserError::OamdBodyAssembly(
                    OamdBodyAssemblyError::SectionMissing,
                ));
            }
            (1, 1) => {
                return Err(CurrentEmdfParserError::OamdBodyAssembly(
                    OamdBodyAssemblyError::DuplicateSection,
                ));
            }
            _ => {
                return Err(CurrentEmdfParserError::OamdBodyAssembly(
                    OamdBodyAssemblyError::SectionKind,
                ));
            }
        }
        let escaped_length = read_escaped_section_length(&mut header)
            .map_err(CurrentEmdfParserError::OamdBodyAssembly)?;
        let payload_length =
            escaped_length
                .checked_add(1)
                .ok_or(CurrentEmdfParserError::OamdBodyAssembly(
                    OamdBodyAssemblyError::EscapedOverflow,
                ))?;
        let payload = header.read_octets(payload_length).map_err(|_| {
            CurrentEmdfParserError::OamdBodyAssembly(OamdBodyAssemblyError::Truncated)
        })?;
        match kind {
            1 => rowdata_payload = Some(payload),
            2 => trim_payload = Some(payload),
            _ => {
                return Err(CurrentEmdfParserError::OamdBodyAssembly(
                    OamdBodyAssemblyError::SectionKind,
                ));
            }
        }
    }
    let Some(rowdata_payload) = rowdata_payload else {
        return Err(CurrentEmdfParserError::OamdBodyAssembly(
            OamdBodyAssemblyError::SectionMissing,
        ));
    };
    let padding_bits = header.remaining();
    if padding_bits > 7 {
        return Err(CurrentEmdfParserError::OamdBodyAssembly(
            OamdBodyAssemblyError::UnconsumedFields,
        ));
    }
    while header.remaining() > 0 {
        if header.read_bits(1)? != 0 {
            return Err(CurrentEmdfParserError::OamdBodyAssembly(
                OamdBodyAssemblyError::NonzeroPadding,
            ));
        }
    }

    let (_, rows, _) = decode_rowdata_payload_with_bits(
        &rowdata_payload,
        row_count,
        leading_lfe,
        &SELECTED_XY_TABLE,
        &SELECTED_Z_TABLE,
    )
    .map_err(|error| CurrentEmdfParserError::OamdBodyAssembly(map_oamd_rowdata_error(error)))?;
    if rows.len() != 16 {
        return Err(CurrentEmdfParserError::OamdSelectedSlots);
    }
    if let Some(payload) = trim_payload {
        let row_count =
            u32::try_from(row_count).map_err(|_| CurrentEmdfParserError::OamdSelectedSlots)?;
        decode_trim_section_payload(&payload, row_count)
            .map_err(CurrentEmdfParserError::OamdTrim)?;
    }
    Ok(OamdValidationSummary {
        row_count: 16,
        leading_lfe,
        section_count,
    })
}

fn read_escaped_section_length(reader: &mut BitReader<'_>) -> Result<u32, OamdBodyAssemblyError> {
    let mut value = 0_u32;
    for group_index in 0..4 {
        let digit = u32::try_from(
            reader
                .read_bits(4)
                .map_err(|_| OamdBodyAssemblyError::Truncated)?,
        )
        .map_err(|_| OamdBodyAssemblyError::EscapedOverflow)?;
        let continues = reader
            .read_bits(1)
            .map_err(|_| OamdBodyAssemblyError::Truncated)?
            != 0;
        value = if group_index == 0 {
            digit
        } else {
            value
                .checked_add(1)
                .and_then(|prior| prior.checked_mul(16))
                .and_then(|prior| prior.checked_add(digit))
                .ok_or(OamdBodyAssemblyError::EscapedOverflow)?
        };
        if !continues {
            return Ok(value);
        }
    }
    Err(OamdBodyAssemblyError::EscapedUnterminated)
}

fn map_oamd_rowdata_error(error: OamdRowdataDecodeError) -> OamdBodyAssemblyError {
    match error {
        OamdRowdataDecodeError::SectionControl => OamdBodyAssemblyError::SectionControl,
        OamdRowdataDecodeError::Truncated => OamdBodyAssemblyError::Truncated,
        OamdRowdataDecodeError::UnconsumedFields => OamdBodyAssemblyError::UnconsumedFields,
        OamdRowdataDecodeError::NonzeroPadding => OamdBodyAssemblyError::NonzeroPadding,
        OamdRowdataDecodeError::TimingBase => OamdBodyAssemblyError::TimingBase,
        OamdRowdataDecodeError::TimingDomain => OamdBodyAssemblyError::TimingDomain,
        OamdRowdataDecodeError::TimingDuration => OamdBodyAssemblyError::TimingDuration,
        OamdRowdataDecodeError::TimingReserved => OamdBodyAssemblyError::TimingReserved,
        OamdRowdataDecodeError::TimingMultiframe => OamdBodyAssemblyError::TimingMultiframe,
        OamdRowdataDecodeError::RowDomain => OamdBodyAssemblyError::RowDomain,
        OamdRowdataDecodeError::RowXyz => OamdBodyAssemblyError::RowXyz,
        OamdRowdataDecodeError::RowGain => OamdBodyAssemblyError::RowGain,
        OamdRowdataDecodeError::RowSpread => OamdBodyAssemblyError::RowSpread,
        OamdRowdataDecodeError::RowExtension => OamdBodyAssemblyError::RowExtension,
        OamdRowdataDecodeError::EscapedOverflow => OamdBodyAssemblyError::EscapedOverflow,
    }
}

fn parse_joc_payload(
    bytes: &[u8],
    provenance: &Id14ProducerProvenance,
) -> Result<JocValidationSummary, CurrentEmdfParserError> {
    const BAND_COUNTS: [u8; 6] = [1, 3, 5, 7, 9, 12];

    check_joc_payload_capacity(bytes)?;
    if !provenance.matches_body(provenance.producer_generation(), bytes) {
        return Err(CurrentEmdfParserError::JocProducerBodyMismatch);
    }
    if provenance.object_count().value() != 15 {
        return Err(CurrentEmdfParserError::JocSelectedHeader);
    }

    let mut reader = BitReader::new(bytes);
    let downmix_index = u8::try_from(reader.read_bits(3)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let object_count = u8::try_from(reader.read_bits(6)? + 1)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let extension_config = u8::try_from(reader.read_bits(3)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let clip_gain_index = u8::try_from(reader.read_bits(8)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let sequence_count = u16::try_from(reader.read_bits(10)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    if downmix_index != 3
        || object_count != 15
        || extension_config != 0
        || !(128..=255).contains(&clip_gain_index)
    {
        return Err(CurrentEmdfParserError::JocSelectedHeader);
    }
    if sequence_count != provenance.sequence_count() {
        return Err(CurrentEmdfParserError::JocSequenceMismatch {
            expected: provenance.sequence_count(),
            actual: sequence_count,
        });
    }

    let mut descriptors = Vec::with_capacity(usize::from(object_count));
    for descriptor_index in 0..object_count {
        if reader.read_bits(1)? == 0 {
            return Err(CurrentEmdfParserError::JocDescriptorAbsent(
                descriptor_index,
            ));
        }
        let band_enum = u8::try_from(reader.read_bits(3)?)
            .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
        let band_count = BAND_COUNTS
            .get(usize::from(band_enum))
            .copied()
            .ok_or(CurrentEmdfParserError::JocBandEnum(descriptor_index))?;
        let mode = match reader.read_bits(2)? {
            0 => JocCodingMode::FullCoarse,
            1 => JocCodingMode::FullFine,
            2 => JocCodingMode::SparseCoarse,
            3 => JocCodingMode::SparseFine,
            _ => return Err(CurrentEmdfParserError::TruncatedBitstream),
        };
        let steep = reader.read_bits(1)? != 0;
        let point_count_minus_one = reader.read_bits(1)?;
        if point_count_minus_one != 0 {
            return Err(CurrentEmdfParserError::JocTwoPointProfile(descriptor_index));
        }
        let mut point_offsets = Vec::new();
        if steep {
            let offset = u8::try_from(reader.read_bits(5)?)
                .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
            if offset >= 24 {
                return Err(CurrentEmdfParserError::JocPointOffset(descriptor_index));
            }
            point_offsets.push(offset);
        }
        descriptors.push(JocObjectDescriptorSummary {
            band_enum,
            band_count,
            mode,
            steep,
            point_offsets,
        });
    }

    let codebooks = HuffmanCodebooks::selected().map_err(CurrentEmdfParserError::Huffman)?;
    let mut coefficient_symbol_count = 0_u32;
    let mut position_symbol_count = 0_u32;
    for (descriptor_index, descriptor) in descriptors.iter().enumerate() {
        let descriptor_index = u8::try_from(descriptor_index)
            .map_err(|_| CurrentEmdfParserError::JocSelectedHeader)?;
        let (coefficient_table_id, symbol_modulus, initial_previous) = match descriptor.mode {
            JocCodingMode::FullCoarse => (HuffmanCodebookId::FullCoarse, 96_u16, 48_u16),
            JocCodingMode::FullFine => (HuffmanCodebookId::FullFine, 192_u16, 96_u16),
            JocCodingMode::SparseCoarse => (HuffmanCodebookId::SparseCoarse, 96_u16, 50_u16),
            JocCodingMode::SparseFine => (HuffmanCodebookId::SparseFine, 192_u16, 100_u16),
        };
        let coefficient_table = codebooks.table(coefficient_table_id);
        if matches!(
            descriptor.mode,
            JocCodingMode::FullCoarse | JocCodingMode::FullFine
        ) {
            for _channel in 0..5 {
                let mut previous = initial_previous;
                for _band in 0..descriptor.band_count {
                    let symbol = read_huffman_symbol(&mut reader, coefficient_table)?;
                    previous = (previous + symbol) % symbol_modulus;
                    coefficient_symbol_count = coefficient_symbol_count
                        .checked_add(1)
                        .ok_or(CurrentEmdfParserError::VaruintOverflow)?;
                }
            }
        } else {
            let mut previous_channel = u8::try_from(reader.read_bits(3)?)
                .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
            if previous_channel > 4 {
                return Err(CurrentEmdfParserError::JocChannelIndex(descriptor_index));
            }
            let position_table = codebooks.table(HuffmanCodebookId::Position5);
            for _band in 1..descriptor.band_count {
                let position_symbol = read_huffman_symbol(&mut reader, position_table)?;
                previous_channel =
                    u8::try_from((u16::from(previous_channel) + position_symbol) % 5)
                        .map_err(|_| CurrentEmdfParserError::JocChannelIndex(descriptor_index))?;
                position_symbol_count = position_symbol_count
                    .checked_add(1)
                    .ok_or(CurrentEmdfParserError::VaruintOverflow)?;
            }
            let mut previous = initial_previous;
            for _band in 0..descriptor.band_count {
                let symbol = read_huffman_symbol(&mut reader, coefficient_table)?;
                previous = (previous + symbol) % symbol_modulus;
                coefficient_symbol_count = coefficient_symbol_count
                    .checked_add(1)
                    .ok_or(CurrentEmdfParserError::VaruintOverflow)?;
            }
        }
    }

    let padding_bits = reader.remaining();
    if padding_bits > 7 {
        return Err(CurrentEmdfParserError::JocExcessPadding);
    }
    while reader.remaining() > 0 {
        if reader.read_bits(1)? != 0 {
            return Err(CurrentEmdfParserError::JocNonzeroPadding);
        }
    }
    Ok(JocValidationSummary {
        downmix_index,
        object_count,
        extension_config,
        clip_gain_index,
        sequence_count,
        descriptors,
        coefficient_symbol_count,
        position_symbol_count,
        padding_bits: u8::try_from(padding_bits)
            .map_err(|_| CurrentEmdfParserError::JocExcessPadding)?,
    })
}

fn check_joc_payload_capacity(bytes: &[u8]) -> Result<(), CurrentEmdfParserError> {
    if bytes.len()
        > usize::try_from(MAX_ID14_PAYLOAD_BYTES)
            .map_err(|_| CurrentEmdfParserError::VaruintOverflow)?
    {
        Err(CurrentEmdfParserError::JocBodyCapacity)
    } else {
        Ok(())
    }
}

fn read_huffman_symbol(
    reader: &mut BitReader<'_>,
    table: &HuffmanTable,
) -> Result<u16, CurrentEmdfParserError> {
    let available_bits = reader.remaining();
    let packed_bits = reader.remaining_packed_bits();
    let valid_bits =
        u64::try_from(available_bits).map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let mut huffman_reader = HuffmanBitReader::new(&packed_bits, HuffmanBitCount::new(valid_bits));
    let symbol = huffman_reader
        .read_symbol(table)
        .map_err(CurrentEmdfParserError::Huffman)?;
    let consumed_bits = usize::try_from(huffman_reader.bit_cursor().value())
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    reader.skip_bits(consumed_bits)?;
    Ok(symbol)
}

/// Parses the selected route through framing, config, F14K, ID11, and ID14.
///
/// The result checks ID1/ID2 configurations against the selected typed producer
/// constructors, enforces all selected payload capacities, validates ID11 with
/// the approved integer-generated PG06 tables, and checks ID14 identity/sequence
/// against its producer receipt. ID1/ID2 bodies remain bounded opaque bytes whose
/// typed producer/pop receipts must be bound by the F13 wrapper. The wrapper must
/// reject a missing producer transaction token and bind its current consumer
/// transaction separately. This parser does not establish current consumer
/// transaction or source/AU queue lineage and is not an F12 publication capability.
pub fn parse_current_emdf_container(
    bytes: &[u8],
    expected_mode: CurrentEmdfCarriageMode,
    carrier_block: CarrierBlock,
    id14_provenance: &Id14ProducerProvenance,
) -> Result<ParsedCurrentEmdfContainer, CurrentEmdfParserError> {
    if carrier_block.value() != 2 {
        return Err(CurrentEmdfParserError::UnsupportedCarrierBlock);
    }
    let body_bytes = validate_outer_header(bytes)?;
    let mut reader = BitReader::new(body_bytes);
    if reader.read_bits(2)? != 0 {
        return Err(CurrentEmdfParserError::UnsupportedVersion);
    }
    if reader.read_bits(3)? != 0 {
        return Err(CurrentEmdfParserError::UnsupportedKeyId);
    }

    let mut parsed_payloads = Vec::with_capacity(EXPECTED_PAYLOAD_IDS.len());
    let mut seen_ids = Vec::with_capacity(EXPECTED_PAYLOAD_IDS.len());
    let mut payload_index = 0;
    loop {
        let payload_wire_id = u8::try_from(reader.read_bits(5)?)
            .map_err(|_| CurrentEmdfParserError::UnboundPayload(u8::MAX))?;
        if payload_wire_id == 0 {
            if let Some(expected) = EXPECTED_PAYLOAD_IDS.get(payload_index) {
                return Err(CurrentEmdfParserError::MissingPayload(
                    expected.wire_value(),
                ));
            }
            break;
        }
        if seen_ids.contains(&payload_wire_id) {
            return Err(CurrentEmdfParserError::DuplicatePayload(payload_wire_id));
        }
        let Some(expected) = EXPECTED_PAYLOAD_IDS.get(payload_index).copied() else {
            if CurrentPayloadId::from_wire(payload_wire_id).is_some() {
                return Err(CurrentEmdfParserError::DuplicatePayload(payload_wire_id));
            }
            return Err(CurrentEmdfParserError::UnboundPayload(payload_wire_id));
        };
        if payload_wire_id != expected.wire_value() {
            if CurrentPayloadId::from_wire(payload_wire_id).is_none() {
                return Err(CurrentEmdfParserError::UnboundPayload(payload_wire_id));
            }
            return Err(CurrentEmdfParserError::PayloadOrder {
                expected: expected.wire_value(),
                actual: payload_wire_id,
            });
        }

        let configuration = read_configuration(&mut reader)?;
        validate_selected_configuration(expected, configuration, expected_mode)?;
        let payload_length = read_varuint(&mut reader, 8, 2)?;
        let maximum_payload_bytes = selected_payload_capacity(expected);
        if payload_length > maximum_payload_bytes {
            return Err(CurrentEmdfParserError::PayloadCapacityExceeded {
                payload_id: expected.wire_value(),
                actual_bytes: payload_length,
                maximum_bytes: u16::try_from(maximum_payload_bytes)
                    .map_err(|_| CurrentEmdfParserError::VaruintOverflow)?,
            });
        }
        let payload_bytes = reader.read_octets(payload_length)?;
        parsed_payloads.push(ProtectedPayload {
            id: expected,
            configuration,
            bytes: payload_bytes,
            oamd_validation: None,
            joc_validation: None,
        });
        seen_ids.push(payload_wire_id);
        payload_index += 1;
    }

    let primary_length_code = reader.read_bits(2)?;
    let secondary_length_code = reader.read_bits(2)?;
    if primary_length_code != 2 || secondary_length_code != 1 {
        return Err(CurrentEmdfParserError::ProtectionLengthCode);
    }
    let primary_start = reader.position();
    let primary_checksum = u32::try_from(reader.read_bits(32)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let secondary_start = reader.position();
    let secondary_checksum = u8::try_from(reader.read_bits(8)?)
        .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
    let checksum_tail = reader.remaining();
    if checksum_tail > 7 {
        return Err(CurrentEmdfParserError::ExcessTerminalPadding);
    }
    while reader.remaining() > 0 {
        if reader.read_bits(1)? != 0 {
            return Err(CurrentEmdfParserError::NonzeroTerminalPadding);
        }
    }

    let mut crc_body = body_bytes.to_vec();
    clear_bit_range(&mut crc_body, primary_start, 32)?;
    clear_bit_range(&mut crc_body, secondary_start, 8)?;
    if crc32_iso_hdlc(&crc_body) != primary_checksum {
        return Err(CurrentEmdfParserError::PrimaryChecksumMismatch);
    }
    if crc8_smbus(&primary_checksum.to_be_bytes()) != secondary_checksum {
        return Err(CurrentEmdfParserError::SecondaryChecksumMismatch);
    }

    for payload in &mut parsed_payloads {
        match payload.id {
            CurrentPayloadId::Oamd => {
                payload.oamd_validation = Some(validate_oamd_body(&payload.bytes)?);
            }
            CurrentPayloadId::Joc => {
                payload.joc_validation = Some(parse_joc_payload(&payload.bytes, id14_provenance)?);
            }
            CurrentPayloadId::AdditionalBsi | CurrentPayloadId::Programme => {}
        }
    }

    Ok(ParsedCurrentEmdfContainer {
        bytes: bytes.to_vec(),
        mode: expected_mode,
        carrier_block,
        id14_provenance: id14_provenance.clone(),
        payloads: parsed_payloads,
    })
}

fn read_configuration(
    reader: &mut BitReader<'_>,
) -> Result<PayloadConfiguration, CurrentEmdfParserError> {
    let sample_offset = if reader.read_bits(1)? != 0 {
        let value = u16::try_from(reader.read_bits(11)?)
            .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
        if reader.read_bits(1)? != 0 {
            return Err(CurrentEmdfParserError::SampleOffsetExtension);
        }
        Some(value)
    } else {
        None
    };
    let duration = if reader.read_bits(1)? != 0 {
        Some(read_varuint(reader, 11, 2)?)
    } else {
        None
    };
    let group_id = if reader.read_bits(1)? != 0 {
        Some(read_varuint(reader, 2, 8)?)
    } else {
        None
    };
    let codec_data_reserved = if reader.read_bits(1)? != 0 {
        let value = u8::try_from(reader.read_bits(8)?)
            .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?;
        if value != 0 {
            return Err(CurrentEmdfParserError::CodecDataReserved);
        }
        Some(value)
    } else {
        None
    };
    let discard_unknown = reader.read_bits(1)? != 0;
    let mut frame_aligned = None;
    let mut create_duplicate = None;
    let mut remove_duplicate = None;
    let mut priority = None;
    let mut processing_allowed = None;
    if !discard_unknown {
        if sample_offset.is_none() {
            let aligned = reader.read_bits(1)? != 0;
            frame_aligned = Some(aligned);
            if aligned {
                create_duplicate = Some(reader.read_bits(1)? != 0);
                remove_duplicate = Some(reader.read_bits(1)? != 0);
            }
        }
        if sample_offset.is_some() || matches!(frame_aligned, Some(true)) {
            priority = Some(
                u8::try_from(reader.read_bits(5)?)
                    .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?,
            );
            processing_allowed = Some(
                u8::try_from(reader.read_bits(2)?)
                    .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?,
            );
        }
    }

    Ok(PayloadConfiguration {
        sample_offset,
        duration,
        group_id,
        codec_data_reserved,
        discard_unknown,
        frame_aligned,
        create_duplicate,
        remove_duplicate,
        priority,
        processing_allowed,
    })
}

fn validate_selected_configuration(
    id: CurrentPayloadId,
    configuration: PayloadConfiguration,
    mode: CurrentEmdfCarriageMode,
) -> Result<(), CurrentEmdfParserError> {
    let selected_writer_configuration = match id {
        CurrentPayloadId::Programme => Some(Configuration::selected_id1()),
        CurrentPayloadId::AdditionalBsi => Some(Configuration::selected_id2()),
        CurrentPayloadId::Oamd | CurrentPayloadId::Joc => None,
    };
    if let Some(expected) = selected_writer_configuration {
        let parsed = writer_configuration(configuration);
        return if parsed == Some(expected) {
            Ok(())
        } else {
            Err(CurrentEmdfParserError::SelectedConfiguration(
                id.wire_value(),
            ))
        };
    }

    let valid_common = configuration.sample_offset.is_none()
        && configuration.duration.is_none()
        && configuration.group_id == Some(0)
        && !configuration.discard_unknown;
    let valid_mode = match mode {
        CurrentEmdfCarriageMode::DeployedCompat => {
            if configuration.codec_data_reserved.is_some() {
                false
            } else if id == CurrentPayloadId::Oamd {
                configuration.frame_aligned == Some(false)
                    && configuration.create_duplicate.is_none()
                    && configuration.remove_duplicate.is_none()
                    && configuration.priority.is_none()
                    && configuration.processing_allowed.is_none()
            } else {
                configuration.frame_aligned == Some(true)
                    && configuration.create_duplicate == Some(false)
                    && configuration.remove_duplicate == Some(false)
                    && configuration.priority == Some(0)
                    && configuration.processing_allowed == Some(0)
            }
        }
        CurrentEmdfCarriageMode::EtsiEmdf => {
            configuration.codec_data_reserved == Some(0)
                && configuration.frame_aligned == Some(true)
                && configuration.create_duplicate == Some(false)
                && configuration.remove_duplicate == Some(false)
                && configuration.priority == Some(0)
                && configuration.processing_allowed == Some(0)
        }
    };
    if valid_common && valid_mode {
        Ok(())
    } else {
        Err(CurrentEmdfParserError::SelectedConfiguration(
            id.wire_value(),
        ))
    }
}

fn writer_configuration(configuration: PayloadConfiguration) -> Option<Configuration> {
    Configuration::try_new(
        configuration.sample_offset.map(u32::from),
        configuration.duration,
        configuration.group_id,
        u8::from(configuration.codec_data_reserved.is_some()),
        u8::from(configuration.discard_unknown),
        configuration.frame_aligned.map(u8::from),
        configuration.create_duplicate.map(u8::from),
        configuration.remove_duplicate.map(u8::from),
        configuration.priority,
        configuration.processing_allowed,
    )
}

fn selected_payload_capacity(id: CurrentPayloadId) -> u32 {
    match id {
        CurrentPayloadId::Oamd => MAX_OAMD_PAYLOAD_BYTES,
        CurrentPayloadId::Joc => MAX_ID14_PAYLOAD_BYTES,
        CurrentPayloadId::AdditionalBsi => MAX_ID2_PAYLOAD_BYTES,
        CurrentPayloadId::Programme => MAX_ID1_PAYLOAD_BYTES,
    }
}

fn read_varuint(
    reader: &mut BitReader<'_>,
    group_width: u8,
    maximum_groups: u8,
) -> Result<u32, CurrentEmdfParserError> {
    let mut value = reader.read_bits(group_width)?;
    let mut groups = 1_u8;
    while reader.read_bits(1)? != 0 {
        if groups >= maximum_groups {
            return Err(CurrentEmdfParserError::VaruintTooManyGroups);
        }
        let next = reader.read_bits(group_width)?;
        value = value
            .checked_add(1)
            .and_then(|prior| prior.checked_mul(1_u64 << group_width))
            .and_then(|prior| prior.checked_add(next))
            .ok_or(CurrentEmdfParserError::VaruintOverflow)?;
        groups += 1;
    }
    u32::try_from(value).map_err(|_| CurrentEmdfParserError::VaruintOverflow)
}

fn clear_bit_range(
    bytes: &mut [u8],
    start: usize,
    width: usize,
) -> Result<(), CurrentEmdfParserError> {
    let end = start
        .checked_add(width)
        .ok_or(CurrentEmdfParserError::TruncatedBitstream)?;
    if end > bytes.len().saturating_mul(8) {
        return Err(CurrentEmdfParserError::TruncatedBitstream);
    }
    for bit_index in start..end {
        let byte_index = bit_index / 8;
        let bit_in_byte = 7 - (bit_index % 8);
        bytes[byte_index] &= !(1 << bit_in_byte);
    }
    Ok(())
}

struct BitReader<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> BitReader<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    fn read_bits(&mut self, width: u8) -> Result<u64, CurrentEmdfParserError> {
        let width_usize = usize::from(width);
        let end = self
            .position
            .checked_add(width_usize)
            .ok_or(CurrentEmdfParserError::TruncatedBitstream)?;
        if end > self.bytes.len().saturating_mul(8) {
            return Err(CurrentEmdfParserError::TruncatedBitstream);
        }
        let mut value = 0_u64;
        for _ in 0..width_usize {
            let byte_index = self.position / 8;
            let bit_index = 7 - (self.position % 8);
            value = (value << 1) | u64::from((self.bytes[byte_index] >> bit_index) & 1);
            self.position += 1;
        }
        Ok(value)
    }

    fn read_octets(&mut self, count: u32) -> Result<Vec<u8>, CurrentEmdfParserError> {
        let count = usize::try_from(count).map_err(|_| CurrentEmdfParserError::VaruintOverflow)?;
        let bit_count = count
            .checked_mul(8)
            .ok_or(CurrentEmdfParserError::VaruintOverflow)?;
        let end = self
            .position
            .checked_add(bit_count)
            .ok_or(CurrentEmdfParserError::TruncatedBitstream)?;
        if end > self.bytes.len().saturating_mul(8) {
            return Err(CurrentEmdfParserError::TruncatedBitstream);
        }
        let mut output = Vec::with_capacity(count);
        for _ in 0..count {
            output.push(
                u8::try_from(self.read_bits(8)?)
                    .map_err(|_| CurrentEmdfParserError::TruncatedBitstream)?,
            );
        }
        Ok(output)
    }

    const fn position(&self) -> usize {
        self.position
    }

    fn remaining(&self) -> usize {
        self.bytes
            .len()
            .saturating_mul(8)
            .saturating_sub(self.position)
    }

    fn remaining_packed_bits(&self) -> Vec<u8> {
        let remaining = self.remaining();
        let mut packed = vec![0_u8; remaining.div_ceil(8)];
        for offset in 0..remaining {
            let absolute = self.position + offset;
            let source = (self.bytes[absolute / 8] >> (7 - absolute % 8)) & 1;
            packed[offset / 8] |= source << (7 - offset % 8);
        }
        packed
    }

    fn skip_bits(&mut self, count: usize) -> Result<(), CurrentEmdfParserError> {
        let end = self
            .position
            .checked_add(count)
            .ok_or(CurrentEmdfParserError::TruncatedBitstream)?;
        if end > self.bytes.len().saturating_mul(8) {
            return Err(CurrentEmdfParserError::TruncatedBitstream);
        }
        self.position = end;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BitReader, CarrierBlock, CurrentEmdfCarriageMode, CurrentEmdfParserError, CurrentPayloadId,
        PayloadConfiguration, check_joc_payload_capacity, clear_bit_range, read_varuint,
        selected_payload_capacity, selected_xy_table, selected_z_table, validate_outer_header,
        validate_selected_configuration,
    };
    use crate::integrity::keyless_checksum::{crc8_smbus, crc32_iso_hdlc};

    #[test]
    fn approved_f14k_crc_primitives_match_known_check_values() {
        assert_eq!(crc32_iso_hdlc(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc8_smbus(b"123456789"), 0xf4);
    }

    #[test]
    fn checksum_zeroing_preserves_uncovered_neighbor_bits() {
        let mut bytes = [0b1011_0110, 0b0101_1101, 0b1110_0011];
        clear_bit_range(&mut bytes, 3, 10).unwrap();
        assert_eq!(bytes, [0b1010_0000, 0b0000_0101, 0b1110_0011]);
    }

    #[test]
    fn payload_length_varuint_decodes_single_and_continued_groups() {
        let mut zero = BitReader::new(&[0x00, 0x00]);
        assert_eq!(read_varuint(&mut zero, 8, 2).unwrap(), 0);

        let mut one = BitReader::new(&[0x01, 0x00]);
        assert_eq!(read_varuint(&mut one, 8, 2).unwrap(), 1);

        let mut single = BitReader::new(&[0xff, 0x00]);
        assert_eq!(read_varuint(&mut single, 8, 2).unwrap(), 255);

        let mut continued = BitReader::new(&[0x00, 0x80, 0x00]);
        assert_eq!(read_varuint(&mut continued, 8, 2).unwrap(), 256);
    }

    #[test]
    fn payload_length_varuint_rejects_excess_groups_and_truncation() {
        let mut too_many = BitReader::new(&[0x00, 0x80, 0x00]);
        assert_eq!(
            read_varuint(&mut too_many, 8, 1),
            Err(CurrentEmdfParserError::VaruintTooManyGroups)
        );

        let mut truncated = BitReader::new(&[0x00]);
        assert_eq!(
            read_varuint(&mut truncated, 8, 2),
            Err(CurrentEmdfParserError::TruncatedBitstream)
        );
    }

    #[test]
    fn outer_header_rejects_bad_sync_and_nonexact_length() {
        assert_eq!(
            validate_outer_header(&[0, 0, 0, 0]),
            Err(CurrentEmdfParserError::InvalidSyncWord)
        );
        assert_eq!(
            validate_outer_header(&[0x58, 0x38, 0, 1]),
            Err(CurrentEmdfParserError::OuterLengthMismatch)
        );
    }

    #[test]
    fn carrier_block_construction_enforces_selected_domain() {
        assert!(CarrierBlock::new(0).is_ok());
        assert!(CarrierBlock::new(5).is_ok());
        assert_eq!(
            CarrierBlock::new(6),
            Err(CurrentEmdfParserError::CarrierBlockOutOfRange)
        );
    }

    #[test]
    fn approved_oamd_coordinate_formulas_produce_selected_boundaries() {
        let xy = selected_xy_table();
        let z = selected_z_table();
        assert_eq!([xy[0], xy[31], xy[62]], [0, 16_384, 32_767]);
        assert_eq!([z[0], z[15]], [0, 32_767]);
        assert!(xy.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(z.windows(2).all(|pair| pair[0] < pair[1]));
    }

    #[test]
    fn selected_id14_payload_capacity_is_bounded() {
        assert_eq!(
            check_joc_payload_capacity(&[0; 373]),
            Err(CurrentEmdfParserError::JocBodyCapacity)
        );
    }

    #[test]
    fn selected_id1_and_id2_configs_match_the_typed_producer_projections() {
        let id1 = PayloadConfiguration {
            sample_offset: None,
            duration: Some(1_536),
            group_id: None,
            codec_data_reserved: None,
            discard_unknown: false,
            frame_aligned: Some(false),
            create_duplicate: None,
            remove_duplicate: None,
            priority: None,
            processing_allowed: None,
        };
        let id2 = PayloadConfiguration {
            sample_offset: None,
            duration: None,
            group_id: None,
            codec_data_reserved: None,
            discard_unknown: true,
            frame_aligned: None,
            create_duplicate: None,
            remove_duplicate: None,
            priority: None,
            processing_allowed: None,
        };
        for mode in [
            CurrentEmdfCarriageMode::DeployedCompat,
            CurrentEmdfCarriageMode::EtsiEmdf,
        ] {
            assert_eq!(
                validate_selected_configuration(CurrentPayloadId::Programme, id1, mode),
                Ok(())
            );
            assert_eq!(
                validate_selected_configuration(CurrentPayloadId::AdditionalBsi, id2, mode),
                Ok(())
            );
        }
        let mismatched_id1 = PayloadConfiguration {
            duration: Some(1_535),
            ..id1
        };
        assert_eq!(
            validate_selected_configuration(
                CurrentPayloadId::Programme,
                mismatched_id1,
                CurrentEmdfCarriageMode::DeployedCompat,
            ),
            Err(CurrentEmdfParserError::SelectedConfiguration(1))
        );
    }

    #[test]
    fn selected_payload_capacities_match_the_f13_owner_limits() {
        assert_eq!(selected_payload_capacity(CurrentPayloadId::Oamd), 160);
        assert_eq!(selected_payload_capacity(CurrentPayloadId::Programme), 10);
        assert_eq!(
            selected_payload_capacity(CurrentPayloadId::AdditionalBsi),
            8
        );
        assert_eq!(selected_payload_capacity(CurrentPayloadId::Joc), 372);
    }
}
