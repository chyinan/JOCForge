# Quick start

Install Rust and Cargo, then install the CLI:

```sh
git clone https://github.com/chyinan/JOCForge.git
cd JOCForge
cargo install --path crates/jocforge-cli --locked
```

Encode a 48 kHz ADM BWF to 5.1 at the default 1024 kbps:

```sh
jocforge encode input.adm.wav output.ec3
```

With OpenJOC installed, inspect the result:

```sh
openjoc inspect output.ec3 --emdf
```

For Flat 7.1 at the default 1664 kbps, add `--joc-layout flat7x`:

```sh
jocforge encode input.adm.wav output-7.1.ec3 --joc-layout flat7x
```

![JOCForge CLI](assets/cli-help.png){ .jocforge-hero }
