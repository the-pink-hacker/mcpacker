pub mod asset;
pub mod cli;
pub mod compile;
pub mod config;
pub mod minecraft;
pub mod runner;
pub mod sanitize;

use clap::Parser;
use cli::{Args, Subcommands};
use runner::Runner;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match &args.commands {
        Subcommands::Build {
            profile,
            builds,
            listen: _,
        } => {
            Runner::build(
                args.config,
                args.minecraft,
                builds.to_vec(),
                profile.clone(),
            )?
            .run()
            .await
        }
        Subcommands::Deploy {
            profile,
            builds,
            modrinth_api_token,
        } => {
            Runner::deploy(
                args.config,
                args.minecraft,
                builds.to_vec(),
                profile.clone(),
                &modrinth_api_token,
            )?
            .run()
            .await
        }
    }
}
