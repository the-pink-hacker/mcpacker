pub mod asset;
pub mod cli;
pub mod compile;
pub mod config;
pub mod minecraft;
pub mod runner;
pub mod sanitize;

use clap::Parser;
use cli::Args;
use runner::Runner;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match &args.commands {
        cli::Subcommands::Build {
            profile,
            builds,
            listen: _,
        } => {
            let runner = Runner::new(
                args.config,
                args.minecraft,
                builds.to_vec(),
                profile.clone(),
            )?;

            runner.start_standard().await?;
        }
    }

    Ok(())
}
