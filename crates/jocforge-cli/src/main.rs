// pattern: Imperative Shell

use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [command] if command == "--help" || command == "-h" => {
            println!(
                "jocforge\nusage: jocforge --help | --version | encode\nencode is unavailable pending architecture review and implementation"
            );
            ExitCode::SUCCESS
        }
        [command] if command == "--version" || command == "-V" => {
            println!("jocforge {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [command, ..] if command == "encode" => match jocforge_application::encode() {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
        },
        _ => {
            eprintln!("invalid command: use --help");
            ExitCode::from(2)
        }
    }
}
