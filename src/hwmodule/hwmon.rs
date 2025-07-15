use super::*;
use std::fs::read_to_string;
use std::io;
use std::path::{Path, PathBuf};
use glob::glob;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

#[derive(Debug)]
pub struct HWMonitor {
    display_name: String,
    sensors: Vec<Sensor>,
    hwmon_path: PathBuf,
}

impl HWMonitor {
    pub fn new(hwmon_path: PathBuf) -> io::Result<Self> {
        let hwmon = HWMonitor {
            display_name: read_to_string(hwmon_path.join("name"))?
                .trim_ascii()
                .to_string(),
            sensors: HWMonitor::init_sensors(&hwmon_path)?,
            hwmon_path,
        };

        Ok(hwmon)
    }

    pub fn init_sensors(hwmon_path: &Path) -> io::Result<Vec<Sensor>> {
        let mut sensors: Vec<Sensor> = vec![];

        let string_parse_err = io::Error::other(format!(
            "Could not parse string from path: {:#?}",
            hwmon_path
        ));
        let glob_path = hwmon_path.to_str().ok_or(string_parse_err)?.to_owned() + "/*_input";

        match glob(&glob_path) {
            Ok(paths) => {
                for path in paths {
                    match path {
                        Ok(file) => {
                            let sensor_exists = sensors.iter().any(|s| s.input_file_path == file);

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

    pub fn update_sensors(&mut self) {
        for sensor in &mut self.sensors {
            sensor.value = read_to_string(&sensor.input_file_path)
                .unwrap_or_default()
                .trim_ascii()
                .parse::<i32>()
                .unwrap_or_default();
        }
    }
}

impl HWModule for HWMonitor {

}

impl Widget for HWMonitor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let module_name = Line::from(self.display_name.as_str());
        let module_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(module_name.centered());
        let mut constraints = vec![];
        self.sensors.iter().for_each(|_| constraints.push(Constraint::Fill(1)));

        let layout = Layout::vertical(constraints).split(module_block.inner(area));

        module_block.render(area, buf);
        for i in 0..self.sensors.len() {
            self.sensors[i].render(layout[i], buf);
        }
    }
}