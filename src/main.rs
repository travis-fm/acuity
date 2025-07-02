use glob::glob;
use std::fs::{read_to_string};
use std::io::{self};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

#[derive(PartialEq, Debug)]
enum SensorType {
    Chip,
    Temperature,
    Voltage,
    Current,
    Power,
    Energy,
    Humidity,
    Fan,
    Unknown,
}

#[derive(Debug)]
struct Sensor {
    display_name: String,
    file_name: String,
    input_file_path: PathBuf,
    sensor_type: SensorType,
    value: i32,
}

#[derive(Debug)]
struct HwMon {
    display_name: String,
    sensors: Vec<Sensor>,
    hwmon_path: PathBuf,
}

impl Sensor {
    fn new(value_path: PathBuf) -> Self {
        let file_name = value_path
            .file_name().unwrap_or_default().to_str().unwrap_or_default().to_owned();
        let display_name = file_name
            .split("_").next().unwrap_or_default().to_owned();

        Sensor {
            sensor_type: Sensor::parse_type_from_file(&display_name),
            file_name,
            display_name,
            value: 0,
            input_file_path: value_path
        }
    }

    fn parse_type_from_file(file_name: &str) -> SensorType {
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
}

impl HwMon {
    fn new(hwmon_path: PathBuf) -> io::Result<Self> {
        let hwmon = HwMon {
            display_name: read_to_string(hwmon_path.join("name"))?.trim_ascii().to_string(),
            sensors: HwMon::init_sensors(&hwmon_path)?,
            hwmon_path,
        };

        Ok(hwmon)
    }

    fn init_sensors(hwmon_path: &Path) -> io::Result<Vec<Sensor>> {
        let mut sensors: Vec<Sensor> = vec![];

        let string_parse_err =
            io::Error::other(format!("Could not parse string from path: {:#?}", hwmon_path));
        let glob_path = hwmon_path.to_str().ok_or(string_parse_err)?.to_owned() + "/*_input";

        match glob(&glob_path) {
            Ok(paths) => {
                for path in paths {
                    match path {
                        Ok(file) => {
                            let sensor_exists= sensors.iter().any(|s| s.input_file_path == file);

                            if sensor_exists {
                                continue;
                            } else {
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

    fn update_sensors(&mut self) {
        for sensor in &mut self.sensors {
            sensor.value = read_to_string(&sensor.input_file_path).unwrap_or_default().trim_ascii().parse::<i32>().unwrap_or_default();
        }
    }
}

fn main() {
    let mut modules: Vec<HwMon> = vec![];

    match glob("/sys/class/hwmon/hwmon*") {
        Ok(paths) => {
            for path in paths.flatten() {
                if let Ok(module) = HwMon::new(path) {
                    modules.push(module);
                }
            }
        },
        Err(..) => {
            println!("Unable to read glob pattern");
        }
    }

    loop {
        for module in &mut modules {
            module.update_sensors();
            println!("{:#?}", module);
        }

        sleep(Duration::from_secs(5));
    }
}
