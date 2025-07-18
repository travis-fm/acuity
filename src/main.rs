pub mod app;
pub mod hwmodule;
pub mod sensor;

use std::io;

use crate::app::App;

fn main() -> io::Result<()> {
    let mut app = App::new(None);
    app.run()?;

    Ok(())
}
