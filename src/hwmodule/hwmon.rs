use crate::sensor::SensorType;

use super::{Module, Sensor};
use async_trait::async_trait;
use glob::glob;
use std::fs::read_to_string;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    async fn init_sensors(&mut self) -> io::Result<()> {
        let string_parse_err = io::Error::other(format!(
            "Could not parse string from path: {}",
            self.hwmon_path.display()
        ));
        let glob_path = self.hwmon_path.to_str().ok_or(string_parse_err)?.to_owned() + "/*_input";

        let sensors = Arc::new(Mutex::new(vec![]));
        let mut tasks = vec![];

        match glob(&glob_path) {
            Ok(paths) => {
                for path in paths.flatten() {
                    let sensors = sensors.clone();

                    tasks.push(tokio::spawn(async move {
                        let file_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or_default();
                        let sensor_type = Self::parse_sensor_type(file_name);
                        let value = Self::read_sensor(&path).unwrap_or_default();

                        sensors.lock().await.push(HWMonSensor {
                            sensor: Sensor::new(file_name.to_string(), sensor_type, value),
                            file_path: path,
                        });
                    }));
                }
            }
            Err(e) => {
                io::Error::other(e);
            }
        }

        for task in tasks {
            task.await.unwrap_or_default();
        }

        self.hwmon_sensors = Arc::into_inner(sensors).unwrap().into_inner();

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

#[async_trait]
impl Module for HWMon {
    async fn refresh_sensors(&mut self) {
        let mut tasks = vec![];

        for sensor in &self.hwmon_sensors {
            let file_path = sensor.file_path.clone();

            tasks.push(tokio::spawn(async move {
                Self::read_sensor(&file_path).unwrap_or_default()
            }));
        }

        for (i, task) in tasks.into_iter().enumerate() {
            self.hwmon_sensors[i].sensor.value = task.await.unwrap_or_default();
        }
    }

    async fn init() -> Vec<Self> {
        let modules = Arc::new(Mutex::new(vec![]));
        let mut tasks = vec![];

        match glob("/sys/class/hwmon/hwmon*") {
            Ok(paths) => {
                for path in paths.flatten() {
                    let modules = modules.clone();
                    tasks.push(tokio::spawn(async move {
                        let mut hwmon = HWMon {
                            name: read_to_string(path.join("name"))
                                .unwrap_or_default()
                                .trim_ascii()
                                .to_string(),
                            hwmon_path: path,
                            hwmon_sensors: vec![],
                        };

                        hwmon.init_sensors().await.unwrap_or_default();
                        modules.lock().await.push(hwmon);
                    }));
                }
            }
            Err(..) => {
                println!("Unable to read glob pattern");
            }
        }

        for task in tasks {
            task.await.unwrap_or_default();
        }

        Arc::into_inner(modules).unwrap().into_inner()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn sensors(&mut self) -> Vec<&mut Sensor> {
        self.hwmon_sensors
            .iter_mut()
            .map(|s| &mut s.sensor)
            .collect::<Vec<_>>()
    }
}
