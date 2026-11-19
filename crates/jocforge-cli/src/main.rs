// pattern: Imperative Shell

use clap::{
    Args, ColorChoice, CommandFactory, FromArgMatches, Parser, Subcommand,
    builder::styling::{RgbColor, Styles},
};
use clap_complete::Shell;
use jocforge_application::meter_source_bridge::{CodecRateProfile, JocReconstructionLayout};
use std::{fmt::Write as _, io::IsTerminal, path::PathBuf, process::ExitCode};

const ROOT_LOGO: &str = r"     ██╗ ██████╗  ██████╗███████╗ ██████╗ ██████╗  ██████╗ ███████╗
     ██║██╔═══██╗██╔════╝██╔════╝██╔═══██╗██╔══██╗██╔════╝ ██╔════╝
     ██║██║   ██║██║     █████╗  ██║   ██║██████╔╝██║  ███╗█████╗
██   ██║██║   ██║██║     ██╔══╝  ██║   ██║██╔══██╗██║   ██║██╔══╝
╚█████╔╝╚██████╔╝╚██████╗██║     ╚██████╔╝██║  ██║╚██████╔╝███████╗
 ╚════╝  ╚═════╝  ╚═════╝╚═╝      ╚═════╝ ╚═╝  ╚═════╝ ╚══════╝";

const ANSI_RESET: &str = "\x1b[0m";
const ROOT_DESCRIPTION: &str = "Selected ADM BWF to E-AC-3 JOC encoder";
const ROOT_LOGO_GRADIENT: [(u8, u8, u8); 4] = [
    (0x77, 0xd6, 0x5a),
    (0x20, 0xd6, 0xb5),
    (0x32, 0x95, 0xe8),
    (0x8a, 0x55, 0xe8),
];

#[derive(Parser)]
#[command(
    version,
    about = ROOT_DESCRIPTION,
    arg_required_else_help = true
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Encode one selected full-file ADM programme.
    #[command(
        after_help = "Flat7x writes a 1,664 kbps I0+D0 stream. Use a .candidate.ec3 file in a new empty attempt directory. XML jobs support ADM BWF input and raw EC-3 output."
    )]
    Encode(EncodeArgs),
    /// Generate a completion script for your shell.
    Completions { shell: Shell },
}

#[derive(Args)]
struct EncodeArgs {
    /// Input ADM BWF file.
    #[arg(value_name = "INPUT.adm.wav", required_unless_present = "job_xml")]
    input: Option<PathBuf>,
    /// Output EC-3 file.
    #[arg(value_name = "OUTPUT.ec3", required_unless_present = "job_xml")]
    output: Option<PathBuf>,
    /// Bitrate for direct file input. Defaults to 1024 kbps.
    #[arg(long, value_name = "KBPS", value_parser = ["384", "448", "576", "640", "768", "1024"])]
    data_rate: Option<String>,
    /// Select experimental Flat7x. Defaults to 5.1.
    #[arg(long, value_name = "LAYOUT", value_parser = ["flat7x"])]
    joc_layout: Option<String>,
    /// Read input and output paths from an XML job.
    #[arg(long, value_name = "JOB.xml", conflicts_with_all = ["input", "output", "data_rate", "joc_layout"])]
    job_xml: Option<PathBuf>,
    /// Validate the XML job and input metadata without starting the encoder.
    #[arg(long, requires = "job_xml")]
    preflight_only: bool,
}

fn main() -> ExitCode {
    let color = terminal_color_enabled();
    let styles = Styles::styled()
        .header(RgbColor(32, 214, 181).on_default())
        .usage(RgbColor(32, 214, 181).on_default());
    let mut command = Cli::command()
        .styles(styles)
        .color(if color {
            ColorChoice::Always
        } else {
            ColorChoice::Never
        })
        .before_help(render_root_logo(color))
        .about(if color {
            format!("\x1b[38;2;119;214;90m{ROOT_DESCRIPTION}{ANSI_RESET}")
        } else {
            ROOT_DESCRIPTION.to_owned()
        });

    let cli = command
        .clone()
        .try_get_matches()
        .and_then(|matches| Cli::from_arg_matches(&matches))
        .unwrap_or_else(|error| error.exit());
    let result = match cli.command {
        Command::Completions { shell } => {
            clap_complete::generate(shell, &mut command, "jocforge", &mut std::io::stdout());
            return ExitCode::SUCCESS;
        }
        Command::Encode(args) => {
            if let Some(job_xml) = args.job_xml {
                if args.preflight_only {
                    jocforge::encode_request::preflight_from_job_xml(&job_xml)
                } else {
                    jocforge::encode_request::run_from_job_xml(&job_xml)
                }
            } else {
                let rate =
                    CodecRateProfile::from_xml_token(args.data_rate.as_deref().unwrap_or("1024"))
                        .expect("clap validates the bitrate");
                let layout = if args.joc_layout.is_some() {
                    JocReconstructionLayout::Flat7x
                } else {
                    JocReconstructionLayout::ExistingSelectedFivePointOne
                };
                jocforge::encode_request::run_with_data_rate_and_joc_layout(
                    &args.input.expect("clap requires an input path"),
                    &args.output.expect("clap requires an output path"),
                    rate,
                    layout,
                )
            }
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(2)
        }
    }
}

fn terminal_color_enabled() -> bool {
    should_use_color(
        std::io::stdout().is_terminal(),
        std::env::var_os("NO_COLOR").is_some(),
        std::env::var("TERM").ok().as_deref(),
    )
}

fn should_use_color(is_tty: bool, no_color: bool, term: Option<&str>) -> bool {
    is_tty && !no_color && !term.is_some_and(|value| value.eq_ignore_ascii_case("dumb"))
}

fn render_root_logo(color: bool) -> String {
    if !color {
        return ROOT_LOGO.to_owned();
    }

    let lines: Vec<_> = ROOT_LOGO.lines().collect();
    let span = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(1);
    let mut output = String::with_capacity(ROOT_LOGO.len() + 256);
    for (line_index, line) in lines.iter().enumerate() {
        for (column, character) in line.chars().enumerate() {
            if character == ' ' {
                output.push(character);
                continue;
            }
            let (red, green, blue) = gradient_color(column, span);
            write!(output, "\x1b[38;2;{red};{green};{blue}m{character}")
                .expect("writing to a String cannot fail");
        }
        output.push_str(ANSI_RESET);
        if line_index + 1 < lines.len() {
            output.push('\n');
        }
    }
    output
}

fn gradient_color(column: usize, span: usize) -> (u8, u8, u8) {
    let denominator = span.saturating_sub(1).max(1);
    let scaled = column.saturating_mul(ROOT_LOGO_GRADIENT.len() - 1);
    let segment = (scaled / denominator).min(ROOT_LOGO_GRADIENT.len() - 2);
    let position = scaled.saturating_sub(segment * denominator);
    interpolate_color(
        ROOT_LOGO_GRADIENT[segment],
        ROOT_LOGO_GRADIENT[segment + 1],
        position,
        denominator,
    )
}

fn interpolate_color(
    start: (u8, u8, u8),
    end: (u8, u8, u8),
    position: usize,
    length: usize,
) -> (u8, u8, u8) {
    let position = u32::try_from(position).expect("logo width fits in u32");
    let length = u32::try_from(length).expect("logo width fits in u32");
    let remaining = length - position;
    let channel = |start: u8, end: u8| {
        let value = u32::from(start) * remaining + u32::from(end) * position;
        u8::try_from((value + length / 2) / length).expect("interpolated RGB channel fits in u8")
    };
    (
        channel(start.0, end.0),
        channel(start.1, end.1),
        channel(start.2, end.2),
    )
}
