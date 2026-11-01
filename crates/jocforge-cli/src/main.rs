// pattern: Imperative Shell

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [command] if command == "--help" || command == "-h" => {
            println!(
                "jocforge\nusage: jocforge --help | --version | encode <input.adm.wav> <output.ec3>\nencode prepares one selected full-file ADM request, then requires production owner admissions before writing"
            );
            ExitCode::SUCCESS
        }
        [command] if command == "--version" || command == "-V" => {
            println!("jocforge {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [command, input_path, output_path] if command == "encode" => {
            match jocforge::encode_request::run(
                std::path::Path::new(input_path),
                std::path::Path::new(output_path),
            ) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
            }
        }
        [command] if command == "encode" => {
            eprintln!("usage: jocforge encode <input.adm.wav> <output.ec3>");
            ExitCode::from(2)
        }
        _ => {
            eprintln!("invalid command: use --help");
            ExitCode::from(2)
        }
    }
}
