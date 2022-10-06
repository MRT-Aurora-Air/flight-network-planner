mod cmd;
mod types;
mod utils;

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use clap::Parser;
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
    subcmd: Subcmd,
}
#[derive(Parser)]
enum Subcmd {
    /// Run the planner
    Run(Run),
    /// Gets the configuration for the planner
    GetConfig,
    /// Tool to format the output as a mapping of gates to destinations
    GateKeys(GateKeys),
}

#[derive(Parser)]
struct Run {
    file: PathBuf,
    #[clap(short, long, value_parser, default_value = "out.txt")]
    output: PathBuf,
    #[clap(short, long, action)]
    stats: bool,
    #[clap(short, long, value_parser)]
    old: Option<PathBuf>,
}
#[derive(Parser)]
struct GateKeys {
    #[clap(default_value = "out.txt")]
    out_file: PathBuf,
}

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    let args = Args::parse();
    match args.subcmd {
        Subcmd::Run(run) => {
            let mut config: Config = serde_yaml::from_reader(std::fs::File::open(&run.file)?)?;
            let mut fd = FlightData::from_sheets()?;
            fd.preprocess(&mut config)?;
            let old_plan = if let Some(ref old) = run.old {
                Some(update::load_from_out(old.to_owned())?)
            } else {
                None
            };
            let mut result = run::run(&mut config, &fd, &old_plan)?;
            if run.stats {
                println!("{}", stats::get_stats(&result, &mut config)?)
            }
            if let Some(ref old) = run.old {
                result = update::update(old.to_owned(), result, &mut config)?;
            }
            std::fs::write(
                run.output,
                result
                    .into_iter()
                    .sorted_by_key(|f| f.flight_number)
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
        Subcmd::GetConfig => {
            println!("Hello, world!");
        }
        Subcmd::GateKeys(gate_keys) => {
            let flights = update::load_from_out(gate_keys.out_file)?;
            let mut map: HashMap<_, Vec<_>> = HashMap::new();
            for flight in flights {
                map.entry(flight.airport1)
                    .or_default()
                    .push((flight.airport2, flight.flight_number));
            }
            println!(
                "{}",
                map.iter()
                    .map(|((ka, kg), vs)| format!(
                        "{} {}: {}",
                        ka,
                        kg,
                        vs.iter()
                            .map(|((va, vg), num)| format!("{} {} {}", num, va, vg))
                            .join(", ")
                    ))
                    .sorted()
                    .join("\n")
            )
        }
    }
    Ok(())
}
