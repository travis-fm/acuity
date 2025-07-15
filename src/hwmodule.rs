use std::io;

use ratatui::{buffer::Buffer, layout::Rect, widgets::Widget};

use crate::sensor::Sensor;

pub mod hwmon;

pub trait HWModule: Widget {

}