// pattern: Imperative Shell

//! Thin browser-facing orchestration around the in-memory JOCForge encode path.

mod ffi;

/// Largest ADM resource the browser target keeps resident.
///
/// A single encode holds one copy of the input beside the PCM working set, and wasm32 linear
/// memory doubles while it grows inside a 4 GiB address space. 512 MiB leaves comfortable room for
/// both; larger programmes are a native encode, not a browser one.
const MAX_RESIDENT_INPUT_BYTES: usize = 512 * 1024 * 1024;

use jocforge_pipeline::encode_request::EncodeRequestError;

/// Result categories stay small and stable at the browser boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FailureCategory {
    InvalidInput,
    Encode,
    Lifecycle,
    Internal,
}

/// Non-error result of a browser bridge operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncoderStatus {
    NeedMoreInput,
    BitstreamAvailable,
    EndOfStream,
    Error,
}

/// Bounded user-facing bridge error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EncoderError {
    pub category: FailureCategory,
    pub detail: String,
}

/// In-memory ADM BWF to elementary E-AC-3.
///
/// Callers push the complete RIFF resource, then finish. Paths are not accepted.
pub struct Encoder {
    input: Vec<u8>,
    output: Option<Vec<u8>>,
    finished: bool,
    last_error: Option<EncoderError>,
    rate_kbps: u32,
    object_count: u8,
    job_xml: Option<Vec<u8>>,
    layout: u8,
    dependent_count: Option<u8>,
    surround_ex: u8,
}

impl Encoder {
    pub fn new() -> Result<Self, EncoderError> {
        Ok(Self {
            input: Vec::new(),
            output: None,
            finished: false,
            last_error: None,
            rate_kbps: 1024,
            object_count: 15,
            job_xml: None,
            layout: 0,
            dependent_count: None,
            surround_ex: 1,
        })
    }

    /// Appends ADM BWF bytes. Encoding starts only in `finish`.
    pub fn push_bytes(&mut self, bytes: &[u8]) -> EncoderStatus {
        if self.finished {
            return EncoderStatus::EndOfStream;
        }
        if self.output.is_some() {
            return EncoderStatus::BitstreamAvailable;
        }
        // The browser target keeps the whole resource resident beside the working set, and wasm32
        // memory doubles as it grows. Refusing an oversized resource here is the only way to fail
        // cleanly: an allocation trap would poison the module for the rest of the page.
        if self
            .input
            .len()
            .checked_add(bytes.len())
            .is_none_or(|total| total > MAX_RESIDENT_INPUT_BYTES)
        {
            return self.fail(EncoderError {
                category: FailureCategory::InvalidInput,
                detail: format!(
                    "ADM input exceeds the browser encode budget of {} bytes; encode it natively or trim it first",
                    MAX_RESIDENT_INPUT_BYTES
                ),
            });
        }
        if !bytes.is_empty() && self.input.try_reserve(bytes.len()).is_err() {
            return self.fail(EncoderError {
                category: FailureCategory::InvalidInput,
                detail: "ADM input exceeds the bounded bridge allocation".to_owned(),
            });
        }
        self.input.extend_from_slice(bytes);
        EncoderStatus::NeedMoreInput
    }

    /// Selects the carrier bitrate for the next finish.
    ///
    /// Accepted values are the six selected 5.1 rates; anything else is rejected at finish.
    pub fn set_rate_kbps(&mut self, rate_kbps: u32) {
        self.rate_kbps = rate_kbps;
    }

    /// Selects the coded JOC object count (8, 12 or 15).
    pub fn set_object_count(&mut self, object_count: u8) {
        self.object_count = object_count;
    }

    /// Requests a composed (multi-substream) programme with the given dependent-frame count.
    ///
    /// The plan is built by the pipeline from this count and the selected layout, so the bridge
    /// never carries a topology type across the crate boundary.
    pub fn set_dependent_count(&mut self, dependent_count: u8) {
        self.dependent_count = Some(dependent_count);
    }

    /// Selects the reconstruction layout: 0 = selected 5.1, 1 = flat 7.1.
    pub fn set_layout(&mut self, layout: u8) {
        self.layout = layout;
    }

    /// Selects the native flat-7.1 Surround EX declaration: 0 = unspecified,
    /// 1 = not encoded (default), 2 = encoded.
    ///
    /// Modes 0 and 2 write the two declared BSI suffix bits, so the pipeline only
    /// accepts them for flat 7.1 output without a composed topology.
    pub fn set_surround_ex(&mut self, surround_ex: u8) {
        self.surround_ex = surround_ex;
    }

    /// Supplies a caller-authored job XML for the next finish.
    ///
    /// The XML carries the sound parameters (DRC, downmix, metering, trims) and timing. Its
    /// `<data_rate>` must match the selected rate, because the pipeline cross-checks the
    /// prepared configuration against it.
    pub fn set_job_xml(&mut self, xml: Vec<u8>) {
        self.job_xml = Some(xml);
    }

    /// Encodes the pushed ADM bytes and exposes one elementary stream.
    pub fn finish(&mut self) -> Result<EncoderStatus, EncoderError> {
        if self.finished {
            return Ok(if self.output.is_some() {
                EncoderStatus::BitstreamAvailable
            } else {
                EncoderStatus::EndOfStream
            });
        }
        if self.input.is_empty() {
            let error = EncoderError {
                category: FailureCategory::InvalidInput,
                detail: "empty ADM BWF".to_owned(),
            };
            self.fail(error.clone());
            return Err(error);
        }
        // The browser hands the pushed buffer over; native test builds keep borrowing it.
        #[cfg(target_arch = "wasm32")]
        let outcome = jocforge_pipeline::encode_request::encode_adm_resource(
            std::sync::Arc::from(std::mem::take(&mut self.input).into_boxed_slice()),
            self.rate_kbps,
            self.object_count,
            self.job_xml.take(),
            self.layout,
            self.dependent_count,
            self.surround_ex,
        )
        .map_err(map_encode_request_error);
        #[cfg(not(target_arch = "wasm32"))]
        let outcome = encode_adm(&self.input);
        match outcome {
            Ok(bytes) => {
                self.output = Some(bytes);
                self.finished = true;
                Ok(EncoderStatus::BitstreamAvailable)
            }
            Err(error) => {
                self.fail(error.clone());
                Err(error)
            }
        }
    }

    /// Returns the owned elementary stream once.
    pub fn receive_bitstream(&mut self) -> Option<Vec<u8>> {
        self.output.take()
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.output = None;
        self.finished = false;
        self.last_error = None;
    }

    fn fail(&mut self, error: EncoderError) -> EncoderStatus {
        self.last_error = Some(error);
        self.finished = true;
        EncoderStatus::Error
    }

    pub(crate) fn mark_internal_failure(&mut self) {
        self.fail(EncoderError {
            category: FailureCategory::Internal,
            detail: "JOCForge WASM bridge trapped while encoding".to_owned(),
        });
    }

    pub(crate) fn last_error(&self) -> Option<&EncoderError> {
        self.last_error.as_ref()
    }

    pub(crate) fn error_category_code(&self) -> Option<u32> {
        self.last_error.as_ref().map(|error| match error.category {
            FailureCategory::InvalidInput => 0,
            FailureCategory::Encode => 1,
            FailureCategory::Lifecycle => 2,
            FailureCategory::Internal => 3,
        })
    }

    pub(crate) fn expose_bitstream(&mut self) -> Option<&[u8]> {
        self.output.as_deref()
    }
}

/// Encodes one complete ADM BWF resource already resident in memory.
///
/// The bridge stays thin: input preparation, admission and the encode loop all live in
/// `jocforge-pipeline`, so the browser and the CLI cannot drift into two different preparations.
pub fn encode_adm(adm: &[u8]) -> Result<Vec<u8>, EncoderError> {
    jocforge_pipeline::encode_request::encode_adm_bytes(adm).map_err(map_encode_request_error)
}

/// Classifies one pipeline failure for the browser boundary.
///
/// Anything the caller can fix by changing the resource or its request is `InvalidInput`; the
/// rest is a real encode failure.
fn map_encode_request_error(error: EncodeRequestError) -> EncoderError {
    let category = match &error {
        EncodeRequestError::Riff(_)
        | EncodeRequestError::InputFile(_)
        | EncodeRequestError::InvalidPath
        | EncodeRequestError::PreparedInput(_)
        | EncodeRequestError::JobXmlInput(_)
        | EncodeRequestError::JobXmlIngress(_)
        | EncodeRequestError::InvalidJobXmlReference
        | EncodeRequestError::UnsupportedJobXmlOption(_)
        | EncodeRequestError::OutputAlreadyExists
        | EncodeRequestError::UnsupportedDataRate
        | EncodeRequestError::UnsupportedJocLayout
        | EncodeRequestError::ExperimentalCandidatePathRequired
        | EncodeRequestError::ExperimentalCandidateDirectoryNotEmpty
        | EncodeRequestError::InputExtent => FailureCategory::InvalidInput,
        _ => FailureCategory::Encode,
    };
    EncoderError {
        category,
        detail: bound_detail(&error.to_string()),
    }
}

fn bound_detail(detail: &str) -> String {
    detail.chars().take(256).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use jocforge_io::pcm::riff_pcm24::parse_selected_riff_pcm24;

    fn short_adm_excerpt(
        bytes: &[u8],
        data_offset: u64,
        old_payload_bytes: usize,
        block_align: u16,
        frames: usize,
    ) -> Vec<u8> {
        let block = usize::from(block_align);
        let data_off = usize::try_from(data_offset).expect("data offset");
        let new_len = frames * block;
        assert!(new_len <= old_payload_bytes);
        let mut out = bytes[..data_off].to_vec();
        out[data_off - 4..data_off]
            .copy_from_slice(&u32::try_from(new_len).expect("data size").to_le_bytes());
        out.extend_from_slice(&bytes[data_off..data_off + new_len]);
        if new_len % 2 == 1 {
            out.push(0);
        }
        let tail = data_off + old_payload_bytes + (old_payload_bytes % 2);
        out.extend_from_slice(&bytes[tail..]);
        let riff_size = u32::try_from(out.len() - 8).expect("riff size");
        out[4..8].copy_from_slice(&riff_size.to_le_bytes());
        out
    }

    #[test]
    fn empty_input_is_rejected_before_encode() {
        let mut encoder = Encoder::new().expect("encoder");
        let error = encoder.finish().expect_err("empty");
        assert_eq!(error.category, FailureCategory::InvalidInput);
        assert_eq!(encoder.push_bytes(&[]), EncoderStatus::EndOfStream);
    }

    #[test]
    fn pale_ride_adm_bridge_matches_direct_result() {
        let Ok(path) = std::env::var("JOCFORGE_ADM_PARITY") else {
            return;
        };
        if path.is_empty() || path == "1" {
            return;
        }
        let bytes = match std::fs::read(&path) {
            Ok(bytes) => bytes,
            Err(error) => panic!("ADM parity file: {error}"),
        };
        let parsed = parse_selected_riff_pcm24(&bytes, None).expect("RIFF");
        assert_eq!(parsed.sample_rate_hz(), 48_000);
        assert_eq!(parsed.channels(), 52);
        assert_eq!(parsed.block_align(), 156);
        let excerpt = short_adm_excerpt(
            &bytes,
            parsed.data_offset(),
            parsed.data_payload_bytes(),
            parsed.block_align(),
            12_288,
        );
        let first = encode_adm(&excerpt).expect("excerpt encode");
        let second = encode_adm(&excerpt).expect("excerpt encode repeat");
        assert!(!first.is_empty());
        assert_eq!(first, second);
        let mut encoder = Encoder::new().expect("encoder");
        assert_eq!(encoder.push_bytes(&excerpt), EncoderStatus::NeedMoreInput);
        assert_eq!(
            encoder.finish().expect("bridge"),
            EncoderStatus::BitstreamAvailable
        );
        assert_eq!(encoder.receive_bitstream().expect("bitstream"), first);
    }

    #[test]
    fn dumps_excerpt_and_native_bitstream_for_cross_target_parity() {
        let (Ok(path), Ok(directory)) = (
            std::env::var("JOCFORGE_ADM_PARITY"),
            std::env::var("JOCFORGE_ADM_DUMP"),
        ) else {
            return;
        };
        if path.is_empty() || path == "1" || directory.is_empty() {
            return;
        }
        let bytes = std::fs::read(&path).expect("ADM parity file");
        let parsed = parse_selected_riff_pcm24(&bytes, None).expect("RIFF");
        let excerpt = short_adm_excerpt(
            &bytes,
            parsed.data_offset(),
            parsed.data_payload_bytes(),
            parsed.block_align(),
            12_288,
        );
        let product = encode_adm(&excerpt).expect("excerpt encode");
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).expect("dump directory");
        std::fs::write(directory.join("excerpt.wav"), &excerpt).expect("excerpt dump");
        std::fs::write(directory.join("native.ec3"), &product).expect("native dump");
        eprintln!(
            "cross-target dump: excerpt={} native_bitstream={}",
            excerpt.len(),
            product.len()
        );
    }

    #[test]
    fn bridge_matches_direct_encode_on_truncated_riff() {
        let bytes = b"RIFF\x00\x00\x00\x00WAVE";
        let direct = encode_adm(bytes);
        let mut encoder = Encoder::new().expect("encoder");
        assert_eq!(encoder.push_bytes(bytes), EncoderStatus::NeedMoreInput);
        let bridged = encoder.finish();
        assert_eq!(bridged.err(), direct.err());
    }
}
