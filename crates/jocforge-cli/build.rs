use std::{env, fs, io::Write, path::Path};

const BUNDLES: &[&str] = &[
    "ADM-F03W-APPROVED-PROVIDER-BUNDLE-V1",
    "F01B-F03W-COLD-STATE-ASSETS-V1",
    "METER-APPROVED-PARAMETER-ASSETS-V1",
    "METER-PROVIDER-SET-IDENTITY-APPLICATION-V1",
    "F04-RECEIVER-APPROVED-PARAMETERS-V1",
];

fn main() -> std::io::Result<()> {
    let root = Path::new("../../outputs/IMPLEMENTER_SUPPLEMENTS").canonicalize()?;
    let out = env::var_os("OUT_DIR").expect("Cargo sets OUT_DIR");
    let mut index = fs::File::create(Path::new(&out).join("provider_assets.rs"))?;
    writeln!(index, "&[")?;
    for bundle in BUNDLES {
        let path = root.join(bundle);
        println!("cargo:rerun-if-changed={}", path.display());
        append_files(&root, &path, &mut index)?;
    }
    writeln!(index, "]")
}

fn append_files(root: &Path, directory: &Path, index: &mut fs::File) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type()?.is_dir() {
            append_files(root, &path, index)?;
        } else if entry.file_type()?.is_file() {
            let relative = path.strip_prefix(root).expect("entry is below root");
            writeln!(
                index,
                "({:?}, include_bytes!({:?})),",
                relative.to_str().unwrap(),
                path.to_str().unwrap()
            )?;
        }
    }
    Ok(())
}
