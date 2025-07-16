use crate::sensor::Sensor;

pub mod hwmonitor;

pub trait HWModule {
    fn update_sensors(&mut self);
}