pub mod cli;
pub mod config;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();
    let config = args.parse_config()?;

    println!("{:#?}", config);

    match &args.commands {
        cli::Subcommands::Build { profile, build } => config.build_packs(&args, &profile, &build),
    }
}
