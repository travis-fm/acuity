use super::{Module, Sensor};
use std::fs::read_to_string;
use std::io;
use std::path::{Path, PathBuf};
use glob::glob;

pub struct HWMon {
    name: String,
    sensors: Vec<Sensor>,
    hwmon_path: PathBuf,
}

impl HWMon {
    pub fn new(hwmon_path: PathBuf) -> io::Result<Self> {
        let hwmon = HWMon {
            name: read_to_string(hwmon_path.join("name"))?
                .trim_ascii()
                .to_string(),
            sensors: HWMon::init_sensors(&hwmon_path)?,
            hwmon_path,
        };

        Ok(hwmon)
    }

    pub fn init_sensors(hwmon_path: &Path) -> io::Result<Vec<Sensor>> {
        let mut sensors: Vec<Sensor> = vec![];

        let string_parse_err = io::Error::other(format!("Could not parse string from path: {}", hwmon_path.display()));
        let glob_path = hwmon_path.to_str().ok_or(string_parse_err)?.to_owned() + "/*_input";

        match glob(&glob_path) {
            Ok(paths) => {
                for path in paths {
                    match path {
                        Ok(file) => {
                            let sensor_exists = sensors.iter().any(|s| s.input_file_path == file);

                            if !sensor_exists {
                                sensors.push(Sensor::new(file));
                            }
                        }
                        Err(e) => {
                            io::Error::other(e);
                        }
                    }
                }
            }
            Err(e) => {
                io::Error::other(e);
            }
        }

        Ok(sensors)
    }
}

impl Module for HWMon {
    fn poll_sensors(&mut self) {
        for sensor in &mut self.sensors {
            sensor.value = read_to_string(&sensor.input_file_path)
                .unwrap_or_default()
                .trim_ascii()
                .parse::<i32>()
                .unwrap_or_default();
        }
    }

    fn init() -> Vec<Self> {
        let mut modules: Vec<Self> = vec![];

        match glob("/sys/class/hwmon/hwmon*") {
            Ok(paths) => {
                for path in paths.flatten() {
                    let hwmon = HWMon {
                        name: read_to_string(path.join("name"))
                            .unwrap_or_default()
                            .trim_ascii()
                            .to_string(),
                        sensors: HWMon::init_sensors(&path).unwrap_or_default(),
                        hwmon_path: path,
                    };  
                    modules.push(hwmon);
                }
            }
            Err(..) => {
                println!("Unable to read glob pattern");
            }
        }

        modules
    }

    fn get_name(&self) -> String {
        self.name.clone()
    }

    fn set_name(&mut self, name: String) {
        self.name = name;
    }

    fn get_sensors(&self) -> Vec<Sensor> {
        todo!();
    }
}