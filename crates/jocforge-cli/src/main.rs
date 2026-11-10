// pattern: Imperative Shell

use jocforge_application::meter_source_bridge::{CodecRateProfile, JocReconstructionLayout};
use std::process::ExitCode;

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
                eprintln!("data_rate must be one explicit token: 384, 448, 576, 640, 768, or 1024");
                return ExitCode::from(2);
            }
        },
        None => CodecRateProfile::Kbps1024,
    };
    let joc_layout = match layout_token.map(parse_joc_layout_token) {
        Some(Some(layout)) => layout,
        Some(None) => {
            eprintln!("joc-layout must be flat7x");
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
            println!(
                "jocforge\nusage: jocforge --help | --version | encode <input.adm.wav> <output.ec3> [--data-rate <384|448|576|640|768|1024>] [--joc-layout <flat7x>] | encode --job-xml <job.xml> [--preflight-only]\nencode prepares one selected full-file ADM request; omitted --data-rate defaults to 1024 and omitted --joc-layout preserves the selected 5.1 profile\n--job-xml reads ADM BWF through either XML branch (adm or atmos_mezz) and EC-3 output; unsupported fields and containers reject\n--preflight-only admits XML and input metadata but does not start Meter or create output"
            );
            ExitCode::SUCCESS
        }
        [command] if command == "--version" || command == "-V" => {
            println!("jocforge {}", env!("CARGO_PKG_VERSION"));
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
            eprintln!("usage: jocforge encode --job-xml <job.xml> [--preflight-only]");
            ExitCode::from(2)
        }
        [command] if command == "encode" => {
            eprintln!(
                "usage: jocforge encode <input.adm.wav> <output.ec3> [--data-rate <384|448|576|640|768|1024>] | encode --job-xml <job.xml>"
            );
            ExitCode::from(2)
        }
        _ => {
            eprintln!("invalid command: use --help");
            ExitCode::from(2)
        }
    }
}
