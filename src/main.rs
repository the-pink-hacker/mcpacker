pub mod cli;
pub mod compile;
pub mod config;
pub mod minecraft;
pub mod runner;

use clap::Parser;
use cli::Args;
use runner::Runner;

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match &args.commands {
        cli::Subcommands::Build {
            profile,
            builds,
            listen,
        } => {
            let runner = Runner::new(
                args.config,
                args.compile,
                args.minecraft,
                builds.to_vec(),
                profile.clone(),
            );

            if *listen {
                runner.start_listener()?;
            } else {
                runner.start_standard()?;
            }
        }
    }

    Ok(())
}
