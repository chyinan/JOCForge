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
        after_help = "Flat7x and idx2 default to 1664 kbps (640 kbps I0 + 1024 kbps D0). Use a .candidate.ec3 file in a new empty attempt directory. XML jobs support ADM BWF input and raw EC-3 output."
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
    /// Output elementary EC-3 or fragmented CMAF file.
    #[arg(value_name = "OUTPUT.ec3", required_unless_present = "job_xml")]
    output: Option<PathBuf>,
    /// Carrier bitrate (D0 for paired streams). Defaults to 1024 kbps.
    #[arg(long, value_name = "KBPS", value_parser = ["384", "448", "576", "640", "768", "1024"])]
    data_rate: Option<String>,
    /// Total bitrate for paired elementary streams or the CMAF track.
    #[arg(long, value_name = "KBPS", conflicts_with = "data_rate", value_parser = ["1024", "1088", "1216", "1280", "1408", "1536", "1664"])]
    total_rate: Option<String>,
    /// Select idx0 through idx4; flat7x is an alias for idx1. Defaults to idx3.
    #[arg(long, value_name = "LAYOUT", value_parser = ["flat7x","idx0","idx1","idx2","idx3","idx4"])]
    joc_layout: Option<String>,
    /// Write strict ETSI JOC/OAMD EMDF signalling for the selected stream.
    #[arg(long)]
    etsi_strict: bool,
    /// Output container. CMAF writes a fragmented ec-3 track, default 1664 kbps.
    #[arg(long,value_parser=["raw","cmaf"],default_value="raw")]
    container: String,
    /// EMDF carrier: audio-block skip field or dependent frame-end auxdata (etsi or legacy-ac3).
    #[arg(long,value_parser=["skipfld","auxdata","etsi","legacy-ac3"])]
    carriage: Option<String>,
    /// EMDF signalling, independent of the carrier location.
    #[arg(long,value_parser=["etsi","strict","deployed"],conflicts_with="etsi_strict")]
    emdf_profile: Option<String>,
    /// Split each 1536-sample programme interval into equal E-AC-3 syncframes.
    #[arg(long, value_name = "6|3+3|2+2+2|1x6")]
    block_partition: Option<jocforge::encode_request::BlockPartition>,
    /// Emit this many dependent E-AC-3 frames after each independent frame.
    #[arg(long,value_parser=clap::value_parser!(u8).range(0..=6))]
    dependent_count: Option<u8>,
    /// Place programme LFE in I0, D0, both (D0 replaces I0), or neither.
    #[arg(long,value_parser=["independent","dependent","replace-independent","none"])]
    lfe_owner: Option<String>,
    /// Read input and output paths from an XML job.
    #[arg(long, value_name = "JOB.xml", conflicts_with_all = ["input", "output", "data_rate"])]
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
            let layout = match args.joc_layout.as_deref() {
                None | Some("idx3") => JocReconstructionLayout::ExistingSelectedFivePointOne,
                Some("idx0") => JocReconstructionLayout::FivePointOneWithoutPhase,
                Some("flat7x" | "idx1") => JocReconstructionLayout::Flat7x,
                Some("idx2") => JocReconstructionLayout::FivePointOnePlusHeight,
                Some("idx4") => JocReconstructionLayout::HeightPairWithPhase,
                _ => unreachable!("clap validates the JOC layout"),
            };
            let ac3_core = args.carriage.as_deref() == Some("legacy-ac3");
            if ac3_core
                && (args.total_rate.as_deref().is_some_and(|v| v != "1280")
                    || args.data_rate.as_deref().is_some_and(|v| v != "640"))
            {
                eprintln!("legacy-ac3 requires --total-rate 1280 or --data-rate 640");
                return ExitCode::from(2);
            }
            let cmaf = args.container == "cmaf";
            if !cmaf
                && args.total_rate.is_some()
                && (args.joc_layout.is_none() || args.job_xml.is_some())
            {
                eprintln!(
                    "raw --total-rate requires --joc-layout and cannot be used with --job-xml"
                );
                return ExitCode::from(2);
            }
            if cmaf
                && (args.emdf_profile.as_deref() == Some("deployed")
                    || args
                        .carriage
                        .as_deref()
                        .is_some_and(|v| !matches!(v, "etsi" | "auxdata"))
                    || args.dependent_count.is_some_and(|v| v != 1)
                    || args
                        .lfe_owner
                        .as_deref()
                        .is_some_and(|v| !matches!(v, "none" | "independent" | "dependent"))
                    || args
                        .block_partition
                        .is_some_and(|v| v != jocforge::encode_request::BlockPartition::Six)
                    || args.data_rate.as_deref().is_some_and(|v| v != "640"))
            {
                eprintln!(
                    "CMAF requires strict EMDF, six blocks, one auxdata dependent at 640 kbps and LFE owner none, independent or dependent"
                );
                return ExitCode::from(2);
            }
            if !cmaf && args.total_rate.as_deref() == Some("1536") {
                eprintln!("1536 kbps is available for CMAF");
                return ExitCode::from(2);
            }
            if args.total_rate.is_some() && !layout.uses_seven_carriers() && !ac3_core && !cmaf {
                eprintln!("--total-rate requires a paired stream (idx1, idx2 or idx4)");
                return ExitCode::from(2);
            }
            use jocforge::encode_request::BlockPartition;
            use jocforge_application::meter_source_bridge::EmdfCarriageProfile;
            let partition = args.block_partition.unwrap_or(BlockPartition::Six);
            let auxdata = ac3_core || matches!(args.carriage.as_deref(), Some("auxdata" | "etsi"));
            let compose = cmaf
                || auxdata
                || partition != BlockPartition::Six
                || args.dependent_count.is_some()
                || args.lfe_owner.is_some();
            let signalling = if args.emdf_profile.as_deref() == Some("deployed") {
                EmdfCarriageProfile::DeployedCompat
            } else if args.emdf_profile.is_some()
                || args.etsi_strict
                || compose
                || layout.uses_height_pair()
                || layout == JocReconstructionLayout::FivePointOneWithoutPhase
            {
                EmdfCarriageProfile::EtsiEmdf
            } else {
                EmdfCarriageProfile::DeployedCompat
            };
            use jocforge::encode_request::{LfeOwner, SubstreamPlan};
            let topology = if cmaf {
                let rate = args
                    .total_rate
                    .as_deref()
                    .unwrap_or("1664")
                    .parse()
                    .expect("validated total bitrate");
                let lfe = match args.lfe_owner.as_deref() {
                    Some("dependent") => LfeOwner::Dependent,
                    Some("independent") => LfeOwner::Independent,
                    _ => LfeOwner::None,
                };
                match SubstreamPlan::cmaf_with_lfe(layout, rate, lfe) {
                    Ok(plan) => Some(plan),
                    Err(e) => {
                        eprintln!("{e}");
                        return ExitCode::from(2);
                    }
                }
            } else if compose {
                let owner = match args.lfe_owner.as_deref() {
                    Some("none") => LfeOwner::None,
                    Some("dependent") => LfeOwner::Dependent,
                    Some("replace-independent") => LfeOwner::ReplaceIndependent,
                    _ => LfeOwner::Independent,
                };
                match SubstreamPlan::new(
                    layout,
                    args.dependent_count.unwrap_or(u8::from(
                        auxdata
                            || layout.uses_seven_carriers()
                            || matches!(owner, LfeOwner::Dependent | LfeOwner::ReplaceIndependent),
                    )),
                    owner,
                )
                .and_then(|plan| {
                    plan.with_emdf_location(if auxdata {
                        jocforge::encode_request::EmdfLocation::FrameEnd
                    } else {
                        jocforge::encode_request::EmdfLocation::SkipField
                    })
                })
                .and_then(|plan| {
                    let plan = plan.with_partition(partition);
                    if ac3_core {
                        plan.with_ac3_core()
                    } else {
                        Ok(plan)
                    }
                }) {
                    Ok(plan) => Some(plan),
                    Err(e) => {
                        eprintln!("{e}");
                        return ExitCode::from(2);
                    }
                }
            } else {
                None
            };
            if let Some(job_xml) = args.job_xml {
                if let Some(plan) = topology {
                    jocforge::encode_request::encode_xml_substream_layout(
                        &job_xml,
                        layout,
                        signalling,
                        args.preflight_only,
                        plan,
                    )
                } else {
                    jocforge::encode_request::encode_job_xml_with_signalling(
                        &job_xml,
                        layout,
                        args.preflight_only,
                        signalling,
                    )
                }
            } else {
                let rate = if cmaf {
                    CodecRateProfile::Kbps640
                } else if let Some(total) = args.total_rate {
                    let kbps = total.parse::<u32>().expect("validated total bitrate");
                    CodecRateProfile::from_kbps(kbps - 640).expect("validated dependent bitrate")
                } else {
                    CodecRateProfile::from_xml_token(
                        args.data_rate
                            .as_deref()
                            .unwrap_or(if ac3_core { "640" } else { "1024" }),
                    )
                    .expect("clap validates the bitrate")
                };
                if let Some(plan) = topology {
                    jocforge::encode_request::encode_substream_layout(
                        args.input.as_deref().unwrap(),
                        args.output.as_deref().unwrap(),
                        rate,
                        layout,
                        signalling,
                        plan,
                    )
                } else {
                    jocforge::encode_request::run_with_output_signalling(
                        &args.input.expect("clap requires an input path"),
                        &args.output.expect("clap requires an output path"),
                        rate,
                        layout,
                        signalling,
                    )
                }
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
