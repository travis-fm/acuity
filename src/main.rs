pub mod app;
pub mod event_stream;
pub mod hwmodule;
pub mod sensor;
pub mod view_state;

use color_eyre::Result;

use crate::app::App;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new(None);
    ratatui::run(|t| app.run(t));

    Ok(())
}
