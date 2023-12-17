use anyhow::Result as AnyResult;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::LevelFilter;

use crate::app::App;

mod app;
mod colors;
mod mainmenu;
mod mods;
mod mods_views;

fn main() -> AnyResult<()> {
    simple_logging::log_to_file("logs.log", LevelFilter::Debug)?;

    startup()?;

    let status = App::new().run();

    shutdown()?;
    status?;

    Ok(())
}

pub fn startup() -> AnyResult<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

pub fn shutdown() -> AnyResult<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
