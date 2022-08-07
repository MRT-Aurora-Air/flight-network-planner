mod config;
mod flight;
mod flight_data;
mod flight_type;
mod fng;
mod gate;
mod run;
mod stats;
mod types;

use crate::config::Config;
use crate::flight_data::FlightData;
use anyhow::Result;
use clap::Parser;
use itertools::Itertools;

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
}

#[derive(Parser)]
struct Run {
    file: String,
    #[clap(short, long, value_parser, default_value = "out.txt")]
    output: String,
    #[clap(short, long, action)]
    stats: bool,
}

fn main() -> Result<()> {
    pretty_env_logger::try_init()?;
    let args = Args::parse();
    match args.subcmd {
        Subcmd::Run(run) => {
            let mut config: Config = serde_yaml::from_reader(std::fs::File::open(&run.file)?)?;
            let mut fd = FlightData::from_sheets()?;
            fd.preprocess(&mut config)?;
            let result = run::run(&mut config, &fd)?;
            if run.stats {
                println!("{}", stats::get_stats(&result, &mut config)?)
            }
            std::fs::write(
                run.output,
                result
                    .into_iter()
                    .sorted_by_key(|(f, _, _)| f.flight_number)
                    .map(|(f, s, ty)| format!("{} ({}, {})", f, s, ty))
                    .collect::<Vec<_>>()
                    .join("\n"),
            )?;
        }
        Subcmd::GetConfig => {
            println!("Hello, world!");
        }
    }
    Ok(())
}
