use std::path::PathBuf;

use ratatui::{buffer::Buffer, layout::{Alignment, Constraint, Layout, Rect}, text::Line, widgets::{Paragraph, Widget}};

#[derive(PartialEq, Debug)]
pub enum SensorType {
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
pub struct Sensor {
    pub display_name: String,
    pub file_name: String,
    pub input_file_path: PathBuf,
    pub sensor_type: SensorType,
    pub value: i32,
}

impl Sensor {
    pub fn new(value_path: PathBuf) -> Self {
        let file_name = value_path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned();
        let display_name = file_name.split('_').next().unwrap_or_default().to_owned();

        Sensor {
            sensor_type: Sensor::parse_type_from_file(&display_name),
            file_name,
            display_name,
            value: 0,
            input_file_path: value_path,
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

impl Widget for &Sensor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [name_area, value_area] = Layout::horizontal([
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ]).areas(area);

        let render_name = Line::from(self.display_name.as_str());
        let render_curr_value = Line::from(self.value.to_string());

        Paragraph::new(render_name)
            .alignment(Alignment::Left)
            .render(name_area, buf);
        Paragraph::new(render_curr_value)
            .alignment(Alignment::Right)
            .render(value_area, buf);
    }
}