mod cmd;
mod types;
mod utils;

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::Shell;
use clap_complete_fig::Fig;
use itertools::Itertools;
use types::config::Config;

use crate::{
    cmd::{run, stats, update},
    types::flight_data::FlightData,
};

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}
#[derive(Parser)]
enum Command {
    /// Run the planner
    Run(Run),
    /// Gets the configuration for the planner
    GetConfig,
    /// Tool to format the output of `run` as a mapping of gates to destinations
    GateKeys(GateKeys),
    /// Generate a completion file for your shell
    Completion(Completion),
}

#[derive(Parser)]
struct Run {
    /// The configuration YML file to read from
    file: PathBuf,
    /// Whether to print statistics
    #[clap(short, long, action)]
    stats: bool,
    /// The old output file
    /// (will be used to preserve original flight routes so it won't duplicate so much)
    #[clap(short, long, value_parser)]
    old: Option<PathBuf>,
    /// Whether to replace the old file instead of printing to stdout
    #[clap(short, long, action)]
    replace: bool,
}

#[derive(Parser)]
struct GateKeys {
    /// The output file from `run`
    #[clap(default_value = "out.txt")]
    out_file: PathBuf,
}

#[derive(Parser)]
struct Completion {
    /// The shell to generate for
    #[arg(value_enum)]
    shell: Shell,
    /// Whether to generate for Fig instead
    #[clap(short, long, action)]
    fig: bool,
}

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    let args = Args::parse();
    match args.command {
        Command::Run(run) => {
            let mut config: Config = serde_yaml::from_reader(std::fs::File::open(&run.file)?)?;
            config._folder = run.file.parent().map(ToOwned::to_owned);
            let mut fd = FlightData::from_gatelogue()?;
            fd.preprocess(&mut config)?;
            let old_plan = if let Some(old) = &run.old {
                Some(update::load_from_out(old.to_owned())?)
            } else {
                None
            };
            let mut result = run::run(&mut config, &fd, &old_plan)?;
            if run.stats {
                eprintln!("\n{}", stats::get_stats(&result, &mut config)?);
            }
            if let Some(old) = &run.old {
                result = update::update(old.to_owned(), result, &mut config)?;
            }
            let res = result
                .into_iter()
                .sorted_by_key(|f| f.flight_number)
                .map(|f| f.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            if run.replace {
                if let Some(old) = &run.old {
                    std::fs::write(old, res)?;
                    println!("Overwritten {}", old.display());
                } else {
                    println!("{res}");
                }
            } else {
                println!("{res}");
            }
        }
        Command::GetConfig => {
            println!("{}", include_str!("../data/default_config.yml"));
        }
        Command::GateKeys(gate_keys) => {
            let flights = update::load_from_out(gate_keys.out_file)?;
            let mut map: HashMap<_, Vec<_>> = HashMap::new();
            for flight in flights {
                map.entry(flight.airport1)
                    .or_default()
                    .push((flight.airport2, flight.flight_number));
            }
            let res = map
                .iter()
                .map(|((ka, kg), vs)| {
                    format!(
                        "{} {}: {}",
                        ka,
                        kg,
                        vs.iter()
                            .map(|((va, vg), num)| format!("{num} {va} {vg}"))
                            .join(", ")
                    )
                })
                .sorted()
                .join("\n");
            println!("{res}");
        }
        Command::Completion(completion) => {
            let mut cmd = Args::command();
            let name = cmd.get_name().to_owned();
            if completion.fig {
                clap_complete::generate(Fig, &mut cmd, name, &mut std::io::stdout());
            } else {
                clap_complete::generate(completion.shell, &mut cmd, name, &mut std::io::stdout());
            }
        }
    }
    Ok(())
}
