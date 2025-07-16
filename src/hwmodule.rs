pub mod hwmonitor;

use crate::sensor::Sensor;

pub trait HWModule {
    fn update_sensors(&mut self);
}