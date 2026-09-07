// pattern: Imperative Shell

// pattern: Imperative Shell

use std::{
    collections::BTreeMap,
    env,
    fmt::Write as _,
    fs,
    io::Write as _,
    path::{Path, PathBuf},
    process::ExitCode,
};

mod file_scan;

use file_scan::{read_pcm_frame, scan_bw64_file};

use jocforge_core::{
    AdmObject, AdmObjectKind, FinalRenderMeasurement, JocCarriage, JocProfile, ReductionConfig,
    ReductionSource, SOURCE_DERIVED_CARRIER_PEAK_LIMIT, d0_carrier_causality,
    factorize_source_derived, final_render_gate, generate_synthetic_scene_fixture,
    limit_signal_peak, maximum_absolute_difference, package_deployed_compat_access_unit,
    package_source_derived_access_unit_with_frame_sizes, parse_bw64_adm_metadata, profile_spec,
    quantize_coefficient, rate_budget_for_carriage, reconstruction_error,
    reduce_scene_with_previous, source_derived_carrier_causality, spectral_phase_report,
    validate_source_derived_adm_metadata, write_one_au_proof,
};

const JOC_FRAME_SAMPLES: usize = 1536;

fn main() -> ExitCode {
    let arguments = env::args().skip(1).collect::<Vec<_>>();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("jocforge: {error}");
            ExitCode::from(2)
        }
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    match arguments.first().map(String::as_str) {
        Some("proof") => run_proof(&arguments[1..]),
        Some("fixture") => run_fixture(&arguments[1..]),
        Some("fixture-directspeakers") => run_direct_speaker_fixture(&arguments[1..]),
        Some("fixture-unsupported") => run_unsupported_fixture(&arguments[1..]),
        Some("inspect-adm") => run_inspect_adm(&arguments[1..]),
        Some("encode") => run_encode(&arguments[1..]),
        Some("quantization-stress") => run_quantization_stress(&arguments[1..]),
        Some("carrier-causality") => run_carrier_causality(&arguments[1..]),
        Some("spectral-phase") => run_spectral_phase(&arguments[1..]),
        Some("final-render-gate") => run_final_render_gate(&arguments[1..]),
        Some("temporal") => run_temporal(&arguments[1..]),
        Some("temporal-adversarial") => run_temporal_adversarial(&arguments[1..]),
        Some(command) => Err(format!("unsupported command '{command}'")),
        None => Err(
            "usage: jocforge proof|fixture|fixture-directspeakers|fixture-unsupported|inspect-adm|encode|quantization-stress|carrier-causality|spectral-phase|final-render-gate|temporal ..."
                .to_owned(),
        ),
    }
}

fn run_temporal(arguments: &[String]) -> Result<(), String> {
    let [output] = arguments else {
        return Err("usage: jocforge temporal OUTPUT.csv".to_owned());
    };
    write_temporal_metrics(
        &PathBuf::from(output),
        jocforge_core::temporal_corpus(),
        ReductionConfig::default(),
    )
}

fn run_temporal_adversarial(arguments: &[String]) -> Result<(), String> {
    let (output, config) = match arguments {
        [output] => (output.as_str(), ReductionConfig::default()),
        [flag, output] if flag == "--no-continuity" => (
            output.as_str(),
            ReductionConfig {
                temporal_continuity_weight: 0.0,
                ..ReductionConfig::default()
            },
        ),
        _ => {
            return Err(
                "usage: jocforge temporal-adversarial [--no-continuity] OUTPUT.csv".to_owned(),
            );
        }
    };
    write_temporal_metrics(
        &PathBuf::from(output),
        jocforge_core::adversarial_temporal_corpus(),
        config,
    )
}

fn write_temporal_metrics(
    output: &Path,
    scenes: Vec<jocforge_core::TemporalScene>,
    config: ReductionConfig,
) -> Result<(), String> {
    let mut csv = String::from(
        "scene,frame_count,authored_source_count,maximum_coded_object_count,silent_object_drops,maximum_source_reconstruction_error,maximum_spatial_representation_error,maximum_cross_object_leakage,minimum_headroom_margin,total_authored_source_assignment_changes,total_representative_source_changes,total_coded_slot_replacements,authored_source_assignment_changes_per_minute,max_sliding_window_authored_source_assignment_changes,minimum_authored_source_assignment_lifetime_frames,median_authored_source_assignment_lifetime_frames,minimum_representative_source_lifetime_frames,median_representative_source_lifetime_frames,minimum_coded_slot_lifetime_frames,median_coded_slot_lifetime_frames,authored_source_assignment_reassignment_burstiness,maximum_position_jump,maximum_trajectory_error,maximum_gain_jump,maximum_pcm_boundary_discontinuity,maximum_reconstruction_error_change,maximum_speaker_render_discontinuity,stationary_authored_source_assignment_churn_after_initialization\n",
    );
    for scene in scenes {
        let metrics = jocforge_core::analyze_temporal_scene_with_config(&scene, config)?;
        let _ = writeln!(
            csv,
            "{},{},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e},{},{},{:.17e},{},{:.17e},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{}",
            metrics.scene,
            metrics.frame_count,
            metrics.authored_source_count,
            metrics.maximum_coded_object_count,
            metrics.silent_object_drops,
            metrics.maximum_source_reconstruction_error,
            metrics.maximum_spatial_representation_error,
            metrics.maximum_cross_object_leakage,
            metrics.minimum_headroom_margin,
            metrics.total_authored_source_assignment_changes,
            metrics.total_representative_source_changes,
            metrics.total_coded_slot_replacements,
            metrics.authored_source_assignment_changes_per_minute,
            metrics.max_sliding_window_authored_source_assignment_changes,
            metrics.minimum_authored_source_assignment_lifetime_frames,
            metrics.median_authored_source_assignment_lifetime_frames,
            metrics.minimum_representative_source_lifetime_frames,
            metrics.median_representative_source_lifetime_frames,
            metrics.minimum_coded_slot_lifetime_frames,
            metrics.median_coded_slot_lifetime_frames,
            metrics.authored_source_assignment_reassignment_burstiness,
            metrics.maximum_position_jump,
            metrics.maximum_trajectory_error,
            metrics.maximum_gain_jump,
            metrics.maximum_pcm_boundary_discontinuity,
            metrics.maximum_reconstruction_error_change,
            metrics.maximum_speaker_render_discontinuity,
            metrics.stationary_authored_source_assignment_churn_after_initialization,
        );
    }
    write_output(output, csv.into_bytes(), "temporal metrics")
}

fn run_proof(arguments: &[String]) -> Result<(), String> {
    let [profile, output] = arguments else {
        return Err("usage: jocforge proof idx0|idx1|idx2 OUTPUT.ec3".to_owned());
    };
    let profile = parse_profile(profile)?;
    let bytes = write_one_au_proof(profile)?;
    let output = PathBuf::from(output);
    write_output(&output, bytes, "proof")
}

fn run_quantization_stress(arguments: &[String]) -> Result<(), String> {
    let [output] = arguments else {
        return Err("usage: jocforge quantization-stress OUTPUT.json".to_owned());
    };
    let quantum = 820.0 / 4096.0;
    let profiles = [JocProfile::Idx0, JocProfile::Idx1, JocProfile::Idx2];
    let mut max_error = 0.0_f64;
    let mut pass = true;
    let mut matrix_values_checked = 0_usize;
    for profile in profiles {
        let columns = profile_spec(profile).input_labels.len();
        for object_count in 1..=15 {
            for object in 0..object_count {
                for column in 0..columns {
                    let step = i32::try_from((object * columns + column) % 96)
                        .map_err(|_| "quantization stress index overflow".to_owned())?
                        - 48;
                    let coefficient = quantize_coefficient(f64::from(step) * quantum)
                        .map_err(|error| format!("quantization stress failed: {error}"))?;
                    let expected_symbol = step.rem_euclid(96) as u16;
                    pass &= coefficient.encoded_symbol == expected_symbol;
                    pass &= coefficient.decoded.is_finite() && coefficient.error.is_finite();
                    max_error = max_error.max(coefficient.error.abs());
                    matrix_values_checked += 1;
                }
            }
        }
    }
    for step in -48_i32..=47 {
        let coefficient = quantize_coefficient(f64::from(step) * quantum)
            .map_err(|error| format!("quantization domain stress failed: {error}"))?;
        pass &= coefficient.encoded_symbol == step.rem_euclid(96) as u16;
        pass &= coefficient.decoded.is_finite() && coefficient.error.is_finite();
        max_error = max_error.max(coefficient.error.abs());
    }
    pass &= max_error <= quantum * 0.5 + 1.0e-12;
    let json = format!(
        "{{\n  \"pass\": {},\n  \"profile_count\": 3,\n  \"profiles\": [\"idx0\", \"idx1\", \"idx2\"],\n  \"domain_min_step\": -48,\n  \"domain_max_step\": 47,\n  \"domain_values_checked\": 96,\n  \"matrix_values_checked\": {matrix_values_checked},\n  \"quantum\": {quantum:.17e},\n  \"maximum_absolute_error\": {max_error:.17e}\n}}\n",
        json_bool(pass),
    );
    write_output(
        &PathBuf::from(output),
        json.into_bytes(),
        "quantization stress report",
    )?;
    if pass {
        Ok(())
    } else {
        Err("quantization stress gate failed".to_owned())
    }
}

fn run_carrier_causality(arguments: &[String]) -> Result<(), String> {
    let [output] = arguments else {
        return Err("usage: jocforge carrier-causality OUTPUT.json".to_owned());
    };
    let profile_specs = [("idx0", 5_usize), ("idx1", 7_usize), ("idx2", 7_usize)];
    let mut profile_json = Vec::with_capacity(profile_specs.len());
    let mut pass = true;
    for (profile, carrier_count) in profile_specs {
        let measurements = source_derived_carrier_causality(carrier_count)
            .map_err(|error| format!("carrier causality fixture failed for {profile}: {error}"))?;
        let profile_pass = measurements.iter().all(|measurement| {
            measurement.carrier_index == measurement.source_index
                && measurement.carrier_changed
                && measurement.maximum_change > 1.0e-12
        });
        let joc_profile = parse_profile(profile)?;
        let d0_measurements = d0_carrier_causality(joc_profile)
            .map_err(|error| format!("D0 causality fixture failed for {profile}: {error}"))?;
        let d0_pass = d0_measurements
            .iter()
            .all(|measurement| measurement.changed && measurement.changed_byte_count > 0);
        let d0_source_indices = profile_spec(joc_profile).d0_source_indices;
        pass &= measurements.len() == carrier_count && profile_pass && d0_pass;
        let rows = measurements
            .iter()
            .map(|measurement| {
                format!(
                    "{{\"carrier_index\":{},\"source_index\":{},\"maximum_change\":{:.17e},\"carrier_changed\":{}}}",
                    measurement.carrier_index,
                    measurement.source_index,
                    measurement.maximum_change,
                    json_bool(measurement.carrier_changed),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        let d0_rows = d0_measurements
            .iter()
            .map(|measurement| {
                format!(
                    "{{\"input_index\":{},\"changed_byte_count\":{},\"changed\":{}}}",
                    measurement.input_index,
                    measurement.changed_byte_count,
                    json_bool(measurement.changed),
                )
            })
            .collect::<Vec<_>>()
            .join(",");
        profile_json.push(format!(
            "{{\"profile\":\"{profile}\",\"expected_carrier_count\":{carrier_count},\"pass\":{},\"carriers\":[{rows}],\"d0_input_source_indices\":[{},{}],\"d0_writer_pass\":{},\"d0_writer\":[{d0_rows}]}}",
            json_bool(profile_pass && d0_pass),
            d0_source_indices[0],
            d0_source_indices[1],
            json_bool(d0_pass),
        ));
    }
    let json = format!(
        "{{\n  \"fixture\": \"deterministic source perturbation\",\n  \"pass\": {},\n  \"profiles\": [{}]\n}}\n",
        json_bool(pass),
        profile_json.join(","),
    );
    write_output(
        &PathBuf::from(output),
        json.into_bytes(),
        "carrier causality report",
    )?;
    if pass {
        Ok(())
    } else {
        Err("source-derived carrier causality gate failed".to_owned())
    }
}

fn run_spectral_phase(arguments: &[String]) -> Result<(), String> {
    let (output, final_render_report_path) = match arguments {
        [output] => (output.as_str(), None),
        [output, flag, report] if flag == "--final-render-report" => {
            (output.as_str(), Some(report.as_str()))
        }
        _ => {
            return Err(
                "usage: jocforge spectral-phase OUTPUT.json [--final-render-report REPORT.json]"
                    .to_owned(),
            );
        }
    };
    let report = spectral_phase_report();
    let (decoded_eac3_stage, openjoc_stage, final_render_evidence) =
        if let Some(path) = final_render_report_path {
            let evidence = fs::read_to_string(path).map_err(|error| {
                format!("failed to read final decoded render evidence '{path}': {error}")
            })?;
            let evidence = evidence
                .strip_prefix('\u{feff}')
                .unwrap_or(&evidence)
                .to_owned();
            let evidence_value = parse_json_document(&evidence).map_err(|error| {
                format!("failed to parse final decoded render evidence '{path}': {error}")
            })?;
            if !final_render_evidence_is_valid(&evidence_value) {
                return Err(
                    "final decoded render evidence is missing a passing FINAL_DECODED_RENDER stage"
                        .to_owned(),
                );
            }
            ("FINAL_DECODED_RENDER", "FINAL_DECODED_RENDER", evidence)
        } else {
            ("NOT_AVAILABLE", "NOT_AVAILABLE", String::from("null"))
        };
    let metrics = report
        .metrics
        .iter()
        .map(|metric| {
            format!(
                "{{\"name\":\"{}\",\"relationship\":\"{}\",\"hard_gate\":{},\"magnitude_spectrum_deviation\":{:.17e},\"band_energy_error\":{:.17e},\"spectral_tilt_error\":{:.17e},\"transient_spectral_retention\":{:.17e},\"low_frequency_retention\":{:.17e},\"high_frequency_retention\":{:.17e},\"phase_difference\":{:.17e},\"normalized_correlation\":{:.17e},\"polarity_preserved\":{},\"quadrature_phase_error\":{:.17e},\"transient_boundary_discontinuity\":{:.17e},\"stereo_width_error\":{:.17e},\"reduction_coded_objects\":{},\"reduction_pass\":{},\"pass\":{}}}",
                metric.name,
                metric.relationship,
                json_bool(metric.gate_scope.is_hard()),
                metric.magnitude_spectrum_deviation,
                metric.band_energy_error,
                metric.spectral_tilt_error,
                metric.transient_spectral_retention,
                metric.low_frequency_retention,
                metric.high_frequency_retention,
                metric.phase_difference,
                metric.normalized_correlation,
                json_bool(metric.polarity_preserved.is_pass()),
                metric.quadrature_phase_error,
                metric.transient_boundary_discontinuity,
                metric.stereo_width_error,
                metric.reduction_coded_objects,
                json_bool(metric.reduction_pass.is_pass()),
                json_bool(metric.pass.is_pass()),
            )
        })
        .collect::<Vec<_>>()
        .join(",\n    ");
    let json = format!(
        "{{\n  \"pass\": {},\n  \"fixture_count\": {},\n  \"reference_stage\": \"authored deterministic PCM\",\n  \"compatibility_stage\": \"clean pre-E-AC-3 compatibility presentation\",\n  \"decoded_eac3_stage\": \"{}\",\n  \"openjoc_stage\": \"{}\",\n  \"final_decoded_render_evidence\": {},\n  \"fft_size\": 256,\n  \"window\": \"periodic Hann over one 256-sample window\",\n  \"sample_rate\": 48000,\n  \"spectral_gate\": {},\n  \"phase_gate\": {},\n  \"polarity_gate\": {},\n  \"quadrature_gate\": {},\n  \"transient_gate\": {},\n  \"metrics\": [\n    {}\n  ]\n}}\n",
        json_bool(report.pass.is_pass()),
        report.fixture_count,
        decoded_eac3_stage,
        openjoc_stage,
        final_render_evidence,
        json_bool(report.spectral_gate.is_pass()),
        json_bool(report.phase_gate.is_pass()),
        json_bool(report.polarity_gate.is_pass()),
        json_bool(report.quadrature_gate.is_pass()),
        json_bool(report.transient_gate.is_pass()),
        metrics,
    );
    write_output(
        &PathBuf::from(output),
        json.into_bytes(),
        "spectral phase report",
    )?;
    if report.pass.is_pass() {
        Ok(())
    } else {
        Err("spectral/phase compatibility gate failed".to_owned())
    }
}

fn run_final_render_gate(arguments: &[String]) -> Result<(), String> {
    let [
        output,
        finite,
        channel_coverage,
        spectral_error,
        spectral_tilt_error,
        phase_coherence,
        polarity_preserved,
        quadrature_phase_error,
        transient_phase_discontinuity,
        final_render_leakage,
        peak,
        headroom_margin,
    ] = arguments
    else {
        return Err("usage: jocforge final-render-gate OUTPUT.json FINITE CHANNEL_COVERAGE SPECTRAL_ERROR SPECTRAL_TILT_ERROR PHASE_COHERENCE POLARITY_PRESERVED QUADRATURE_PHASE_ERROR TRANSIENT_PHASE_DISCONTINUITY FINAL_RENDER_LEAKAGE PEAK HEADROOM_MARGIN".to_owned());
    };
    let measurement = FinalRenderMeasurement {
        finite: parse_boolean(finite)?,
        channel_coverage: parse_boolean(channel_coverage)?,
        spectral_error: parse_f64("spectral error", spectral_error)?,
        spectral_tilt_error: parse_f64("spectral tilt error", spectral_tilt_error)?,
        phase_coherence: parse_f64("phase coherence", phase_coherence)?,
        polarity_preserved: parse_boolean(polarity_preserved)?,
        quadrature_phase_error: parse_f64("quadrature phase error", quadrature_phase_error)?,
        transient_phase_discontinuity: parse_f64(
            "transient phase discontinuity",
            transient_phase_discontinuity,
        )?,
        final_render_leakage: parse_f64("final render leakage", final_render_leakage)?,
        peak: parse_f64("peak", peak)?,
        headroom_margin: parse_f64("headroom margin", headroom_margin)?,
    };
    let gate = final_render_gate(&measurement);
    let json = format!(
        "{{\n  \"stage\": \"FINAL_DECODED_RENDER\",\n  \"finite\": {},\n  \"channel_coverage\": {},\n  \"spectral_error\": {:.17e},\n  \"spectral_tilt_error\": {:.17e},\n  \"phase_coherence\": {:.17e},\n  \"polarity_preserved\": {},\n  \"quadrature_phase_error\": {:.17e},\n  \"transient_phase_discontinuity\": {:.17e},\n  \"final_render_leakage\": {:.17e},\n  \"peak\": {:.17e},\n  \"headroom_margin\": {:.17e},\n  \"finite_gate\": {},\n  \"channel_coverage_gate\": {},\n  \"spectral_gate\": {},\n  \"phase_gate\": {},\n  \"polarity_gate\": {},\n  \"quadrature_gate\": {},\n  \"transient_gate\": {},\n  \"final_render_leakage_gate\": {},\n  \"headroom_gate\": {},\n  \"pass\": {}\n}}\n",
        json_bool(measurement.finite),
        json_bool(measurement.channel_coverage),
        measurement.spectral_error,
        measurement.spectral_tilt_error,
        measurement.phase_coherence,
        json_bool(measurement.polarity_preserved),
        measurement.quadrature_phase_error,
        measurement.transient_phase_discontinuity,
        measurement.final_render_leakage,
        measurement.peak,
        measurement.headroom_margin,
        json_bool(gate.finite.is_pass()),
        json_bool(gate.channel_coverage.is_pass()),
        json_bool(gate.spectral.is_pass()),
        json_bool(gate.phase.is_pass()),
        json_bool(gate.polarity.is_pass()),
        json_bool(gate.quadrature.is_pass()),
        json_bool(gate.transient.is_pass()),
        json_bool(gate.final_render_leakage.is_pass()),
        json_bool(gate.headroom.is_pass()),
        json_bool(gate.pass),
    );
    write_output(
        &PathBuf::from(output),
        json.into_bytes(),
        "final decoded render gate report",
    )?;
    if gate.pass {
        Ok(())
    } else {
        Err("final decoded render gate failed".to_owned())
    }
}

fn parse_boolean(value: &str) -> Result<bool, String> {
    match value {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(format!("invalid boolean '{value}': expected true or false")),
    }
}

fn parse_f64(name: &str, value: &str) -> Result<f64, String> {
    value
        .parse::<f64>()
        .map_err(|error| format!("invalid {name} '{value}': {error}"))
}

#[derive(Clone, Debug, PartialEq)]
enum JsonValue {
    Object(BTreeMap<String, JsonValue>),
    Array(Vec<JsonValue>),
    String(String),
    Bool(bool),
    Number(String),
    Null,
}

struct JsonParser<'a> {
    bytes: &'a [u8],
    offset: usize,
}

fn parse_json_document(document: &str) -> Result<JsonValue, String> {
    let document = document.strip_prefix('\u{feff}').unwrap_or(document);
    let mut parser = JsonParser {
        bytes: document.as_bytes(),
        offset: 0,
    };
    let value = parser.parse_value()?;
    parser.skip_whitespace();
    if parser.offset != parser.bytes.len() {
        return Err("trailing data after JSON value".to_owned());
    }
    Ok(value)
}

impl JsonParser<'_> {
    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();
        match self.bytes.get(self.offset).copied() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') => self.parse_literal(b"true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", JsonValue::Bool(false)),
            Some(b'n') => self.parse_literal(b"null", JsonValue::Null),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JsonValue::Number),
            Some(_) => Err(format!("unexpected JSON byte at offset {}", self.offset)),
            None => Err("unexpected end of JSON document".to_owned()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'{')?;
        let mut object = BTreeMap::new();
        self.skip_whitespace();
        if self.consume_byte(b'}') {
            return Ok(JsonValue::Object(object));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            object.insert(key, value);
            self.skip_whitespace();
            if self.consume_byte(b'}') {
                return Ok(JsonValue::Object(object));
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'[')?;
        let mut array = Vec::new();
        self.skip_whitespace();
        if self.consume_byte(b']') {
            return Ok(JsonValue::Array(array));
        }
        loop {
            array.push(self.parse_value()?);
            self.skip_whitespace();
            if self.consume_byte(b']') {
                return Ok(JsonValue::Array(array));
            }
            self.expect_byte(b',')?;
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut value = String::new();
        loop {
            let byte = self
                .bytes
                .get(self.offset)
                .copied()
                .ok_or_else(|| "unterminated JSON string".to_owned())?;
            self.offset += 1;
            match byte {
                b'"' => return Ok(value),
                b'\\' => {
                    let escape = self
                        .bytes
                        .get(self.offset)
                        .copied()
                        .ok_or_else(|| "unterminated JSON escape".to_owned())?;
                    self.offset += 1;
                    let character = match escape {
                        b'"' => '"',
                        b'\\' => '\\',
                        b'/' => '/',
                        b'b' => '\u{0008}',
                        b'f' => '\u{000c}',
                        b'n' => '\n',
                        b'r' => '\r',
                        b't' => '\t',
                        b'u' => {
                            let mut code = 0_u32;
                            for _ in 0..4 {
                                let digit = self
                                    .bytes
                                    .get(self.offset)
                                    .copied()
                                    .and_then(hex_digit)
                                    .ok_or_else(|| "invalid JSON unicode escape".to_owned())?;
                                self.offset += 1;
                                code = code * 16 + u32::from(digit);
                            }
                            char::from_u32(code)
                                .ok_or_else(|| "invalid JSON unicode scalar".to_owned())?
                        }
                        _ => return Err("invalid JSON escape".to_owned()),
                    };
                    value.push(character);
                }
                0..=0x1f => return Err("JSON string contains a control byte".to_owned()),
                byte => value.push(char::from(byte)),
            }
        }
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.offset;
        if self.consume_byte(b'-') {
            // The integer portion below validates that '-' is not standalone.
        }
        match self.bytes.get(self.offset).copied() {
            Some(b'0') => self.offset += 1,
            Some(b'1'..=b'9') => {
                self.offset += 1;
                while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                    self.offset += 1;
                }
            }
            _ => return Err("invalid JSON number".to_owned()),
        }
        if self.consume_byte(b'.') {
            let fraction_start = self.offset;
            while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.offset += 1;
            }
            if fraction_start == self.offset {
                return Err("JSON number has an empty fraction".to_owned());
            }
        }
        if self
            .bytes
            .get(self.offset)
            .is_some_and(|byte| *byte == b'e' || *byte == b'E')
        {
            self.offset += 1;
            if self
                .bytes
                .get(self.offset)
                .is_some_and(|byte| *byte == b'+' || *byte == b'-')
            {
                self.offset += 1;
            }
            let exponent_start = self.offset;
            while self.bytes.get(self.offset).is_some_and(u8::is_ascii_digit) {
                self.offset += 1;
            }
            if exponent_start == self.offset {
                return Err("JSON number has an empty exponent".to_owned());
            }
        }
        Ok(String::from_utf8_lossy(&self.bytes[start..self.offset]).into_owned())
    }

    fn parse_literal(&mut self, literal: &[u8], value: JsonValue) -> Result<JsonValue, String> {
        if self.bytes.get(self.offset..self.offset + literal.len()) == Some(literal) {
            self.offset += literal.len();
            Ok(value)
        } else {
            Err("invalid JSON literal".to_owned())
        }
    }

    fn skip_whitespace(&mut self) {
        while self
            .bytes
            .get(self.offset)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.offset += 1;
        }
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.bytes.get(self.offset).copied() == Some(expected) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(format!(
                "expected JSON byte '{}' at offset {}",
                char::from(expected),
                self.offset
            ))
        }
    }
}

fn hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn final_render_evidence_is_valid(value: &JsonValue) -> bool {
    let JsonValue::Object(root) = value else {
        return false;
    };
    if json_string(root, "stage") != Some("FINAL_DECODED_RENDER")
        || json_string(root, "decoder") != Some("PINNED_OPENJOC")
        || json_boolean(root, "final_render_pass") != Some(true)
    {
        return false;
    }
    let Some(JsonValue::Array(profiles)) = root.get("profiles") else {
        return false;
    };
    ["idx0", "idx1", "idx2"].into_iter().all(|profile| {
        profiles
            .iter()
            .any(|value| json_profile_gate_is_passing(value, profile))
    })
}

fn json_profile_gate_is_passing(value: &JsonValue, profile: &str) -> bool {
    let JsonValue::Object(object) = value else {
        return false;
    };
    if json_string(object, "stage") != Some("FINAL_DECODED_RENDER")
        || json_string(object, "profile") != Some(profile)
        || json_boolean(object, "finite") != Some(true)
        || json_boolean(object, "channel_coverage") != Some(true)
        || json_boolean(object, "polarity_preserved") != Some(true)
        || !required_finite_measurements_present(object)
    {
        return false;
    }
    let Some(JsonValue::Object(gate)) = object.get("gate") else {
        return false;
    };
    let required_gates = [
        "finite_gate",
        "channel_coverage_gate",
        "spectral_gate",
        "phase_gate",
        "polarity_gate",
        "quadrature_gate",
        "transient_gate",
        "final_render_leakage_gate",
        "headroom_gate",
    ];
    json_number(gate, "command_exit_code") == Some(0.0)
        && required_gates
            .into_iter()
            .all(|name| json_boolean(gate, name) == Some(true))
        && json_boolean(gate, "pass") == Some(true)
}

fn required_finite_measurements_present(object: &BTreeMap<String, JsonValue>) -> bool {
    [
        "spectral_error",
        "spectral_tilt_error",
        "phase_coherence",
        "quadrature_phase_error",
        "transient_phase_discontinuity",
        "final_render_leakage",
        "peak",
        "headroom_margin",
    ]
    .into_iter()
    .all(|name| {
        object
            .get(name)
            .and_then(json_number_value)
            .is_some_and(f64::is_finite)
    })
}

fn json_string<'a>(object: &'a BTreeMap<String, JsonValue>, name: &str) -> Option<&'a str> {
    match object.get(name) {
        Some(JsonValue::String(value)) => Some(value),
        _ => None,
    }
}

fn json_boolean(object: &BTreeMap<String, JsonValue>, name: &str) -> Option<bool> {
    match object.get(name) {
        Some(JsonValue::Bool(value)) => Some(*value),
        _ => None,
    }
}

fn json_number(object: &BTreeMap<String, JsonValue>, name: &str) -> Option<f64> {
    object.get(name).and_then(json_number_value)
}

fn json_number_value(value: &JsonValue) -> Option<f64> {
    match value {
        JsonValue::Number(value) => value.parse::<f64>().ok(),
        _ => None,
    }
}
fn json_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

fn run_fixture(arguments: &[String]) -> Result<(), String> {
    if arguments.len() != 3 && arguments.len() != 5 {
        return Err("usage: jocforge fixture --seconds N OUTPUT.bw64".to_owned());
    }
    if arguments[0] != "--seconds" {
        return Err("usage: jocforge fixture --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = arguments[1]
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{}': {error}", arguments[1]))?;
    let (object_count, output_index) = if arguments.len() == 3 {
        (15, 2)
    } else {
        if arguments[2] != "--objects" {
            return Err("usage: jocforge fixture --seconds N [--objects N] OUTPUT.bw64".to_owned());
        }
        let object_count = arguments[3]
            .parse::<usize>()
            .map_err(|error| format!("invalid fixture object count '{}': {error}", arguments[3]))?;
        (object_count, 4)
    };
    let fixture = generate_synthetic_scene_fixture(seconds, object_count)?;
    let output = PathBuf::from(&arguments[output_index]);
    write_output(&output, fixture.bw64_header_and_pcm, "fixture")
}

fn run_direct_speaker_fixture(arguments: &[String]) -> Result<(), String> {
    let [flag, seconds, output] = arguments else {
        return Err("usage: jocforge fixture-directspeakers --seconds N OUTPUT.bw64".to_owned());
    };
    if flag != "--seconds" {
        return Err("usage: jocforge fixture-directspeakers --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = seconds
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{seconds}': {error}"))?;
    let fixture = jocforge_core::generate_synthetic_direct_speaker_fixture(seconds)?;
    write_output(
        &PathBuf::from(output),
        fixture.bw64_header_and_pcm,
        "DirectSpeakers fixture",
    )
}

fn run_unsupported_fixture(arguments: &[String]) -> Result<(), String> {
    let [flag, seconds, output] = arguments else {
        return Err("usage: jocforge fixture-unsupported --seconds N OUTPUT.bw64".to_owned());
    };
    if flag != "--seconds" {
        return Err("usage: jocforge fixture-unsupported --seconds N OUTPUT.bw64".to_owned());
    }
    let seconds = seconds
        .parse::<u32>()
        .map_err(|error| format!("invalid fixture duration '{seconds}': {error}"))?;
    let fixture = jocforge_core::generate_synthetic_unsupported_fixture(seconds)?;
    write_output(
        &PathBuf::from(output),
        fixture.bw64_header_and_pcm,
        "unsupported fixture",
    )
}

fn run_inspect_adm(arguments: &[String]) -> Result<(), String> {
    let (input, inventory_output) = match arguments {
        [input] => (input.as_str(), None),
        [input, flag, output] if flag == "--direct-speakers-csv" => {
            (input.as_str(), Some(output.as_str()))
        }
        _ => {
            return Err(
                "usage: jocforge inspect-adm INPUT.bw64 [--direct-speakers-csv OUTPUT.csv]"
                    .to_owned(),
            );
        }
    };
    let input = PathBuf::from(input);
    let scanned = scan_bw64_file(&input)?;
    let parsed = parse_bw64_adm_metadata(
        &scanned.fmt,
        &scanned.axml,
        &scanned.chna,
        scanned.data_size,
    )
    .map_err(|error| format!("failed to inspect ADM: {error}"))?;
    let chunk_ids = scanned
        .chunks
        .iter()
        .map(|chunk| String::from_utf8_lossy(&chunk.id).to_string())
        .collect::<Vec<_>>()
        .join(",");
    println!(
        "form={} chunks={} chunk_ids={} data_offset={} data_bytes={} dbmd_bytes={} sample_rate={} channels={} samples={} tracks={} track_uids={} objects={} dynamic_objects={} direct_speakers={} direct_speaker_channels={}",
        String::from_utf8_lossy(&scanned.form),
        scanned.chunks.len(),
        chunk_ids,
        scanned.data_offset,
        scanned.data_size,
        scanned.dbmd.len(),
        parsed.sample_rate,
        parsed.channel_count,
        parsed.sample_count,
        parsed.track_count,
        parsed.track_uid_count,
        parsed.objects.len(),
        parsed.dynamic_object_count(),
        parsed.direct_speaker_count(),
        parsed.direct_speaker_channel_count()
    );
    if let Some(output) = inventory_output {
        write_direct_speaker_inventory(&PathBuf::from(output), &parsed)?;
    }
    Ok(())
}

fn write_direct_speaker_inventory(
    path: &Path,
    metadata: &jocforge_core::ParsedAdmMetadata,
) -> Result<(), String> {
    let programme_relationship = metadata
        .programmes
        .iter()
        .flat_map(|programme| {
            programme
                .content_id_refs
                .iter()
                .map(|content_id| format!("{} -> {content_id}", programme.programme_id))
        })
        .collect::<Vec<_>>()
        .join(";");
    let mut output = String::from(
        "object_id,object_name,pack_format_id,pack_format_name,channel_id,channel_name,speaker_label,low_pass,track_uid,track_index,is_lfe,azimuth,elevation,distance,object_start,object_duration,block_rtime,block_duration,gain,programme_relationship\n",
    );
    for object in metadata
        .objects
        .iter()
        .filter(|object| object.kind == AdmObjectKind::DirectSpeakers)
    {
        for channel in &object.channels {
            let _ = writeln!(
                output,
                "{},{},{},{},{},{},{},{},{},{},{},{:.17e},{:.17e},{:.17e},{},{},{},{},{},{}",
                csv_cell(&object.object_id),
                csv_cell(&object.object_name),
                csv_cell(&object.pack_format_id),
                csv_cell(&object.pack_format_name),
                csv_cell(&channel.channel_id),
                csv_cell(&channel.channel_name),
                csv_cell(channel.speaker_label.as_deref().unwrap_or("")),
                channel
                    .low_pass
                    .map_or_else(String::new, |value| value.to_string()),
                csv_cell(&channel.track_uid),
                channel.track_index,
                channel.is_lfe,
                channel.position.azimuth,
                channel.position.elevation,
                channel.position.distance,
                csv_cell(object.start.as_deref().unwrap_or("")),
                csv_cell(object.duration.as_deref().unwrap_or("")),
                csv_cell(channel.block_rtime.as_deref().unwrap_or("")),
                csv_cell(channel.block_duration.as_deref().unwrap_or("")),
                csv_cell(channel.gain.as_deref().unwrap_or("")),
                csv_cell(&programme_relationship),
            );
        }
    }
    write_output(path, output.into_bytes(), "DirectSpeakers inventory")
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\r', '\n']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

fn run_encode(arguments: &[String]) -> Result<(), String> {
    if arguments.len() < 4 || (arguments.len() - 4) % 2 != 0 {
        return Err(
            "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2 [--carriage etsi-strict|deployed-compat] [--bitrate KBPS] [--temporal-weight VALUE]"
                .to_owned(),
        );
    }
    let input = &arguments[0];
    let output = &arguments[1];
    let flag = &arguments[2];
    let profile_name = &arguments[3];
    if flag != "--profile" {
        return Err(
            "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2 [--carriage etsi-strict|deployed-compat] [--bitrate KBPS] [--temporal-weight VALUE]"
                .to_owned(),
        );
    }
    let mut target_bitrate_kbps = 1664;
    let mut carriage = JocCarriage::EtsiStrict;
    let mut temporal_continuity_weight = ReductionConfig::default().temporal_continuity_weight;
    let mut option_index = 4;
    while option_index < arguments.len() {
        let option = &arguments[option_index];
        let value = arguments
            .get(option_index + 1)
            .ok_or_else(|| "encode option is missing its value".to_owned())?;
        match option.as_str() {
            "--carriage" => {
                carriage = parse_carriage(value)?;
            }
            "--bitrate" => {
                target_bitrate_kbps = value
                    .parse::<u32>()
                    .map_err(|error| format!("invalid total bitrate '{value}': {error}"))?;
            }
            "--temporal-weight" => {
                temporal_continuity_weight = value.parse::<f64>().map_err(|error| {
                    format!("invalid temporal continuity weight '{value}': {error}")
                })?;
            }
            _ => {
                return Err(
                    "usage: jocforge encode INPUT.bw64 OUTPUT.ec3 --profile idx0|idx1|idx2 [--carriage etsi-strict|deployed-compat] [--bitrate KBPS] [--temporal-weight VALUE]"
                        .to_owned(),
                );
            }
        }
        option_index += 2;
    }
    let profile = parse_profile(profile_name)?;
    if carriage == JocCarriage::DeployedCompat && profile != JocProfile::Idx0 {
        return Err("deployed-compatible carriage currently supports idx0 only".to_owned());
    }
    let rate = rate_budget_for_carriage(carriage, target_bitrate_kbps)
        .map_err(|error| format!("invalid encode budget: {error}"))?;
    let input_path = PathBuf::from(input);
    let scanned = scan_bw64_file(&input_path)?;
    let metadata = parse_bw64_adm_metadata(
        &scanned.fmt,
        &scanned.axml,
        &scanned.chna,
        scanned.data_size,
    )
    .map_err(|error| {
        format!(
            "failed to parse ADM metadata '{}': {error}",
            input_path.display()
        )
    })?;
    validate_source_derived_adm_metadata(&metadata)
        .map_err(|error| format!("failed to validate ADM encode boundary: {error}"))?;
    if let Some(object) = metadata
        .objects
        .iter()
        .find(|object| object.kind == AdmObjectKind::Other)
    {
        return Err(format!(
            "unsupported ADM object kind for {}: only Objects and DirectSpeakers are supported",
            object.object_id
        ));
    }
    let supported_objects = metadata
        .objects
        .iter()
        .filter(|object| object.kind != AdmObjectKind::Other)
        .collect::<Vec<_>>();
    if supported_objects.is_empty() {
        return Err("ADM scene has no supported authored objects".to_owned());
    }
    let track_indices = supported_objects
        .iter()
        .flat_map(|object| object.channels.iter().map(|channel| channel.track_index))
        .collect::<Vec<_>>();
    let lfe_source_indices = supported_objects
        .iter()
        .flat_map(|object| object.channels.iter())
        .enumerate()
        .filter_map(|(index, channel)| channel.is_lfe.then_some(index))
        .collect::<Vec<_>>();
    let i0_channels = 5 + usize::from(!lfe_source_indices.is_empty());
    let rank = profile_spec(profile).input_labels.len();
    let frame_count = metadata.sample_count.div_ceil(JOC_FRAME_SAMPLES);
    let encoded_samples = frame_count
        .checked_mul(JOC_FRAME_SAMPLES)
        .ok_or_else(|| "ADM encoded sample count overflows".to_owned())?;
    let work_root = env::temp_dir().join(format!("jocforge-encode-{}", std::process::id()));
    fs::create_dir_all(&work_root)
        .map_err(|error| format!("failed to create encoder work directory: {error}"))?;
    let i0_raw_path = work_root.join("carrier-i0.f32le");
    let i0_carrier_path = work_root.join("carrier-i0.ec3");
    let reduction_config = ReductionConfig {
        temporal_continuity_weight,
        ..ReductionConfig::default()
    };
    let mut metrics = String::from(
        "record,frame,source_index,object_id,kind,coded_object_index,contribution_gain,source_energy,represented_energy,relative_error,spatial_error,source_count,coded_count,silent_drops,source_reconstruction_error,spatial_representation_error,cross_object_leakage,unexplained_cluster_leakage,speaker_render_error,factorization_reconstruction_error,matrix_coefficient_jump,carrier_coefficient_jump,max_coded_peak,headroom_margin,authored_source_assignment_changes,representative_source_changes,coded_representative_source_index,source_azimuth,source_elevation,source_distance,coded_azimuth,coded_elevation,coded_distance,coded_first_sample,coded_last_sample\n",
    );
    let mut previous_reduction = None;
    let mut previous_factorization: Option<jocforge_core::RankFactorization> = None;
    let mut raw_file = fs::File::create(&i0_raw_path).map_err(|error| {
        format!(
            "failed to create carrier PCM '{}': {error}",
            i0_raw_path.display()
        )
    })?;
    for frame_index in 0..frame_count {
        let source_pcm = read_pcm_frame(&input_path, &scanned, frame_index, &track_indices)?;
        let lfe_pcm = lfe_carrier(&source_pcm, &lfe_source_indices)?;
        let sources = reduction_sources(&supported_objects, &source_pcm)?;
        let reduction =
            reduce_scene_with_previous(&sources, reduction_config, previous_reduction.as_ref())
                .map_err(|error| {
                    format!("failed to reduce ADM scene at frame {frame_index}: {error}")
                })?;
        let object_pcm = reduction
            .coded_objects
            .iter()
            .map(|object| object.signal.clone())
            .collect::<Vec<_>>();
        let factorization = factorize_source_derived(&object_pcm, rank).map_err(|error| {
            format!("failed to factorize reduced objects at frame {frame_index}: {error}")
        })?;
        let factorization_reconstruction_error = reconstruction_error(&object_pcm, &factorization)
            .map_err(|error| {
                format!("failed to measure factorization residual at frame {frame_index}: {error}")
            })?;
        let matrix_coefficient_jump = if let Some(previous) = previous_factorization.as_ref() {
            maximum_absolute_difference(&previous.scaled_matrix, &factorization.scaled_matrix)
                .map_err(|error| {
                    format!(
                        "failed to measure matrix coefficient jump at frame {frame_index}: {error}"
                    )
                })?
        } else {
            0.0
        };
        let carrier_coefficient_jump = if let Some(previous) = previous_factorization.as_ref() {
            maximum_absolute_difference(&previous.carriers, &factorization.carriers).map_err(
                |error| {
                    format!(
                        "failed to measure carrier coefficient jump at frame {frame_index}: {error}"
                    )
                },
            )?
        } else {
            0.0
        };
        let mut i0_carriers = factorization.carriers[..5].to_vec();
        if !lfe_source_indices.is_empty() {
            i0_carriers.push(lfe_pcm.clone());
        }
        raw_file
            .write_all(&interleaved_f32le(&i0_carriers, JOC_FRAME_SAMPLES))
            .map_err(|error| format!("failed to write carrier PCM frame {frame_index}: {error}"))?;
        append_reduction_metrics(
            &mut metrics,
            frame_index,
            &sources,
            &reduction,
            factorization_reconstruction_error,
            matrix_coefficient_jump,
            carrier_coefficient_jump,
        );
        append_lfe_metrics(
            &mut metrics,
            frame_index,
            &supported_objects,
            &source_pcm,
            &lfe_source_indices,
            reduction.coded_objects.len(),
            &lfe_pcm,
        );
        previous_factorization = Some(factorization);
        previous_reduction = Some(reduction);
    }
    drop(raw_file);
    encode_ffmpeg_carrier(
        &i0_raw_path,
        &i0_carrier_path,
        i0_channels,
        &format!("{}k", rate.carrier_encode_bitrate_kbps),
    )?;
    let i0_carrier_bytes = read_carrier_frames(
        &i0_carrier_path,
        rate.carrier_encoded_frame_bytes,
        frame_count,
    )?;
    let mut output_bytes =
        Vec::with_capacity(frame_count * (rate.carrier_frame_bytes + rate.d0_frame_bytes));
    previous_reduction = None;
    for frame_index in 0..frame_count {
        let i0_start = frame_index * rate.carrier_encoded_frame_bytes;
        let i0_frame = &i0_carrier_bytes[i0_start..i0_start + rate.carrier_encoded_frame_bytes];
        let source_pcm = read_pcm_frame(&input_path, &scanned, frame_index, &track_indices)?;
        let sources = reduction_sources(&supported_objects, &source_pcm)?;
        let reduction =
            reduce_scene_with_previous(&sources, reduction_config, previous_reduction.as_ref())
                .map_err(|error| {
                    format!("failed to repeat ADM reduction at frame {frame_index}: {error}")
                })?;
        let object_pcm = reduction
            .coded_objects
            .iter()
            .map(|object| object.signal.clone())
            .collect::<Vec<_>>();
        let factorization = factorize_source_derived(&object_pcm, rank).map_err(|error| {
            format!("failed to repeat reduced factorization at frame {frame_index}: {error}")
        })?;
        let sequence = u16::try_from(frame_index % 1023 + 1).unwrap_or(1);
        let positions = reduction
            .coded_objects
            .iter()
            .map(|object| object.position)
            .collect::<Vec<_>>();
        let d0_source_indices = profile_spec(profile).d0_source_indices;
        let d0_carriers = d0_source_indices
            .into_iter()
            .map(|index| factorization.carriers[index].clone())
            .collect::<Vec<_>>();
        let access_unit = match carriage {
            JocCarriage::EtsiStrict => package_source_derived_access_unit_with_frame_sizes(
                profile,
                sequence,
                &factorization.scaled_matrix,
                &positions,
                i0_frame,
                &d0_carriers,
                0,
                rate.d0_frame_bytes,
            ),
            JocCarriage::DeployedCompat => package_deployed_compat_access_unit(
                profile,
                sequence,
                &factorization.scaled_matrix,
                &positions,
                i0_frame,
            ),
        }
        .map_err(|error| format!("failed to package frame {frame_index}: {error}"))?;
        output_bytes.extend_from_slice(&access_unit);
        previous_reduction = Some(reduction);
    }
    let output_path = PathBuf::from(output);
    let output_bytes_len = output_bytes.len();
    write_output(&output_path, output_bytes, "encoded")?;
    let metrics_path = PathBuf::from(format!("{}.metrics.csv", output_path.display()));
    write_output(&metrics_path, metrics.into_bytes(), "reduction metrics")?;
    let _ = fs::remove_dir_all(work_root);
    println!(
        "encoded profile={profile_name} carriage={carriage:?} frames={frame_count} samples={encoded_samples} authored_sources={} authored_channels={} dynamic_objects={} direct_speakers={} direct_speaker_channels={} lfe_core_channels={} coded_objects_last_frame={} output_bytes={output_bytes_len} target_bitrate={} carrier_bitrate={} d0_bitrate={} temporal_weight={:.17e} reduction=deterministic:farthest-point+weighted-aggregate metrics={}",
        supported_objects.len(),
        track_indices.len(),
        metadata.dynamic_object_count(),
        metadata.direct_speaker_count(),
        metadata.direct_speaker_channel_count(),
        usize::from(!lfe_source_indices.is_empty()),
        previous_reduction
            .as_ref()
            .map_or(0, |result| result.coded_objects.len()),
        rate.total_bitrate_kbps * 1000,
        rate.carrier_bitrate_kbps * 1000,
        rate.d0_bitrate_kbps * 1000,
        reduction_config.temporal_continuity_weight,
        metrics_path.display(),
    );
    Ok(())
}

fn reduction_sources(
    objects: &[&AdmObject],
    pcm: &[Vec<f64>],
) -> Result<Vec<ReductionSource>, String> {
    let expected_sources = objects
        .iter()
        .map(|object| object.channels.len())
        .sum::<usize>();
    if expected_sources != pcm.len() {
        return Err(format!(
            "ADM source and PCM counts differ: {} channel bindings, {} tracks",
            expected_sources,
            pcm.len()
        ));
    }
    let mut sources = Vec::with_capacity(
        pcm.len().saturating_sub(
            objects
                .iter()
                .flat_map(|object| object.channels.iter())
                .filter(|channel| channel.is_lfe)
                .count(),
        ),
    );
    let mut flat_index = 0;
    for object in objects {
        for channel in &object.channels {
            let signal = pcm
                .get(flat_index)
                .cloned()
                .ok_or_else(|| "ADM source and PCM channel bindings are misaligned".to_owned())?;
            if channel.is_lfe {
                flat_index += 1;
                continue;
            }
            let object_id = if object.channels.len() == 1 {
                object.object_id.clone()
            } else {
                format!("{}:{}", object.object_id, channel.channel_id)
            };
            sources.push(ReductionSource {
                source_index: flat_index,
                object_id,
                kind: object.kind,
                position: channel.position,
                signal,
            });
            flat_index += 1;
        }
    }
    Ok(sources)
}

fn lfe_carrier(pcm: &[Vec<f64>], lfe_indices: &[usize]) -> Result<Vec<f64>, String> {
    let mut carrier = vec![0.0; JOC_FRAME_SAMPLES];
    for index in lfe_indices {
        let channel = pcm
            .get(*index)
            .ok_or_else(|| "LFE PCM binding is outside the selected frame".to_owned())?;
        if channel.len() != JOC_FRAME_SAMPLES {
            return Err("LFE PCM frame does not have the public frame length".to_owned());
        }
        for (output, value) in carrier.iter_mut().zip(channel) {
            *output += *value;
        }
    }
    limit_signal_peak(&carrier, SOURCE_DERIVED_CARRIER_PEAK_LIMIT)
        .map_err(|error| format!("failed to apply LFE carrier peak policy: {error}"))
}

fn append_reduction_metrics(
    output: &mut String,
    frame_index: usize,
    sources: &[ReductionSource],
    reduction: &jocforge_core::ReductionResult,
    factorization_reconstruction_error: f64,
    matrix_coefficient_jump: f64,
    carrier_coefficient_jump: f64,
) {
    for accounting in &reduction.source_accounting {
        let source = sources
            .iter()
            .find(|source| source.source_index == accounting.source_index);
        let (object_id, kind) = source.map_or(("unknown", "unknown"), |source| {
            (
                source.object_id.as_str(),
                match source.kind {
                    AdmObjectKind::Objects => "Objects",
                    AdmObjectKind::DirectSpeakers => "DirectSpeakers",
                    AdmObjectKind::Other => "Other",
                },
            )
        });
        let coded = reduction.coded_objects.get(accounting.coded_object_index);
        let source_position = source.map_or(
            jocforge_core::AdmPosition {
                azimuth: 0.0,
                elevation: 0.0,
                distance: 0.0,
            },
            |source| source.position,
        );
        let _ = writeln!(
            output,
            "source,{frame_index},{},{object_id},{kind},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{},{},{},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e},{:.17e}",
            accounting.source_index,
            accounting.coded_object_index,
            accounting.contribution_gain,
            accounting.source_energy,
            accounting.represented_energy,
            accounting.relative_error,
            accounting.spatial_error,
            reduction.diagnostics.authored_source_count,
            reduction.diagnostics.coded_object_count,
            reduction.diagnostics.silent_object_drops,
            reduction.diagnostics.source_reconstruction_error,
            reduction.diagnostics.spatial_representation_error,
            reduction.diagnostics.cross_object_leakage,
            reduction.diagnostics.unexplained_cluster_leakage,
            reduction.diagnostics.speaker_render_error,
            factorization_reconstruction_error,
            matrix_coefficient_jump,
            carrier_coefficient_jump,
            reduction.diagnostics.max_coded_peak,
            reduction.diagnostics.headroom_margin,
            reduction.diagnostics.authored_source_assignment_changes,
            reduction.diagnostics.representative_source_changes,
            coded.map_or(usize::MAX, |object| object.representative_source_index),
            source_position.azimuth,
            source_position.elevation,
            source_position.distance,
            coded.map_or(0.0, |object| object.position.azimuth),
            coded.map_or(0.0, |object| object.position.elevation),
            coded.map_or(0.0, |object| object.position.distance),
            coded
                .and_then(|object| object.signal.first())
                .copied()
                .unwrap_or(0.0),
            coded
                .and_then(|object| object.signal.last())
                .copied()
                .unwrap_or(0.0),
        );
    }
}

fn append_lfe_metrics(
    output: &mut String,
    frame_index: usize,
    objects: &[&AdmObject],
    pcm: &[Vec<f64>],
    lfe_source_indices: &[usize],
    coded_count: usize,
    lfe_carrier: &[f64],
) {
    let lfe_peak = lfe_carrier
        .iter()
        .map(|value| value.abs())
        .fold(0.0, f64::max);
    let lfe_headroom = (SOURCE_DERIVED_CARRIER_PEAK_LIMIT - lfe_peak).max(0.0);
    let mut flat_index = 0;
    for object in objects {
        for channel in &object.channels {
            if !lfe_source_indices.contains(&flat_index) {
                flat_index += 1;
                continue;
            }
            let signal = &pcm[flat_index];
            let energy =
                signal.iter().map(|value| value * value).sum::<f64>() / signal.len().max(1) as f64;
            let row = [
                "lfe".to_owned(),
                frame_index.to_string(),
                flat_index.to_string(),
                format!("{}:{}", object.object_id, channel.channel_id),
                "DirectSpeakers".to_owned(),
                "-1".to_owned(),
                "1.00000000000000000e0".to_owned(),
                format!("{energy:.17e}"),
                format!("{energy:.17e}"),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                pcm.len().to_string(),
                coded_count.to_string(),
                "0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                "0.00000000000000000e0".to_owned(),
                format!("{lfe_peak:.17e}"),
                format!("{lfe_headroom:.17e}"),
                "0".to_owned(),
                "0".to_owned(),
                "0".to_owned(),
                format!("{:.17e}", channel.position.azimuth),
                format!("{:.17e}", channel.position.elevation),
                format!("{:.17e}", channel.position.distance),
                format!("{:.17e}", channel.position.azimuth),
                format!("{:.17e}", channel.position.elevation),
                format!("{:.17e}", channel.position.distance),
                format!("{:.17e}", signal.first().copied().unwrap_or(0.0)),
                format!("{:.17e}", signal.last().copied().unwrap_or(0.0)),
            ];
            output.push_str(&row.join(","));
            output.push('\n');
            flat_index += 1;
        }
    }
}

fn encode_ffmpeg_carrier(
    raw_path: &Path,
    carrier_path: &Path,
    channels: usize,
    bitrate: &str,
) -> Result<(), String> {
    let mut command = std::process::Command::new("ffmpeg");
    command
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "f32le",
            "-ar",
            "48000",
            "-ac",
        ])
        .arg(channels.to_string())
        .args(["-i"])
        .arg(raw_path)
        .args(["-c:a", "eac3", "-b:a", bitrate]);
    if bitrate == "384k" {
        // FFmpeg's automatic low-rate coupling emits a carrier that the
        // pinned OpenJOC renderer cannot traverse (grouped exponent 125).
        // Disable only that low-rate mode; the public 5-channel carrier
        // topology and its exact frame budget remain unchanged.
        command.args(["-channel_coupling", "0"]);
    }
    let ffmpeg = command
        .args(["-f", "eac3"])
        .arg(carrier_path)
        .status()
        .map_err(|error| format!("failed to start FFmpeg E-AC-3 encoder: {error}"))?;
    if !ffmpeg.success() {
        return Err(format!(
            "FFmpeg E-AC-3 {channels}-channel carrier encoding failed"
        ));
    }
    Ok(())
}

fn read_carrier_frames(
    path: &Path,
    frame_bytes: usize,
    expected_frames: usize,
) -> Result<Vec<u8>, String> {
    let bytes = fs::read(path).map_err(|error| {
        format!(
            "failed to read FFmpeg carrier output '{}': {error}",
            path.display()
        )
    })?;
    if bytes.len() % frame_bytes != 0 {
        return Err(format!(
            "FFmpeg carrier output '{}' is not a sequence of {frame_bytes}-byte frames",
            path.display()
        ));
    }
    if bytes.len() / frame_bytes != expected_frames {
        return Err(format!(
            "FFmpeg carrier '{}' frame count differs: expected {expected_frames}, got {}",
            path.display(),
            bytes.len() / frame_bytes
        ));
    }
    Ok(bytes)
}

fn interleaved_f32le(carriers: &[Vec<f64>], samples: usize) -> Vec<u8> {
    let channels = carriers.len();
    let mut bytes = Vec::with_capacity(samples * channels * 4);
    for sample in 0..samples {
        for carrier in carriers {
            bytes.extend_from_slice(&(carrier[sample] as f32).to_le_bytes());
        }
    }
    bytes
}

fn parse_profile(value: &str) -> Result<JocProfile, String> {
    match value {
        "idx0" => Ok(JocProfile::Idx0),
        "idx1" => Ok(JocProfile::Idx1),
        "idx2" => Ok(JocProfile::Idx2),
        other => Err(format!("unsupported JOC profile '{other}'")),
    }
}

fn parse_carriage(value: &str) -> Result<JocCarriage, String> {
    match value {
        "etsi-strict" => Ok(JocCarriage::EtsiStrict),
        "deployed-compat" => Ok(JocCarriage::DeployedCompat),
        other => Err(format!("unsupported JOC carriage '{other}'")),
    }
}

fn write_output(path: &Path, bytes: Vec<u8>, description: &str) -> Result<(), String> {
    fs::write(path, bytes).map_err(|error| {
        format!(
            "failed to write {description} output '{}': {error}",
            path.display()
        )
    })
}
