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
use tokio::sync::OnceCell;

static RUNNER: OnceCell<Runner> = OnceCell::const_new();

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match &args.commands {
        Subcommands::Build {
            profile,
            builds,
            listen,
        } => {
            let runner = RUNNER
                .get_or_try_init(|| async {
                    Runner::build(
                        args.config,
                        args.minecraft,
                        builds.to_vec(),
                        profile.clone(),
                    )
                })
                .await?;

            if *listen {
                runner.spawn_run_listener().await
            } else {
                runner.run().await
            }
        }
        Subcommands::Deploy {
            profile,
            builds,
            modrinth_api_token,
        } => {
            RUNNER
                .get_or_try_init(|| async {
                    Runner::deploy(
                        args.config,
                        args.minecraft,
                        builds.to_vec(),
                        profile.clone(),
                        &modrinth_api_token,
                    )
                })
                .await?
                .run()
                .await
        }
    }
}
