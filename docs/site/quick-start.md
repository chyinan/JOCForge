# Quick start

Install Rust and Cargo, then install the CLI:

```sh
git clone https://github.com/chyinan/JOCForge.git
cd JOCForge
cargo install --path crates/jocforge-cli --locked
```

Encode a 48 kHz ADM BWF to 5.1 at the default 1024 kbps. JOCForge requires absolute paths; `$PWD` works in PowerShell and POSIX shells:

```sh
jocforge encode "$PWD/input.adm.wav" "$PWD/output.ec3"
```

With OpenJOC installed, inspect the result:

```sh
openjoc inspect output.ec3 --emdf
```

For Flat 7.1 at the default 1664 kbps, use a fresh empty directory and the `.candidate.ec3` suffix:

```sh
mkdir flat7x
jocforge encode "$PWD/input.adm.wav" "$PWD/flat7x/mix.candidate.ec3" --joc-layout flat7x
```

![JOCForge CLI](assets/cli-help.png){ .jocforge-hero }

Layout, substream, carriage and container flags are in the encoding pages. `jocforge encode --help` lists them.
