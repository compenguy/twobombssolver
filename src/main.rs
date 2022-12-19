use clap::Arg;
use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use flexi_logger::Logger;
use log::debug;
use tempdir::TempDir;

mod errors;
use anyhow::Result;

mod coord;
mod terminal;
mod twobombs;

fn main() -> Result<()> {
    let td = TempDir::new(crate_name!())?;
    println!("Logging to {}", td.path().display());
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("debug")
                .short("g")
                .multiple(true)
                .help("Write extended debug log information to a temp file."),
        )
        .get_matches();

    // Initialize logging
    let log_level = match matches.occurrences_of("debug") {
        0 => log::LevelFilter::Off,
        1 => log::LevelFilter::Error,
        2 => log::LevelFilter::Warn,
        3 => log::LevelFilter::Info,
        4 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    let mut log_builder =
        Logger::try_with_env_or_str(format!("error, {} = {}", crate_name!(), log_level))?;
    log_builder = log_builder
        .log_to_file(
            flexi_logger::FileSpec::default()
            .directory(td.path()));

    log_builder.start()?;

    debug!("Starting game...");
    terminal::play_game()?;
    Ok(())
}
