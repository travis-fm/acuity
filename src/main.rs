pub mod app;
pub mod sensor;
pub mod hwmodule;

use std::io;

use crate::app::App;

fn main() -> io::Result<()> {
    let mut app = App::new(None);
    app.run()?;
    
    Ok(())
}
