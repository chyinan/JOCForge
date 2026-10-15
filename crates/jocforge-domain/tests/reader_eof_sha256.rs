// pattern: Functional Core
use jocforge_domain::state::ReaderEofDigest;

#[test]
fn sha256_known_answers_match_fips_vectors() {
    assert_eq!(
        ReaderEofDigest::from_canonical_bytes(b"").bytes(),
        hex("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );
    assert_eq!(
        ReaderEofDigest::from_canonical_bytes(b"abc").bytes(),
        hex("ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad")
    );
}

fn hex(value: &str) -> [u8; 32] {
    let mut out = [0_u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        out[index] = (digit(pair[0]) << 4) | digit(pair[1]);
    }
    out
}

fn digit(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("invalid hex"),
    }
}
