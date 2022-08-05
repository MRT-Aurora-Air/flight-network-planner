mod config;
mod flight_data;
mod run;
mod types;

use crate::config::Config;
use crate::flight_data::FlightData;
use anyhow::Result;
use clap::Parser;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcmd: Subcmd,
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,
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
    #[clap(short, long, value_parser, default_value = "plan.txt")]
    output: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    match args.subcmd {
        Subcmd::Run(run) => {
            let config: Config = serde_yaml::from_reader(std::fs::File::open(&run.file)?)?;
            run::run(config)?;
        }
        Subcmd::GetConfig => {
            println!("Hello, world!");
        }
    }
    Ok(())
}
