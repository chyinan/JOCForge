// pattern: Imperative Shell

use jocforge_application::meter_source_bridge::CodecRateProfile;
use std::process::ExitCode;

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [command] if command == "--help" || command == "-h" => {
            println!(
                "jocforge\nusage: jocforge --help | --version | encode <input.adm.wav> <output.ec3> [--data-rate <384|448|576|640|768|1024>]\nencode prepares one selected full-file ADM request; omitted --data-rate defaults to 1024"
            );
            ExitCode::SUCCESS
        }
        [command] if command == "--version" || command == "-V" => {
            println!("jocforge {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [command, input_path, output_path, option, value]
            if command == "encode" && option == "--data-rate" =>
        {
            let Some(codec_rate) = CodecRateProfile::from_xml_token(value) else {
                eprintln!("data_rate must be one explicit token: 384, 448, 576, 640, 768, or 1024");
                return ExitCode::from(2);
            };
            match jocforge::encode_request::run_with_data_rate(
                std::path::Path::new(input_path),
                std::path::Path::new(output_path),
                codec_rate,
            ) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
            }
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
