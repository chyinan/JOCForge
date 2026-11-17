// pattern: Functional Core
//! Normalizes caller-projected six-byte ID14 row members under the sealed static formula.
//!
//! The input member meanings remain unbound to ADM, OAMD, or EMDF fields. This module only
//! applies the observed classifier/run grouping and marker padding to the supplied byte rows;
//! it does not create coefficient rows, write an ID14 payload, or commit state.

pub const ID14_ROW_STRIDE_BYTES: usize = 6;

const MIN_ROW_COUNT: usize = 1;
const MAX_ROW_COUNT: usize = 128;
const CLASSIFIER_OFFSET_BYTES: usize = 2;
const RUN_VALUE_OFFSET_BYTES: usize = 4;

/// Valid count for the sealed ID14 raw-row normalizer input or initialized row state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14RowCount(u8);

impl Id14RowCount {
    pub fn new(value: usize) -> Result<Self, Id14RowNormalizationError> {
        if !(MIN_ROW_COUNT..=MAX_ROW_COUNT).contains(&value) {
            return Err(Id14RowNormalizationError::RowCountOutsideContract { value });
        }
        Ok(Self(u8::try_from(value).map_err(|_| {
            Id14RowNormalizationError::RowCountOutsideContract { value }
        })?))
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    fn as_usize(self) -> usize {
        usize::from(self.0)
    }
}

/// One normalized U16 word. Its semantic relationship to an ADM/JOC field is unbound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Id14NormalizedWord(u16);

impl Id14NormalizedWord {
    pub const fn value(self) -> u16 {
        self.0
    }
}

/// Noncommitting output of one row-normalization call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Id14NormalizedRows {
    input_count: Id14RowCount,
    output_count: Id14RowCount,
    words: Vec<Id14NormalizedWord>,
}

impl Id14NormalizedRows {
    pub const fn input_count(&self) -> Id14RowCount {
        self.input_count
    }

    /// Includes marker padding copied from a larger initialized prior row count.
    pub const fn output_count(&self) -> Id14RowCount {
        self.output_count
    }

    pub fn words(&self) -> &[Id14NormalizedWord] {
        &self.words
    }
}

/// Failures in the bounded structural normalizer input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Id14RowNormalizationError {
    RowCountOutsideContract { value: usize },
    RowByteExtentMismatch { expected: usize, actual: usize },
    UnmappedClassifier { row_index: usize, value: i8 },
}

impl Id14RowNormalizationError {
    /// Return code from the corresponding sealed branch when one was observed.
    pub const fn contract_return_code(&self) -> Option<u8> {
        match self {
            Self::RowCountOutsideContract { .. } => Some(5),
            Self::UnmappedClassifier { .. } => Some(7),
            Self::RowByteExtentMismatch { .. } => None,
        }
    }
}

/// Normalizes `row_count` caller-projected raw rows and pads to a larger initialized state count.
///
/// The classifier is the signed byte at offset +2. The little-endian U16 run-group value is read
/// at offset +4 for the x86 Windows target. Offset +3 is not read. `prior_state_count=None` models
/// an uninitialized state; an admitted prior count larger than this call's count adds
/// `0x8000 | row_index` markers.
pub fn normalize_id14_six_byte_rows(
    row_count: usize,
    row_bytes: &[u8],
    prior_state_count: Option<Id14RowCount>,
) -> Result<Id14NormalizedRows, Id14RowNormalizationError> {
    let input_count = Id14RowCount::new(row_count)?;
    let expected_bytes = row_count * ID14_ROW_STRIDE_BYTES;
    if row_bytes.len() != expected_bytes {
        return Err(Id14RowNormalizationError::RowByteExtentMismatch {
            expected: expected_bytes,
            actual: row_bytes.len(),
        });
    }

    let output_count = prior_state_count.map_or(input_count, |prior| {
        if prior.as_usize() > input_count.as_usize() {
            prior
        } else {
            input_count
        }
    });
    let mut words = Vec::with_capacity(output_count.as_usize());
    let mut previous_run_value = None;
    let mut run_count = 1_u8;

    for row_index in 0..row_count {
        let row_start = row_index * ID14_ROW_STRIDE_BYTES;
        let classifier = i8::from_ne_bytes([row_bytes[row_start + CLASSIFIER_OFFSET_BYTES]]);
        if classifier == -1 {
            words.push(Id14NormalizedWord(0x8000 | (row_index as u16 & 0x00ff)));
            continue;
        }

        let mapped_code =
            classifier_code(classifier).ok_or(Id14RowNormalizationError::UnmappedClassifier {
                row_index,
                value: classifier,
            })?;
        let run_value = u16::from_le_bytes([
            row_bytes[row_start + RUN_VALUE_OFFSET_BYTES],
            row_bytes[row_start + RUN_VALUE_OFFSET_BYTES + 1],
        ]);
        if let Some(previous) = previous_run_value {
            if previous != run_value {
                run_count += 1;
            }
        }
        previous_run_value = Some(run_value);

        let tag = u16::from(mapped_code & 0x7f) << 8;
        words.push(Id14NormalizedWord(tag | u16::from(run_count)));
    }

    for row_index in row_count..output_count.as_usize() {
        words.push(Id14NormalizedWord(0x8000 | (row_index as u16 & 0x00ff)));
    }

    Ok(Id14NormalizedRows {
        input_count,
        output_count,
        words,
    })
}

fn classifier_code(value: i8) -> Option<u8> {
    Some(match value {
        0 => 1,
        1 => 2,
        2 => 3,
        3 => 4,
        4 => 11,
        5 => 12,
        6 => 5,
        7 => 6,
        8 => 7,
        9 => 8,
        12 => 9,
        13 => 10,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ID14_ROW_STRIDE_BYTES, Id14NormalizedWord, Id14RowCount, Id14RowNormalizationError,
        normalize_id14_six_byte_rows,
    };
    use serde_json::Value;

    const WASHED_INPUT_ONLY_VECTORS: &[u8] = include_bytes!(
        "../../../../outputs/IMPLEMENTER_SUPPLEMENTS/id14_row_normalizer_static_formula_v1/INPUT-ONLY-VECTORS.json"
    );

    #[test]
    fn washed_static_formula_vectors_match_the_six_byte_input_path() {
        let vectors: Value = serde_json::from_slice(WASHED_INPUT_ONLY_VECTORS).unwrap();
        let vectors = vectors["vectors"].as_array().unwrap();
        assert_eq!(vectors.len(), 29);

        for vector in vectors {
            let id = vector["id"].as_str().unwrap();
            let input = &vector["input_fields"];
            let row_count = usize::try_from(input["row_count"].as_u64().unwrap()).unwrap();
            let rows = input["rows"].as_array().unwrap();
            let mut row_bytes = vec![0_u8; rows.len() * ID14_ROW_STRIDE_BYTES];
            for (row_index, row) in rows.iter().enumerate() {
                let start = row_index * ID14_ROW_STRIDE_BYTES;
                let classifier = i8::try_from(row["byte_plus2_signed"].as_i64().unwrap()).unwrap();
                row_bytes[start + 2] = classifier.to_ne_bytes()[0];
                row_bytes[start + 3] = u8::try_from(row["byte_plus3"].as_u64().unwrap()).unwrap();
                let run_value = u16::try_from(row["u16_plus4"].as_u64().unwrap()).unwrap();
                row_bytes[start + 4..start + 6].copy_from_slice(&run_value.to_le_bytes());
            }

            let prior_state_count = if input["param1_state_is_null"].as_bool().unwrap() {
                None
            } else {
                Some(
                    Id14RowCount::new(
                        usize::try_from(input["prior_state_count"].as_u64().unwrap()).unwrap(),
                    )
                    .unwrap(),
                )
            };
            let result = normalize_id14_six_byte_rows(row_count, &row_bytes, prior_state_count);
            let expected_status = vector["expected_status"].as_str().unwrap();
            if expected_status == "RETURN_0" {
                let normalized = result.unwrap_or_else(|error| panic!("{id}: {error:?}"));
                let expected = vector["expected_output"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|word| {
                        u16::from_str_radix(word.as_str().unwrap().trim_start_matches("0x"), 16)
                            .unwrap()
                    })
                    .collect::<Vec<_>>();
                let actual = normalized
                    .words()
                    .iter()
                    .map(|word: &Id14NormalizedWord| word.value())
                    .collect::<Vec<_>>();
                assert_eq!(actual, expected, "{id}");
                assert_eq!(
                    usize::from(normalized.output_count().value()),
                    expected.len(),
                    "{id}"
                );
            } else {
                let error = result.expect_err(id);
                let expected_code = match expected_status {
                    "RETURN_5" => 5,
                    "RETURN_7" => 7,
                    _ => panic!("unexpected washed vector status {expected_status}"),
                };
                assert_eq!(error.contract_return_code(), Some(expected_code), "{id}");
            }
        }
    }

    #[test]
    fn row_buffer_extent_must_match_the_admitted_count() {
        assert_eq!(
            normalize_id14_six_byte_rows(1, &[0; ID14_ROW_STRIDE_BYTES - 1], None),
            Err(Id14RowNormalizationError::RowByteExtentMismatch {
                expected: ID14_ROW_STRIDE_BYTES,
                actual: ID14_ROW_STRIDE_BYTES - 1,
            })
        );
    }
}
