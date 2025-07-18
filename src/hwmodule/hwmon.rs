use crate::sensor::SensorType;

use super::{Module, Sensor};
use glob::glob;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;

struct HWMonSensor {
    sensor: Sensor,
    file_path: PathBuf,
}

pub struct HWMon {
    name: String,
    hwmon_path: PathBuf,
    hwmon_sensors: Vec<HWMonSensor>,
}

impl HWMon {
    fn parse_sensor_type(file_name: &str) -> SensorType {
        match file_name.split(char::is_numeric).next() {
            Some(name) => match name {
                "chip" => SensorType::Chip,
                "temp" => SensorType::Temperature,
                "in" => SensorType::Voltage,
                "curr" => SensorType::Current,
                "power" => SensorType::Power,
                "energy" => SensorType::Energy,
                "humidity" => SensorType::Humidity,
                "fan" => SensorType::Fan,
                _ => SensorType::Unknown,
            },
            None => SensorType::Unknown,
        }
    }

    fn init_sensors(&mut self) -> io::Result<()> {
        let string_parse_err = io::Error::other(format!(
            "Could not parse string from path: {}",
            self.hwmon_path.display()
        ));
        let glob_path = self.hwmon_path.to_str().ok_or(string_parse_err)?.to_owned() + "/*_input";

        self.hwmon_sensors = vec![];

        match glob(&glob_path) {
            Ok(paths) => {
                for path in paths.flatten() {
                    let file_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or_default();
                    let sensor_exists = self.hwmon_sensors.iter().any(|s| s.file_path == path);
                    let sensor_type = Self::parse_sensor_type(file_name);
                    let value = Self::read_sensor(&path)?;

                    if !sensor_exists {
                        self.hwmon_sensors.push(HWMonSensor {
                            sensor: Sensor::new(file_name.to_string(), sensor_type, value),
                            file_path: path,
                        });
                    }
                }
            }
            Err(e) => {
                io::Error::other(e);
            }
        }

        Ok(())
    }

    fn read_sensor(path: &PathBuf) -> io::Result<i32> {
        let value = read_to_string(path)?
            .trim_ascii()
            .parse::<i32>()
            .unwrap_or_default();
        Ok(value)
    }
}

impl Module for HWMon {
    fn poll_sensors(&mut self) {
        for sensor in &mut self.hwmon_sensors {
            sensor.sensor.value = Self::read_sensor(&sensor.file_path).unwrap_or_default();
        }
    }

    fn init() -> Vec<Self> {
        let mut modules: Vec<Self> = vec![];

        match glob("/sys/class/hwmon/hwmon*") {
            Ok(paths) => {
                for path in paths.flatten() {
                    let mut hwmon = HWMon {
                        name: read_to_string(path.join("name"))
                            .unwrap_or_default()
                            .trim_ascii()
                            .to_string(),
                        hwmon_path: path,
                        hwmon_sensors: vec![],
                    };
                    hwmon.init_sensors().unwrap_or_default();
                    modules.push(hwmon);
                }
            }
            Err(..) => {
                println!("Unable to read glob pattern");
            }
        }

        modules
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn sensors(&self) -> Vec<&Sensor> {
        self.hwmon_sensors.iter().map(|s| &s.sensor).collect()
    }
}
