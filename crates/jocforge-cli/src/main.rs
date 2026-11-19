// pattern: Imperative Shell

use jocforge_application::meter_source_bridge::{CodecRateProfile, JocReconstructionLayout};
use std::process::ExitCode;

const ROOT_LOGO: &str = r#"     ██╗ ██████╗  ██████╗███████╗ ██████╗ ██████╗  ██████╗ ███████╗
     ██║██╔═══██╗██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
     ██║██║   ██║██║     █████╗  ██║   ██║██████╔╝██║  ███╗█████╗
██   ██║██║   ██║██║     ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
╚█████╔╝╚██████╔╝╚██████╗██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
 ╚════╝  ╚═════╝  ╚═════╝╚═╝      ╚═════╝ ╚═╝  ╚═════╝ ╚══════╝"#;

const ROOT_HELP: &str = r#"
Selected ADM BWF to E-AC-3 JOC encoder

USAGE
    jocforge --help
    jocforge --version
    jocforge encode <INPUT.adm.wav> <OUTPUT.ec3> [OPTIONS]
    jocforge encode --job-xml <JOB.xml> [OPTIONS]

COMMANDS
    encode    Encode one selected full-file ADM programme

OPTIONS
    -h, --help       Print help
    -V, --version    Print the package version

Run `jocforge encode --help` for encode options.
"#;

const ENCODE_HELP: &str = r#"
Encode a selected ADM programme as E-AC-3 JOC.

USAGE
    jocforge encode <INPUT.adm.wav> <OUTPUT.ec3> [OPTIONS]
    jocforge encode --job-xml <JOB.xml> [OPTIONS]

OPTIONS
    --data-rate <KBPS>        Select 384, 448, 576, 640, 768, or 1024
                              Default is 1024. Applies to direct file input.
    --joc-layout <LAYOUT>     Select `flat7x`. Omit to retain selected 5.1.
    --job-xml <JOB.xml>        Read supported ADM BWF input and EC-3 output fields.
    --preflight-only           Validate the job and input metadata without Meter or output creation.
                              Use with `--job-xml`.
    -h, --help                 Print this help

PROFILES AND OUTPUT
    Default profile is selected 5.1 at 1024 kbps.
    `--joc-layout flat7x` writes an experimental 1,664 kbps I0+D0 stream.
    Flat7x output requires a `.candidate.ec3` file in a new empty attempt directory.

JOB XML
    ADM and atmos_mezz input branches use the ADM BWF reader.
    XML output currently supports EC-3 elementary streams.
    MP4 and DASH output containers are not admitted by this command.
"#;

fn print_root_help() {
    println!(
        "{ROOT_LOGO}\nJOCForge {}{ROOT_HELP}",
        env!("CARGO_PKG_VERSION")
    );
}

fn print_encode_help() {
    println!("JOCForge encode{ENCODE_HELP}");
}

fn parse_joc_layout_token(value: &str) -> Option<JocReconstructionLayout> {
    match value {
        "flat7x" => Some(JocReconstructionLayout::Flat7x),
        _ => None,
    }
}

fn run_full_file_encode(
    input_path: &str,
    output_path: &str,
    rate_token: Option<&str>,
    layout_token: Option<&str>,
) -> ExitCode {
    let codec_rate = match rate_token {
        Some(value) => match CodecRateProfile::from_xml_token(value) {
            Some(rate) => rate,
            None => {
                eprintln!(
                    "error: invalid value '{value}' for '--data-rate'\n       valid values: 384, 448, 576, 640, 768, 1024"
                );
                return ExitCode::from(2);
            }
        },
        None => CodecRateProfile::Kbps1024,
    };
    let joc_layout = match layout_token.map(parse_joc_layout_token) {
        Some(Some(layout)) => layout,
        Some(None) => {
            eprintln!(
                "error: invalid value '{}' for '--joc-layout'\n       valid value: flat7x",
                layout_token.unwrap_or_default()
            );
            return ExitCode::from(2);
        }
        None => JocReconstructionLayout::ExistingSelectedFivePointOne,
    };

    match jocforge::encode_request::run_with_data_rate_and_joc_layout(
        std::path::Path::new(input_path),
        std::path::Path::new(output_path),
        codec_rate,
        joc_layout,
    ) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.as_slice() {
        [command] if command == "--help" || command == "-h" => {
            print_root_help();
            ExitCode::SUCCESS
        }
        [command] if command == "--version" || command == "-V" => {
            println!("jocforge {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        [command, option] if command == "encode" && (option == "--help" || option == "-h") => {
            print_encode_help();
            ExitCode::SUCCESS
        }
        [command, option, job_xml_path, preflight]
            if command == "encode" && option == "--job-xml" && preflight == "--preflight-only" =>
        {
            match jocforge::encode_request::preflight_from_job_xml(std::path::Path::new(
                job_xml_path,
            )) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
            }
        }
        [command, option, job_xml_path] if command == "encode" && option == "--job-xml" => {
            match jocforge::encode_request::run_from_job_xml(std::path::Path::new(job_xml_path)) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
            }
        }
        [command, input_path, output_path, option, value]
            if command == "encode" && option == "--data-rate" =>
        {
            run_full_file_encode(input_path, output_path, Some(value), None)
        }
        [command, input_path, output_path, option, value]
            if command == "encode" && option == "--joc-layout" =>
        {
            run_full_file_encode(input_path, output_path, None, Some(value))
        }
        [
            command,
            input_path,
            output_path,
            rate_option,
            rate_value,
            layout_option,
            layout_value,
        ] if command == "encode"
            && rate_option == "--data-rate"
            && layout_option == "--joc-layout" =>
        {
            run_full_file_encode(
                input_path,
                output_path,
                Some(rate_value),
                Some(layout_value),
            )
        }
        [
            command,
            input_path,
            output_path,
            layout_option,
            layout_value,
            rate_option,
            rate_value,
        ] if command == "encode"
            && layout_option == "--joc-layout"
            && rate_option == "--data-rate" =>
        {
            run_full_file_encode(
                input_path,
                output_path,
                Some(rate_value),
                Some(layout_value),
            )
        }
        [command, input_path, output_path] if command == "encode" => {
            run_full_file_encode(input_path, output_path, None, None)
        }
        [command, option] if command == "encode" && option == "--job-xml" => {
            eprintln!(
                "error: missing <JOB.xml>\n\nUSAGE\n    jocforge encode --job-xml <JOB.xml> [OPTIONS]\n\nFor more information, try 'jocforge encode --help'."
            );
            ExitCode::from(2)
        }
        [command] if command == "encode" => {
            eprintln!(
                "error: missing input and output\n\nUSAGE\n    jocforge encode <INPUT.adm.wav> <OUTPUT.ec3> [OPTIONS]\n    jocforge encode --job-xml <JOB.xml> [OPTIONS]\n\nFor more information, try 'jocforge encode --help'."
            );
            ExitCode::from(2)
        }
        _ => {
            eprintln!(
                "error: unrecognized command\n\nFor more information, try 'jocforge --help'."
            );
            ExitCode::from(2)
        }
    }
}
