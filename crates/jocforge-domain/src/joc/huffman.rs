// pattern: Functional Core
//! Hash-pinned F05Q Huffman codebooks and pure bitstream operations.

use sha2::{Digest, Sha256};
use std::fmt;

pub mod id14_header;
pub use id14_header::{
    Id14HeaderInput, Id14InputShapeError, Id14ObjectDescriptor, Id14OutputLength, Id14Presence,
    Id14WriterOutcome, Id14WriterReturnCode, Id14WriterState, write_id14_header,
};

const SELECTED_CODEBOOKS: &str = include_str!("huffman/codebooks.json");
const SELECTED_CODEBOOKS_SHA256: &str =
    "515d635adc6099493dfd27f787bdf2b9f19a4b4bb1180860b0e820a23fb5d5ae";
const TABLE_ORDER: [HuffmanCodebookId; 6] = [
    HuffmanCodebookId::FullCoarse,
    HuffmanCodebookId::FullFine,
    HuffmanCodebookId::SparseCoarse,
    HuffmanCodebookId::SparseFine,
    HuffmanCodebookId::Position5,
    HuffmanCodebookId::Position7,
];

/// The six codebooks selected by the F05Q Huffman consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HuffmanCodebookId {
    FullCoarse,
    FullFine,
    SparseCoarse,
    SparseFine,
    Position5,
    Position7,
}

/// A count of meaningful or padding Huffman bits.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HuffmanBitCount(u64);

impl HuffmanBitCount {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

/// A number of symbols in a Huffman table or decoded symbol sequence.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HuffmanSymbolCount(u16);

impl HuffmanSymbolCount {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u16 {
        self.0
    }
}

/// A byte capacity for an F05Q Huffman candidate payload.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct HuffmanByteCapacity(u64);

impl HuffmanByteCapacity {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

impl HuffmanCodebookId {
    const fn index(self) -> usize {
        match self {
            Self::FullCoarse => 0,
            Self::FullFine => 1,
            Self::SparseCoarse => 2,
            Self::SparseFine => 3,
            Self::Position5 => 4,
            Self::Position7 => 5,
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::FullCoarse => "full_coarse",
            Self::FullFine => "full_fine",
            Self::SparseCoarse => "sparse_coarse",
            Self::SparseFine => "sparse_fine",
            Self::Position5 => "position_5",
            Self::Position7 => "position_7",
        }
    }
}

/// A local table error uses the package's clean-room error labels.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HuffmanError {
    TableInvalid,
    SymbolOutOfRange {
        symbol: i64,
        symbol_count: HuffmanSymbolCount,
    },
    TruncatedCodeword,
    CoefficientScratchExceeded,
    PayloadCapacityExceeded,
}

impl HuffmanError {
    pub const fn code(&self) -> &'static str {
        match self {
            Self::TableInvalid => "HUFFMAN_TABLE_INVALID",
            Self::SymbolOutOfRange { .. } => "HUFFMAN_SYMBOL_OUT_OF_RANGE",
            Self::TruncatedCodeword => "HUFFMAN_TRUNCATED_CODEWORD",
            Self::CoefficientScratchExceeded => "JOC_COEFFICIENT_SCRATCH_EXCEEDED",
            Self::PayloadCapacityExceeded => "JOC_PAYLOAD_CAPACITY_EXCEEDED",
        }
    }
}

impl fmt::Display for HuffmanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.code())
    }
}

impl std::error::Error for HuffmanError {}

/// A local shape rejection for the typed F05Q input view, not a native error-code mapping.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum F05QConsumerError {
    InputShape,
    Huffman(HuffmanError),
}

impl From<HuffmanError> for F05QConsumerError {
    fn from(error: HuffmanError) -> Self {
        Self::Huffman(error)
    }
}

impl fmt::Display for F05QConsumerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InputShape => formatter.write_str("invalid F05Q consumer input shape"),
            Self::Huffman(error) => fmt::Display::fmt(error, formatter),
        }
    }
}

impl std::error::Error for F05QConsumerError {}

#[derive(Clone, Copy, Debug)]
struct Codeword {
    code: u32,
    bit_length: u8,
}

#[derive(Clone, Copy, Debug, Default)]
struct DecodeNode {
    children: [Option<usize>; 2],
    symbol: Option<u16>,
}

/// An immutable six-table set, validated before any table can be used.
#[derive(Debug)]
pub struct HuffmanCodebooks {
    tables: [HuffmanTable; 6],
}

impl HuffmanCodebooks {
    /// Loads the hash-pinned selected tables embedded with the domain crate.
    pub fn selected() -> Result<Self, HuffmanError> {
        if sha256_hex(SELECTED_CODEBOOKS.as_bytes()) != SELECTED_CODEBOOKS_SHA256 {
            return Err(HuffmanError::TableInvalid);
        }
        Self::from_json(SELECTED_CODEBOOKS)
    }

    /// Returns the fully validated table identified by its F05Q role.
    pub fn table(&self, id: HuffmanCodebookId) -> &HuffmanTable {
        &self.tables[id.index()]
    }

    /// Encodes only the F05Q codebook-selection and delta-write connection.
    /// It does not run coefficient fitting, retry policy, ID14 framing, or parent commit.
    pub fn encode_f05q_consumer(
        &self,
        input: &F05QConsumerInput,
    ) -> Result<F05QConsumerOutput, F05QConsumerError> {
        let (coefficient_id, position_id) = match input.mode {
            0 => (HuffmanCodebookId::FullCoarse, None),
            1 => (HuffmanCodebookId::FullFine, None),
            2 => (
                HuffmanCodebookId::SparseCoarse,
                Some(position_table_id(input.channel_count)?),
            ),
            3 => (
                HuffmanCodebookId::SparseFine,
                Some(position_table_id(input.channel_count)?),
            ),
            _ => return Err(F05QConsumerError::InputShape),
        };
        let coefficient_table = self.table(coefficient_id);
        let position_table = position_id.map(|id| self.table(id));
        let initial_previous = match input.mode {
            0 | 1 => i64::from(coefficient_table.symbol_count().value() / 2),
            2 => 50,
            3 => 100,
            _ => return Err(F05QConsumerError::InputShape),
        };
        let mut writer = HuffmanBitWriter::default();
        let mut reset_count = 0_u64;
        let mut coefficient_write_count = 0_u64;
        let mut position_write_count = 0_u64;
        let mut coefficient_bit_count = 0_u64;
        let mut position_bit_count = 0_u64;

        for point in &input.points {
            match (input.mode < 2, point) {
                (true, F05QConsumerPointInput::Full { quantized_rows }) => {
                    if quantized_rows.len() != usize::from(input.channel_count) {
                        return Err(F05QConsumerError::InputShape);
                    }
                    for row in quantized_rows {
                        reset_count += 1;
                        let mut previous = initial_previous;
                        for &quantized in row {
                            let symbol = (i128::from(quantized) - i128::from(previous))
                                .rem_euclid(i128::from(coefficient_table.symbol_count().value()))
                                as i64;
                            let codeword = coefficient_table.codeword(symbol)?;
                            writer.write_codeword(codeword);
                            previous = quantized;
                            coefficient_write_count += 1;
                            coefficient_bit_count += u64::from(codeword.bit_length);
                        }
                    }
                }
                (
                    false,
                    F05QConsumerPointInput::Sparse {
                        selected_channels,
                        quantized_coefficients,
                    },
                ) => {
                    reset_count += 1;
                    let mut previous_channel: Option<i64> = None;
                    let channel_count = i64::from(input.channel_count);
                    for &channel in selected_channels {
                        if channel < 0 || channel >= channel_count {
                            return Err(F05QConsumerError::Huffman(
                                HuffmanError::SymbolOutOfRange {
                                    symbol: channel,
                                    symbol_count: HuffmanSymbolCount::new(u16::from(
                                        input.channel_count,
                                    )),
                                },
                            ));
                        }
                        if let Some(previous) = previous_channel {
                            let position_symbol = (channel - previous).rem_euclid(channel_count);
                            let table = position_table.ok_or(HuffmanError::TableInvalid)?;
                            let codeword = table.codeword(position_symbol)?;
                            writer.write_codeword(codeword);
                            position_bit_count += u64::from(codeword.bit_length);
                        } else {
                            writer.write_raw_bits(
                                u32::try_from(channel)
                                    .map_err(|_| F05QConsumerError::InputShape)?,
                                3,
                            );
                            position_bit_count += 3;
                        }
                        previous_channel = Some(channel);
                        position_write_count += 1;
                    }

                    let mut previous = initial_previous;
                    for &quantized in quantized_coefficients {
                        let symbol = (i128::from(quantized) - i128::from(previous))
                            .rem_euclid(i128::from(coefficient_table.symbol_count().value()))
                            as i64;
                        let codeword = coefficient_table.codeword(symbol)?;
                        writer.write_codeword(codeword);
                        previous = quantized;
                        coefficient_write_count += 1;
                        coefficient_bit_count += u64::from(codeword.bit_length);
                    }
                }
                _ => return Err(F05QConsumerError::InputShape),
            }
        }

        Ok(F05QConsumerOutput {
            coefficient_codebook: coefficient_id,
            position_codebook: position_id,
            steps: coefficient_table.symbol_count(),
            initial_previous: u16::try_from(initial_previous)
                .map_err(|_| F05QConsumerError::InputShape)?,
            reset_count,
            coefficient_write_count,
            position_write_count,
            coefficient_bit_count: HuffmanBitCount::new(coefficient_bit_count),
            position_bit_count: HuffmanBitCount::new(position_bit_count),
            bitstream: writer.finish(),
        })
    }

    fn from_json(source: &str) -> Result<Self, HuffmanError> {
        let root: serde_json::Value =
            serde_json::from_str(source).map_err(|_| HuffmanError::TableInvalid)?;
        if root["schema"] != "joc-huffman-codebooks.v1" {
            return Err(HuffmanError::TableInvalid);
        }
        let representation = &root["representation"];
        let represented_order = representation["table_order"]
            .as_array()
            .ok_or(HuffmanError::TableInvalid)?;
        if representation["entry_order"] != "symbol ascending from zero"
            || representation["bit_order"] != "MSB_FIRST"
            || represented_order.len() != TABLE_ORDER.len()
            || TABLE_ORDER
                .iter()
                .zip(represented_order)
                .any(|(expected, actual)| actual.as_str() != Some(expected.name()))
        {
            return Err(HuffmanError::TableInvalid);
        }

        let tables_object = root["tables"]
            .as_object()
            .ok_or(HuffmanError::TableInvalid)?;
        if tables_object.len() != TABLE_ORDER.len()
            || TABLE_ORDER
                .iter()
                .any(|table_id| !tables_object.contains_key(table_id.name()))
        {
            return Err(HuffmanError::TableInvalid);
        }

        let mut aggregate_binary = Vec::new();
        let mut tables = Vec::with_capacity(TABLE_ORDER.len());
        for table_id in TABLE_ORDER {
            let table_value = &tables_object[table_id.name()];
            let entries = table_value["entries"]
                .as_array()
                .ok_or(HuffmanError::TableInvalid)?;
            let expected_count = usize_value(&table_value["symbol_count"])?;
            if entries.len() != expected_count
                || expected_count == 0
                || expected_count > u16::MAX as usize
            {
                return Err(HuffmanError::TableInvalid);
            }
            let symbol_count = HuffmanSymbolCount::new(
                u16::try_from(expected_count).map_err(|_| HuffmanError::TableInvalid)?,
            );
            let expected_nodes = usize_value(&table_value["node_count"])?;
            let mut entry_binary = Vec::with_capacity(entries.len() * 7);
            let mut codewords = Vec::with_capacity(entries.len());
            let mut nodes = vec![DecodeNode::default()];
            let mut kraft_numerator = 0_u64;

            for (expected_symbol, entry) in entries.iter().enumerate() {
                let symbol_index = usize_value(&entry["symbol"])?;
                let symbol = u16::try_from(symbol_index).map_err(|_| HuffmanError::TableInvalid)?;
                let code = u32_value(&entry["code"])?;
                let bit_length = u8::try_from(usize_value(&entry["bit_length"])?)
                    .map_err(|_| HuffmanError::TableInvalid)?;
                if usize::from(symbol) != expected_symbol || bit_length == 0 || bit_length > 24 {
                    return Err(HuffmanError::TableInvalid);
                }

                let limit = 1_u32 << bit_length;
                let expected_bits = format_bits(code, bit_length);
                if code >= limit || entry["msb_first_bits"].as_str() != Some(expected_bits.as_str())
                {
                    return Err(HuffmanError::TableInvalid);
                }

                insert_codeword(&mut nodes, symbol, code, bit_length)?;
                kraft_numerator += 1_u64 << (24 - bit_length);
                entry_binary.extend_from_slice(&symbol.to_le_bytes());
                entry_binary.push(bit_length);
                entry_binary.extend_from_slice(&code.to_le_bytes());
                codewords.push(Codeword { code, bit_length });
            }

            let expected_table_hash = sha256_hex(&entry_binary);
            let internal_node_count = nodes.iter().filter(|node| node.symbol.is_none()).count();
            if internal_node_count != expected_nodes
                || expected_nodes != expected_count - 1
                || kraft_numerator != (1_u64 << 24)
                || !is_complete_tree(&nodes)
                || table_value["stable_binary_sha256"].as_str()
                    != Some(expected_table_hash.as_str())
            {
                return Err(HuffmanError::TableInvalid);
            }

            let name = table_id.name().as_bytes();
            let name_len = u8::try_from(name.len()).map_err(|_| HuffmanError::TableInvalid)?;
            aggregate_binary.push(name_len);
            aggregate_binary.extend_from_slice(name);
            aggregate_binary.extend_from_slice(&symbol_count.value().to_le_bytes());
            aggregate_binary.extend_from_slice(&entry_binary);
            tables.push(HuffmanTable {
                id: table_id,
                symbol_count,
                codewords,
                nodes,
            });
        }

        let expected_aggregate_bytes = usize_value(&root["aggregate_stable_binary_bytes"])?;
        let expected_aggregate_hash = sha256_hex(&aggregate_binary);
        if aggregate_binary.len() != expected_aggregate_bytes
            || root["aggregate_stable_binary_sha256"].as_str()
                != Some(expected_aggregate_hash.as_str())
        {
            return Err(HuffmanError::TableInvalid);
        }

        let tables: [HuffmanTable; 6] =
            tables.try_into().map_err(|_| HuffmanError::TableInvalid)?;
        Ok(Self { tables })
    }
}

/// Point-local inputs to the table-only F05Q coefficient consumer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum F05QConsumerPointInput {
    Full {
        quantized_rows: Vec<Vec<i64>>,
    },
    Sparse {
        selected_channels: Vec<i64>,
        quantized_coefficients: Vec<i64>,
    },
}

/// A typed input for the F05Q Huffman table-selection connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct F05QConsumerInput {
    pub mode: u8,
    pub channel_count: u8,
    pub points: Vec<F05QConsumerPointInput>,
}

/// Bitstream and table-selection facts from the F05Q Huffman connection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct F05QConsumerOutput {
    coefficient_codebook: HuffmanCodebookId,
    position_codebook: Option<HuffmanCodebookId>,
    steps: HuffmanSymbolCount,
    initial_previous: u16,
    reset_count: u64,
    coefficient_write_count: u64,
    position_write_count: u64,
    coefficient_bit_count: HuffmanBitCount,
    position_bit_count: HuffmanBitCount,
    bitstream: HuffmanBitstream,
}

impl F05QConsumerOutput {
    pub const fn coefficient_codebook(&self) -> HuffmanCodebookId {
        self.coefficient_codebook
    }

    pub const fn position_codebook(&self) -> Option<HuffmanCodebookId> {
        self.position_codebook
    }

    pub const fn steps(&self) -> HuffmanSymbolCount {
        self.steps
    }

    pub const fn initial_previous(&self) -> u16 {
        self.initial_previous
    }

    pub const fn reset_count(&self) -> u64 {
        self.reset_count
    }

    pub const fn coefficient_write_count(&self) -> u64 {
        self.coefficient_write_count
    }

    pub const fn position_write_count(&self) -> u64 {
        self.position_write_count
    }

    pub const fn coefficient_bit_count(&self) -> HuffmanBitCount {
        self.coefficient_bit_count
    }

    pub const fn position_bit_count(&self) -> HuffmanBitCount {
        self.position_bit_count
    }

    pub const fn write_count(&self) -> u64 {
        self.coefficient_write_count + self.position_write_count
    }

    pub const fn bitstream(&self) -> &HuffmanBitstream {
        &self.bitstream
    }
}

/// A validated codebook and its complete binary decode tree.
#[derive(Debug)]
pub struct HuffmanTable {
    id: HuffmanCodebookId,
    symbol_count: HuffmanSymbolCount,
    codewords: Vec<Codeword>,
    nodes: Vec<DecodeNode>,
}

impl HuffmanTable {
    pub const fn id(&self) -> HuffmanCodebookId {
        self.id
    }

    pub const fn symbol_count(&self) -> HuffmanSymbolCount {
        self.symbol_count
    }

    /// Encodes the complete input in MSB-first order and zero-pads only the final byte.
    ///
    /// The integer slice excludes booleans at the type boundary.
    ///
    /// ```compile_fail
    /// use jocforge_domain::joc::huffman::HuffmanCodebooks;
    /// let codebooks = HuffmanCodebooks::selected().unwrap();
    /// codebooks.table(jocforge_domain::joc::huffman::HuffmanCodebookId::FullCoarse)
    ///     .encode_symbols(&[true]);
    /// ```
    pub fn encode_symbols(&self, symbols: &[i64]) -> Result<HuffmanBitstream, HuffmanError> {
        let mut writer = HuffmanBitWriter::default();
        writer.write_symbols(self, symbols)?;
        Ok(writer.finish())
    }

    /// Decodes exactly `symbol_count` complete codewords without consuming partial input on error.
    pub fn decode_symbols(
        &self,
        bytes: &[u8],
        valid_bit_count: HuffmanBitCount,
        symbol_count: HuffmanSymbolCount,
    ) -> Result<Vec<u16>, HuffmanError> {
        let mut reader = HuffmanBitReader::new(bytes, valid_bit_count);
        reader.read_symbols(self, symbol_count)
    }

    fn codeword(&self, symbol: i64) -> Result<Codeword, HuffmanError> {
        let index = usize::try_from(symbol)
            .ok()
            .filter(|index| *index < self.codewords.len())
            .ok_or(HuffmanError::SymbolOutOfRange {
                symbol,
                symbol_count: self.symbol_count,
            })?;
        Ok(self.codewords[index])
    }
}

/// The padded bytes and number of meaningful bits produced by a Huffman write.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HuffmanBitstream {
    bytes: Vec<u8>,
    valid_bit_count: HuffmanBitCount,
    padding_zero_bits: u8,
}

impl HuffmanBitstream {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub const fn valid_bit_count(&self) -> HuffmanBitCount {
        self.valid_bit_count
    }

    pub const fn padding_zero_bits(&self) -> u8 {
        self.padding_zero_bits
    }
}

/// A transactional writer for a sequence of codewords.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HuffmanBitWriter {
    bytes: Vec<u8>,
    valid_bit_count: usize,
}

impl HuffmanBitWriter {
    pub fn valid_bit_count(&self) -> HuffmanBitCount {
        HuffmanBitCount::new(u64::try_from(self.valid_bit_count).unwrap_or(u64::MAX))
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Commits all codewords together; an invalid symbol leaves this writer unchanged.
    pub fn write_symbols(
        &mut self,
        table: &HuffmanTable,
        symbols: &[i64],
    ) -> Result<(), HuffmanError> {
        let mut trial = self.clone();
        for &symbol in symbols {
            let codeword = table.codeword(symbol)?;
            trial.write_codeword(codeword);
        }
        *self = trial;
        Ok(())
    }

    /// Writes a bounded candidate and commits it only after both capacity checks pass.
    pub fn write_symbols_with_capacity(
        &mut self,
        table: &HuffmanTable,
        symbols: &[i64],
        header_bits: HuffmanBitCount,
        scratch_limit_bits: HuffmanBitCount,
        capacity_bytes: HuffmanByteCapacity,
    ) -> Result<(), HuffmanError> {
        let mut trial = self.clone();
        for &symbol in symbols {
            let codeword = table.codeword(symbol)?;
            trial.write_codeword(codeword);
        }
        check_f05q_capacity(
            HuffmanBitCount::new(u64::try_from(trial.valid_bit_count).unwrap_or(u64::MAX)),
            header_bits,
            scratch_limit_bits,
            capacity_bytes,
        )?;
        *self = trial;
        Ok(())
    }

    /// Appends raw MSB-first bits, used only for F05Q's first sparse channel index.
    fn write_raw_bits(&mut self, value: u32, bit_length: u8) {
        for shift in (0..bit_length).rev() {
            self.write_bit(((value >> shift) & 1) as u8);
        }
    }

    fn write_codeword(&mut self, codeword: Codeword) {
        for shift in (0..codeword.bit_length).rev() {
            self.write_bit(((codeword.code >> shift) & 1) as u8);
        }
    }

    fn write_bit(&mut self, bit: u8) {
        let byte_index = self.valid_bit_count / 8;
        let bit_index = 7 - (self.valid_bit_count % 8);
        if byte_index == self.bytes.len() {
            self.bytes.push(0);
        }
        if bit != 0 {
            self.bytes[byte_index] |= 1 << bit_index;
        }
        self.valid_bit_count += 1;
    }

    pub fn finish(self) -> HuffmanBitstream {
        let padding_zero_bits = ((8 - (self.valid_bit_count % 8)) % 8) as u8;
        HuffmanBitstream {
            bytes: self.bytes,
            valid_bit_count: HuffmanBitCount::new(
                u64::try_from(self.valid_bit_count).unwrap_or(u64::MAX),
            ),
            padding_zero_bits,
        }
    }
}

/// Applies F05Q's coefficient scratch limit before checking the padded payload bytes.
pub fn check_f05q_capacity(
    coefficient_bits: HuffmanBitCount,
    header_bits: HuffmanBitCount,
    scratch_limit_bits: HuffmanBitCount,
    capacity_bytes: HuffmanByteCapacity,
) -> Result<(), HuffmanError> {
    if coefficient_bits.value() > scratch_limit_bits.value() {
        return Err(HuffmanError::CoefficientScratchExceeded);
    }
    let total_bits = header_bits
        .value()
        .checked_add(coefficient_bits.value())
        .ok_or(HuffmanError::PayloadCapacityExceeded)?;
    let padded_bytes = total_bits / 8 + u64::from(total_bits % 8 != 0);
    if padded_bytes > capacity_bytes.value() {
        return Err(HuffmanError::PayloadCapacityExceeded);
    }
    Ok(())
}

/// A transactional reader; failed reads leave its bit cursor unchanged.
pub struct HuffmanBitReader<'a> {
    bytes: &'a [u8],
    valid_bit_count: HuffmanBitCount,
    bit_cursor: usize,
}

impl<'a> HuffmanBitReader<'a> {
    pub fn new(bytes: &'a [u8], valid_bit_count: HuffmanBitCount) -> Self {
        Self {
            bytes,
            valid_bit_count,
            bit_cursor: 0,
        }
    }

    pub fn bit_cursor(&self) -> HuffmanBitCount {
        HuffmanBitCount::new(u64::try_from(self.bit_cursor).unwrap_or(u64::MAX))
    }

    pub fn read_symbol(&mut self, table: &HuffmanTable) -> Result<u16, HuffmanError> {
        let mut trial_cursor = self.bit_cursor;
        let symbol = self.read_symbol_at(table, &mut trial_cursor)?;
        self.bit_cursor = trial_cursor;
        Ok(symbol)
    }

    pub fn read_symbols(
        &mut self,
        table: &HuffmanTable,
        symbol_count: HuffmanSymbolCount,
    ) -> Result<Vec<u16>, HuffmanError> {
        let mut trial_cursor = self.bit_cursor;
        let count = usize::from(symbol_count.value());
        let mut symbols = Vec::with_capacity(count);
        for _ in 0..count {
            symbols.push(self.read_symbol_at(table, &mut trial_cursor)?);
        }
        self.bit_cursor = trial_cursor;
        Ok(symbols)
    }

    fn read_symbol_at(
        &self,
        table: &HuffmanTable,
        bit_cursor: &mut usize,
    ) -> Result<u16, HuffmanError> {
        let mut node_index = 0;
        loop {
            if u64::try_from(*bit_cursor).unwrap_or(u64::MAX) >= self.valid_bit_count.value()
                || *bit_cursor / 8 >= self.bytes.len()
            {
                return Err(HuffmanError::TruncatedCodeword);
            }
            let byte_index = *bit_cursor / 8;
            let bit_index = 7 - (*bit_cursor % 8);
            let branch = ((self.bytes[byte_index] >> bit_index) & 1) as usize;
            *bit_cursor += 1;
            node_index =
                table.nodes[node_index].children[branch].ok_or(HuffmanError::TableInvalid)?;
            if let Some(symbol) = table.nodes[node_index].symbol {
                return Ok(symbol);
            }
        }
    }
}

fn insert_codeword(
    nodes: &mut Vec<DecodeNode>,
    symbol: u16,
    code: u32,
    bit_length: u8,
) -> Result<(), HuffmanError> {
    let mut current = 0;
    for shift in (0..bit_length).rev() {
        if nodes[current].symbol.is_some() {
            return Err(HuffmanError::TableInvalid);
        }
        let branch = ((code >> shift) & 1) as usize;
        let next = if let Some(index) = nodes[current].children[branch] {
            index
        } else {
            let index = nodes.len();
            nodes.push(DecodeNode::default());
            nodes[current].children[branch] = Some(index);
            index
        };
        current = next;
    }
    if nodes[current].symbol.is_some() || nodes[current].children != [None, None] {
        return Err(HuffmanError::TableInvalid);
    }
    nodes[current].symbol = Some(symbol);
    Ok(())
}

fn is_complete_tree(nodes: &[DecodeNode]) -> bool {
    nodes.iter().all(|node| match node.symbol {
        Some(_) => node.children == [None, None],
        None => node.children[0].is_some() && node.children[1].is_some(),
    })
}

fn position_table_id(channel_count: u8) -> Result<HuffmanCodebookId, F05QConsumerError> {
    match channel_count {
        5 => Ok(HuffmanCodebookId::Position5),
        7 => Ok(HuffmanCodebookId::Position7),
        _ => Err(F05QConsumerError::InputShape),
    }
}

fn usize_value(value: &serde_json::Value) -> Result<usize, HuffmanError> {
    value
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or(HuffmanError::TableInvalid)
}

fn u32_value(value: &serde_json::Value) -> Result<u32, HuffmanError> {
    value
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or(HuffmanError::TableInvalid)
}

fn format_bits(code: u32, bit_length: u8) -> String {
    format!("{:0width$b}", code, width = usize::from(bit_length))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use fmt::Write as _;
        write!(&mut hex, "{byte:02x}").expect("writing to String cannot fail");
    }
    hex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codebook_provider_rejects_prefix_conflicts_before_use() {
        let mut parameters: serde_json::Value = serde_json::from_str(SELECTED_CODEBOOKS).unwrap();
        parameters["tables"]["position_5"]["entries"][1]["code"] = 0.into();
        parameters["tables"]["position_5"]["entries"][1]["bit_length"] = 1.into();
        parameters["tables"]["position_5"]["entries"][1]["msb_first_bits"] = "0".into();
        assert_eq!(
            HuffmanCodebooks::from_json(&parameters.to_string()).unwrap_err(),
            HuffmanError::TableInvalid,
        );
    }
}
