pub mod app;
pub mod event_stream;
pub mod hwmodule;
pub mod sensor;

use color_eyre::Result;

use crate::app::App;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new(None);
    app.run().await?;

    Ok(())
}
