// pattern: Functional Core
//! 将 Wash 路由输入投影到实际的 XML 解析器、原生 profile、reader 阶段和点行出口。

use super::{RECORD_BYTES, render_target_row};
use crate::frontend::job_configuration::{
    SelectedWaveDescriptor, parse_selected_job_configuration,
};
use crate::joc::JocObjectCount;
use crate::joc::oamd::reader_target::{ReaderTargetPass, ReaderTargetState, write_record_word};
use crate::profile::{
    EmdfCarriageProfile, EncodingProfile, JocReconstructionLayout, resolve_selected_profile,
};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ROUTES: &str =
    include_str!("../../../../../tests/fixtures/default_auto_position/routes.json");
const INPUTS: &str =
    include_str!("../../../../../tests/fixtures/default_auto_position/inputs.json");
const EXPECTED: &str =
    include_str!("../../../../../tests/fixtures/default_auto_position/expected.json");
const JOB_INPUT: &str =
    include_str!("../../../../../tests/fixtures/job_configuration/job-configuration-vectors.json");

fn token(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| value.to_string(), str::to_owned)
}

fn rejection(code: &str) -> Value {
    json!({"calls": 0, "result":{"kind":"reject","code":code}})
}

fn replay(context: &Value, witnesses: &[&Value]) -> Value {
    let jobs: Value = serde_json::from_str(JOB_INPUT).unwrap();
    let hz = u32::try_from(context["sample_hz"].as_u64().unwrap()).unwrap();
    let mut xml = jobs["config_cases"][0]["input_xml"]
        .as_str()
        .unwrap()
        .replace(
            ">1024</data_rate>",
            &format!(">{}</data_rate>", context["kbps"]),
        )
        .replace(
            "auto</surround_trim",
            &format!("{}</surround_trim", token(&context["surround_trim"])),
        )
        .replace(
            "auto</height_trim",
            &format!("{}</height_trim", token(&context["height_trim"])),
        )
        .replace("out.ec3", context["output_label"].as_str().unwrap());
    if context["input"] != "selected_adm_bwf" {
        xml = xml
            .replace("<adm>", "<transcode>")
            .replace("</adm>", "</transcode>");
    }
    let job = match parse_selected_job_configuration(
        xml.as_bytes(),
        SelectedWaveDescriptor::new(hz, 38, 24),
        4608,
    ) {
        Ok(job) => job,
        Err(e) => return rejection(e.code()),
    };
    let index = u8::try_from(context["profile_index"].as_u64().unwrap()).unwrap();
    let layout = JocReconstructionLayout::from_joc_dmx_config_idx(index).unwrap();
    let signalling = if context["signaling"] == "deployed" {
        EmdfCarriageProfile::DeployedCompat
    } else {
        EmdfCarriageProfile::EtsiEmdf
    };
    let output = resolve_selected_profile(
        &EncodingProfile::selected_with_rate_and_carriage_and_joc_reconstruction_layout(
            job.selected_profile().codec_rate(),
            signalling,
            layout,
        ),
    )
    .unwrap();

    // 只有原生输出可向 MAIN 交付 Some(profile)。D0/总码率、结构重组和未知绑定
    // 均不能用同形的 I0 header 替代；实际 CLI 在消耗拓扑计划前作同样的区分。
    let native = context["resolved_path"] == "native_selected"
        && context["rate_kind"] == "native_I0"
        && context["codec"] == "EAC3"
        && context["topology"] == "I0"
        && context["blocks"] == json!([6])
        && context["lfe"] == "I0"
        && context["metadata_owner"] == "I0"
        && context["placement"] == "block_skip";
    let mut state = ReaderTargetState::cold();
    state.trim = job.main_target_trim(native.then_some(output));
    state.strict_oamd = signalling == EmdfCarriageProfile::EtsiEmdf;
    state.object_count =
        match serde_json::from_value::<JocObjectCount>(json!(token(&context["coded_objects"]))) {
            Ok(count) => count,
            Err(e) => return rejection(&e.to_string()),
        };

    // reader 的选定几何/折叠是固定类型，不提供运行时更换其槽数、映射或
    // 折叠的开关。无法绑定这个实际消费者时，走已有的无阶段入口。
    let pass = if context["stage"] == "METER" {
        ReaderTargetPass::Meter
    } else if context["stage"] == "MAIN"
        && context["consumer"] == "selected_reader_point_geometry_then_fold"
        && context["position_context"] == "selected_default"
        && context["geometry"] == "seven_active_slots_in_eight"
        && context["target"] == "L,R,C,LFE,Ls,Rs"
        && context["fold"] == 3
        && context["object_map"] == "default"
        && context["row"] != "direct_speakers"
    {
        ReaderTargetPass::Main
    } else {
        ReaderTargetPass::Unspecified
    };
    let trim = state.trim_for_pass(pass);
    let mut calls = serde_json::Map::new();
    let mut numeric = serde_json::Map::new();
    let mut digest = Sha256::new();
    for witness in witnesses {
        let id = witness["id"].as_str().unwrap();
        let [x, y, z]: [u32; 3] = serde_json::from_value(witness["q"].clone()).unwrap();
        let direct = context["row"] == "direct_lfe";
        let mut record = [0; RECORD_BYTES];
        let coordinates_valid = context["coordinates"] == "valid_decoded_q15";
        for (i, v) in [
            (0, if direct { 1 } else { 2 }),
            (1, if direct { 4 } else { x }),
            (2, if coordinates_valid { y } else { 32768 }),
            (3, z),
            (11, 1),
            (12, context["gain_code"].as_u64().unwrap() as u32),
            (13, context["row_flag"].as_u64().unwrap() as u32),
        ] {
            write_record_word(&mut record, i, v);
        }
        let mut n = 0;
        let mut observed = None;
        let result = render_target_row(
            usize::from(!direct),
            &record,
            trim,
            context["row"] == "active_regular_type2",
            &mut |eight, factor, pre| {
                n += 1;
                observed = Some(
                    json!({"geometry7":[eight[0],eight[1],eight[2],eight[4],eight[5],eight[6],eight[7]], "factor":factor, "pre8":pre}),
                );
            },
        );
        let post = match result {
            Ok(target) => target,
            Err(e) => {
                assert_eq!(n, 0);
                return rejection(e.code());
            }
        };
        for word in post {
            digest.update(word.to_le_bytes());
        }
        calls.insert(id.into(), json!(n));
        if let Some(mut trace) = observed {
            trace["post6"] = json!(post.map(f32::from_bits));
            numeric.insert(id.into(), trace);
        }
    }
    let result = json!({"kind":"target","sha256":format!("{:x}", digest.finalize())});
    if numeric.is_empty() {
        assert!(calls.values().all(|count| *count == 0));
        json!({"calls":0, "result":result})
    } else {
        json!({"calls":calls, "numeric":numeric, "result":result})
    }
}

#[test]
fn washed_rate_routes_exercise_the_actual_point_consumer() {
    let routes: Value = serde_json::from_str(ROUTES).unwrap();
    let fixtures: Value = serde_json::from_str(INPUTS).unwrap();
    let expected: Value = serde_json::from_str(EXPECTED).unwrap();
    let witnesses = routes["active_witness_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|id| {
            fixtures["decoded_grid"]
                .as_array()
                .unwrap()
                .iter()
                .find(|case| case["id"] == *id)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let mut differences = Vec::new();
    let mut counts = [0; 3];
    for case in routes["cases"].as_array().unwrap() {
        let mut ctx = routes["base"].clone();
        ctx.as_object_mut()
            .unwrap()
            .extend(case[1].as_object().unwrap().clone());
        if ctx["operation"] == "copy_encoded_stream" {
            // 由 CLI 的真实 M4A 再封装测试验证，不向 reader 构造伪作业。
            counts[2] += 1;
            assert_eq!(case[2], "NO_RENDER");
            continue;
        }
        let actual = replay(&ctx, &witnesses);
        println!("JOC51_ROUTE {}", json!([case[0], actual]));
        let decision = if actual.get("numeric").is_some() {
            counts[0] += 1;
            for witness in &witnesses {
                let id = witness["id"].as_str().unwrap();
                assert_eq!(actual["calls"][id], 1, "{}:{id}", case[0]);
                let row = &actual["numeric"][id];
                let words = |v: &Value| {
                    serde_json::from_value::<Vec<f32>>(v.clone())
                        .unwrap()
                        .into_iter()
                        .map(f32::to_bits)
                        .collect::<Vec<_>>()
                };
                for field in ["geometry7", "pre8", "post6"] {
                    assert_eq!(
                        words(&row[field]),
                        words(&expected[id][field]),
                        "{}:{id}:{field}",
                        case[0]
                    );
                }
                assert_eq!(
                    (row["factor"].as_f64().unwrap() as f32).to_bits(),
                    (expected[id]["factor"].as_f64().unwrap() as f32).to_bits()
                );
            }
            "V1_ONCE"
        } else {
            counts[1] += 1;
            "BASELINE"
        };
        if case[2] != decision {
            differences.push((case[0].clone(), decision));
        }
    }
    assert!(differences.is_empty(), "{differences:?}");
    assert_eq!(counts, [13, 46, 1]);
}
