// pattern: Functional Core
//! Replays the sanitized official-sample D0 BSI vector against the selected structural writer.

use jocforge_domain::codec::{
    bitio::{FrameBitCursor, FrameBitWriter},
    frame_builder::write_flat7x_d0_bsi_through_chanmap,
};
use serde_json::Value;

const FIELD_VALUES: &str = include_str!(
    "../../../outputs/IMPLEMENTER_SUPPLEMENTS/OFFICIAL-DEMO-D0-BSI-THROUGH-CHANMAP-V1/FIELD-VALUES.json"
);

fn read_bits(bytes: &[u8], start: u32, width: u8) -> u32 {
    (0..u32::from(width)).fold(0_u32, |value, offset| {
        let bit = start + offset;
        let byte = usize::try_from(bit / 8).unwrap();
        let shift = 7 - bit % 8;
        (value << 1) | u32::from(bytes[byte] & (1_u8 << shift) != 0)
    })
}

#[test]
fn official_demo_d0_bsi_matches_through_chanmap_with_target_rate_override() {
    let evidence: Value = serde_json::from_str(FIELD_VALUES).unwrap();
    assert_eq!(
        evidence["package_id"],
        "OFFICIAL-DEMO-D0-BSI-THROUGH-CHANMAP-V1"
    );
    let sample = &evidence["d0_sample_frame"];
    let target = &evidence["target_rate_contract"];
    assert_eq!(sample["sample_rate_kbps"], 640);
    assert_eq!(sample["observed_frmsiz"], 1279);
    assert_eq!(target["target_rate_kbps"], 1024);
    assert_eq!(target["target_frame_bytes"], 4096);
    assert_eq!(target["target_frmsiz"], 2047);

    let dialnorm = u8::try_from(
        sample["fields_through_chanmap"]
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["name"] == "dialnorm")
            .unwrap()["value"]
            .as_u64()
            .unwrap(),
    )
    .unwrap();
    let compr = u8::try_from(
        sample["fields_through_chanmap"]
            .as_array()
            .unwrap()
            .iter()
            .find(|field| field["name"] == "compr")
            .unwrap()["value"]
            .as_u64()
            .unwrap(),
    )
    .unwrap();

    let mut bytes = [0_u8; 12];
    let end;
    {
        let mut writer = FrameBitWriter::new(&mut bytes, FrameBitCursor::new(0)).unwrap();
        end = write_flat7x_d0_bsi_through_chanmap(&mut writer, dialnorm, Some(compr)).unwrap();
    }
    assert_eq!(end.bit_offset(), 76);

    for field in sample["fields_through_chanmap"].as_array().unwrap() {
        let name = field["name"].as_str().unwrap();
        let width = u8::try_from(field["width_bits"].as_u64().unwrap()).unwrap();
        let start = u32::try_from(field["frame_bit_start"].as_u64().unwrap()).unwrap();
        let observed = u32::try_from(field["value"].as_u64().unwrap()).unwrap();
        let expected = if name == "frmsiz" {
            u32::try_from(target["target_frmsiz"].as_u64().unwrap()).unwrap()
        } else {
            observed
        };
        assert_eq!(read_bits(&bytes, start, width), expected, "D0 field {name}");
    }
}
