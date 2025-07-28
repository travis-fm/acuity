pub mod app;
pub mod hwmodule;
pub mod sensor;

use std::io;

use crate::app::App;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut app = App::new(None);
    app.run().await?;

    Ok(())
}
