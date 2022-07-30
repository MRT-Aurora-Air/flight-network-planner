mod config;
mod flight_data;
mod run;
mod types;

use crate::flight_data::FlightData;
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

fn main() {
    let args = Args::parse();
    match args.subcmd {
        Subcmd::Run(run) => {
            println!("{:#?}", FlightData::from_sheets().unwrap());
        }
        Subcmd::GetConfig => {
            println!("Hello, world!");
        }
    }
}
