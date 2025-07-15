pub mod app;
pub mod sensor;
pub mod hwmodule;

use std::io;

use crate::app::App;

fn main() -> io::Result<()> {

    App::run()
}
