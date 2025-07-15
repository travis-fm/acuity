pub mod app;
pub mod sensor;
pub mod hwmodule;

use glob::glob;
use ratatui::layout::{Alignment, Constraint, Direction, Flex, Layout};
use std::fs::read_to_string;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal,
    Frame
};

use crate::app::App;
use crate::hwmodule::hwmon::HWMonitor;

fn main() -> io::Result<()> {
    
    App::run()
}
