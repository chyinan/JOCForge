// pattern: Imperative Shell
use jocforge_domain::frontend::selected_adm_graph::{SelectedAdmGraph, parse_selected_adm_graph};
use std::fmt::Write;

// Synthetic ten-speaker bed with an interleaved object at physical host 2.
fn metadata() -> (String, Vec<[u8; 40]>) {
    let labels = ["L", "R", "C", "LFE", "Ls", "Rs", "Lrs", "Rrs", "Ltm", "Rtm"];
    let mut xml = String::from("<ebuCoreMain><coreMetadata><format><audioFormatExtended>");
    let mut rows = Vec::new();
    for (kind, name, object_id) in [(1, "DirectSpeakers", "1001"), (3, "Objects", "100B")] {
        let pack = format!("AP_{kind:04X}1001");
        let channels: &[&str] = if kind == 1 { &labels } else { &["Narrator"] };
        let mut channels_xml = String::new();
        let mut pack_channels = String::new();
        let mut object_tracks = String::new();
        for (index, &label) in channels.iter().enumerate() {
            let slot = if kind == 3 {
                2
            } else {
                index + usize::from(index >= 2)
            };
            let uid = format!("ATU_{:08X}", slot + 1);
            let id = format!("{kind:04X}{:04X}", 0x1001 + index);
            let track = format!("AT_{id}_01");
            let channel = format!("AC_{id}");
            let body = if kind == 1 {
                format!("<speakerLabel>RC_{label}</speakerLabel>")
            } else {
                "<cartesian>1</cartesian><position coordinate=\"X\">0</position><position coordinate=\"Y\">1</position><position coordinate=\"Z\">0</position>".into()
            };
            write!(
                pack_channels,
                "<audioChannelFormatIDRef>{channel}</audioChannelFormatIDRef>"
            )
            .unwrap();
            write!(object_tracks, "<audioTrackUIDRef>{uid}</audioTrackUIDRef>").unwrap();
            write!(channels_xml, r#"<audioChannelFormat audioChannelFormatID="{channel}" typeDefinition="{name}"><audioBlockFormat rtime="00:00:00.00000" duration="00:00:00.03200">{body}</audioBlockFormat></audioChannelFormat><audioTrackFormat audioTrackFormatID="{track}"><audioStreamFormatIDRef>AS_{id}</audioStreamFormatIDRef></audioTrackFormat><audioStreamFormat audioStreamFormatID="AS_{id}"><audioChannelFormatIDRef>{channel}</audioChannelFormatIDRef></audioStreamFormat><audioTrackUID UID="{uid}"><audioTrackFormatIDRef>{track}</audioTrackFormatIDRef><audioPackFormatIDRef>{pack}</audioPackFormatIDRef></audioTrackUID>"#).unwrap();
            let mut row = [0u8; 40];
            row[..2].copy_from_slice(&u16::try_from(slot + 1).unwrap().to_le_bytes());
            row[2..14].copy_from_slice(uid.as_bytes());
            row[14..28].copy_from_slice(track.as_bytes());
            row[28..39].copy_from_slice(pack.as_bytes());
            rows.push(row);
        }
        write!(xml, r#"<audioObject audioObjectID="AO_{object_id}" start="00:00:00.00000" duration="00:00:00.03200"><audioPackFormatIDRef>{pack}</audioPackFormatIDRef>{object_tracks}</audioObject><audioPackFormat audioPackFormatID="{pack}" typeDefinition="{name}">{pack_channels}</audioPackFormat>{channels_xml}"#).unwrap();
    }
    xml.push_str("</audioFormatExtended></format></coreMetadata></ebuCoreMain>");
    rows.sort_by_key(|row| row[0]);
    (xml, rows)
}

fn graph(xml: &str, rows: &[[u8; 40]]) -> SelectedAdmGraph {
    let mut chna = vec![11, 0, 11, 0];
    chna.extend(rows.iter().flatten());
    parse_selected_adm_graph(xml.as_bytes(), &chna).unwrap()
}

#[test]
fn chna_record_order_and_unused_capacity_do_not_move_physical_tracks() {
    let (xml, rows) = metadata();
    let reference = graph(&xml, &rows);
    assert_eq!(reference.bed_channel_join().lfe_host(), 4);
    assert_eq!(reference.bed_channel_join().object_hosts(), &[2]);
    assert_eq!(
        reference.bed_channel_join().slots_by_pack(),
        &[0, 1, 3, 4, 5, 6, 7, 8, 9, 10]
    );
    let mut shuffled = rows.clone();
    shuffled.rotate_left(3);
    let mut reserved = rows;
    reserved.insert(2, [0; 40]);
    reserved.push([0; 40]);
    for variant in [shuffled, reserved] {
        let actual = graph(&xml, &variant);
        assert_eq!(actual.tracks(), reference.tracks());
        assert_eq!(actual.bed_channel_join(), reference.bed_channel_join());
        assert_eq!(
            actual.regular_source_bindings(),
            reference.regular_source_bindings()
        );
        assert_eq!(
            actual.object_sampler_owner_bindings(),
            reference.object_sampler_owner_bindings()
        );
        assert_ne!(
            actual.receipt().chna_sha256(),
            reference.receipt().chna_sha256()
        );
    }
}
