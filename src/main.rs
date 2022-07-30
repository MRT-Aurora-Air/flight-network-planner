#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcmd: Subcmd,
    /// Enables debugging info
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,
}

fn main() {
    println!("Hello, world!");
}
