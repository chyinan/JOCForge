// pattern: Imperative Shell
//! Emits a synthetic, six-access-unit ADM for the WASM topology regression.
#[path = "../crates/jocforge-cli/tests/support/synthetic_adm.rs"]
mod synthetic_adm;

fn main() -> std::io::Result<()> {
    let output = std::env::args_os()
        .nth(1)
        .expect("usage: generate-wasm-test-adm OUTPUT.wav");
    std::fs::write(output, synthetic_adm::synthetic_cue(192))
}
