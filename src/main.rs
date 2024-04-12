pub mod cli;
pub mod config;

use std::{
    path::Path,
    time::{Duration, Instant},
};

use clap::Parser;
use cli::Args;
use notify::{INotifyWatcher, RecursiveMode, Watcher};

fn create_file_listener(
    args: Args,
    profile: String,
    build: String,
) -> notify::Result<INotifyWatcher> {
    let config = notify::Config::default().with_poll_interval(Duration::from_secs(2));

    notify::Watcher::new(
        move |res| match res {
            Ok(_) => {
                println!("File changes; recompiling...");
                let current_time = Instant::now();

                let config = args.parse_config().expect("Failed to parse config.");

                match config.build_packs(&args, &profile, &build) {
                    Ok(_) => {
                        let time_passed = current_time.elapsed();
                        println!("Completed in {:.2} seconds.", time_passed.as_secs_f32());
                    }
                    Err(e) => {
                        println!("Build error: {:?}", e);
                    }
                }
            }
            Err(e) => println!("File listener error: {:?}", e),
        },
        config,
    )
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match &args.commands {
        cli::Subcommands::Build {
            profile,
            build,
            listen,
        } => {
            let config = args.parse_config()?;
            config.build_packs(&args, &profile, &build)?;

            if *listen {
                let mut watcher =
                    create_file_listener(args.clone(), profile.to_string(), build.to_string())?;
                watcher.watch(Path::new("src"), RecursiveMode::Recursive)?;
                loop {}
            }
        }
    }

    Ok(())
}
