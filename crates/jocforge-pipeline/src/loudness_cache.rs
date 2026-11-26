// pattern: Imperative Shell
//! Shares completed ADM loudness measurements between file encodes on this computer.

use crate::{
    encode_request::{EncodeEvent, EncodePhase, EncodeRequestError, EventObserver},
    meter_prepass::{PROVIDER_ASSETS, run_meter_with_providers},
    progress::ProgressBar,
};
#[cfg(not(target_arch = "wasm32"))]
use fs2::FileExt;
use jocforge_application::{
    PreparedJobInput,
    meter_finalization_bridge::{
        MeasurementDependentOwnerInputs, MeterOwnerStateReference, ProgrammeMeasurementValue,
        reuse_programme_measurement,
    },
    output_publication::AttemptId,
};
use jocforge_io::pcm::riff_pcm24::ParsedRiffPcm24;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    sync::{
        LazyLock,
        atomic::{AtomicBool, Ordering},
    },
};
#[cfg(not(target_arch = "wasm32"))]
use std::{io::Write, time::Duration};

// Bump when a meter DSP change alters any finalized measurement word.
const METER_REVISION: &str = "adm-loudness-v1";
type AcceptedMeasurement = (MeterOwnerStateReference, MeasurementDependentOwnerInputs);
type Progress<'a> = Option<EventObserver<'a>>;

static PARAMETER_HASH: LazyLock<[u8; 32]> = LazyLock::new(|| {
    let mut assets = PROVIDER_ASSETS.to_vec();
    assets.sort_unstable_by_key(|(path, _)| *path);
    let mut digest = Sha256::new();
    for (name, bytes) in assets {
        digest.update(name.replace('\\', "/").as_bytes());
        digest.update([0]);
        digest.update((bytes.len() as u64).to_le_bytes());
        digest.update(bytes);
    }
    digest.finalize().into()
});

fn cache_key(prepared: &PreparedJobInput) -> [u8; 32] {
    let identity = (
        METER_REVISION,
        env!("CARGO_PKG_VERSION"),
        env::consts::ARCH,
        env::consts::OS,
        *PARAMETER_HASH,
        prepared.resource_identity().input_sha256(),
        prepared.configuration().measurement_settings_identity(),
    );
    Sha256::digest(serde_json::to_vec(&identity).expect("meter key contains only scalar data"))
        .into()
}

fn directory() -> Option<PathBuf> {
    let variable = |name| {
        env::var_os(name)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
    };
    let base = if let Some(root) = env::var_os("JOCFORGE_CACHE_DIR").filter(|v| !v.is_empty()) {
        std::path::absolute(root).ok()?
    } else {
        let user_cache = if cfg!(windows) {
            variable("LOCALAPPDATA")
        } else if cfg!(target_os = "macos") {
            variable("HOME").map(|p| p.join("Library/Caches"))
        } else {
            variable("XDG_CACHE_HOME").or_else(|| variable("HOME").map(|p| p.join(".cache")))
        };
        user_cache?.join("JOCForge")
    };
    Some(base.join(METER_REVISION))
}

#[derive(Serialize, Deserialize)]
struct MeasuredProgramme {
    key: [u8; 32],
    terminal: [u8; 32],
    cold: [u8; 32],
    measurement: ProgrammeMeasurementValue,
}

#[derive(Serialize, Deserialize)]
struct CacheFile {
    result: MeasuredProgramme,
    checksum: [u8; 32],
}

impl MeasuredProgramme {
    fn checksum(&self) -> [u8; 32] {
        Sha256::digest(serde_json::to_vec(self).expect("measurement contains only scalar data"))
            .into()
    }
}

fn load(
    path: &Path,
    key: [u8; 32],
    prepared: &PreparedJobInput,
    attempt: AttemptId,
) -> Option<AcceptedMeasurement> {
    let mut bytes = Vec::new();
    fs::File::open(path)
        .ok()?
        .take(4097)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() > 4096 {
        return None;
    }
    let CacheFile { result, checksum } = serde_json::from_slice(&bytes).ok()?;
    if result.key != key || result.checksum() != checksum {
        return None;
    }
    reuse_programme_measurement(
        prepared,
        attempt,
        key,
        result.terminal,
        result.cold,
        &result.measurement,
    )
    .ok()
}

fn store(path: &Path, result: MeasuredProgramme) -> io::Result<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, result);
        return Ok(());
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap_or(Path::new(".")))?;
        serde_json::to_writer(
            &mut temp,
            &CacheFile {
                checksum: result.checksum(),
                result,
            },
        )?;
        temp.flush()?;
        temp.persist(path)?;
        Ok(())
    }
}

fn lock(path: &Path, cancelled: &AtomicBool, progress: Progress<'_>) -> io::Result<fs::File> {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (path, cancelled, progress);
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "browser encode does not lock a loudness cache",
        ));
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        let mut waiting = false;
        loop {
            if cancelled.load(Ordering::Relaxed) {
                return Err(io::ErrorKind::Interrupted.into());
            }
            match FileExt::try_lock_exclusive(&file) {
                Ok(()) => return Ok(file),
                Err(ref error)
                    if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() =>
                {
                    if !waiting {
                        eprintln!("Waiting for a matching loudness measurement");
                        if let Some(report) = progress {
                            report(EncodeEvent::WaitingForLoudness);
                        }
                        waiting = true;
                    }
                    std::thread::sleep(Duration::from_millis(80));
                }
                Err(error) => return Err(error),
            }
        }
    }
}

pub(crate) fn measure_for_encode(
    prepared: &PreparedJobInput,
    parsed: &ParsedRiffPcm24<'_>,
    attempt: AttemptId,
    supplements: jocforge_io::provider::ProviderTree<'_>,
    progress: Progress<'_>,
    cancelled: &AtomicBool,
) -> Result<AcceptedMeasurement, EncodeRequestError> {
    let key = cache_key(prepared);
    let path =
        directory().map(|dir| dir.join(format!("{}.json", crate::encode_request::hex_lower(&key))));
    let hit = |path: &Path| {
        let result = load(path, key, prepared, attempt)?;
        eprintln!("Loudness cache hit; reusing completed programme measurement");
        if let Some(report) = progress {
            report(EncodeEvent::CachedLoudness);
        }
        Some(result)
    };
    if cancelled.load(Ordering::Relaxed) {
        return Err(EncodeRequestError::Cancelled);
    }
    let mut locked = None;
    if let Some(path) = &path {
        if let Some(result) = hit(path) {
            return Ok(result);
        }
        // Recheck after acquiring the per-key OS lock. A concurrent job may have just published.
        let acquire = || {
            fs::create_dir_all(path.parent().expect("entry is in the cache directory"))?;
            lock(&path.with_extension("lock"), cancelled, progress)
        };
        match acquire() {
            Ok(file) => {
                locked = Some(file);
                if let Some(result) = hit(path) {
                    return Ok(result);
                }
            }
            Err(_) if cancelled.load(Ordering::Relaxed) => {
                return Err(EncodeRequestError::Cancelled);
            }
            Err(error) => eprintln!("Loudness cache unavailable: {error}"),
        }
    }
    eprintln!("Starting full-file Meter prepass...");
    let mut bar = ProgressBar::new("Meter prepass");
    let pass = run_meter_with_providers(
        prepared,
        parsed,
        attempt,
        supplements,
        |done, total| {
            if let Some(report) = progress {
                report(EncodeEvent::Progress(EncodePhase::Metering, done, total));
            } else {
                bar.update(done, total);
            }
        },
        cancelled,
    )
    .map_err(|error| {
        if cancelled.load(Ordering::Relaxed) {
            EncodeRequestError::Cancelled
        } else {
            EncodeRequestError::MeterPrepass(error)
        }
    })?;
    bar.finish();
    if cancelled.load(Ordering::Relaxed) {
        return Err(EncodeRequestError::Cancelled);
    }
    let reference =
        MeterOwnerStateReference::from_root(pass.terminal_meter_root()).map_err(|error| {
            EncodeRequestError::SourcePipeline(format!("terminal measurement: {error:?}"))
        })?;
    if let (Some(path), Some(_lock)) = (path, locked) {
        let result = MeasuredProgramme {
            key,
            terminal: *reference.state_sha256(),
            cold: *reference.cold_sha256(),
            measurement: pass
                .candidate()
                .terminal_measurement()
                .measurement()
                .clone(),
        };
        if let Err(error) = store(&path, result) {
            eprintln!("Could not save loudness cache: {error}");
        }
    }
    Ok((reference, pass.into_dependents()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waiting_for_another_measurement_can_be_cancelled() {
        let cache = tempfile::tempdir().unwrap();
        let path = cache.path().join("programme.lock");
        let cancelled = AtomicBool::new(false);
        let first = lock(&path, &cancelled, None).unwrap();
        let on_wait = |event| {
            assert!(matches!(event, EncodeEvent::WaitingForLoudness));
            cancelled.store(true, Ordering::Relaxed);
        };
        assert_eq!(
            lock(&path, &cancelled, Some(&on_wait)).unwrap_err().kind(),
            io::ErrorKind::Interrupted
        );
        drop(first);
        assert!(lock(&path, &AtomicBool::new(false), None).is_ok());
    }
}
