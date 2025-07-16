use ratatui::{widgets::Widget};
use crate::sensor::Sensor;

pub mod hwmonitor;

pub trait HWModule: Widget {
    fn update_sensors(&mut self);
}