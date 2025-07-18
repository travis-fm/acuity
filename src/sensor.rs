use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, Widget},
};
use uuid::Uuid;

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
    pub name: String,
    id: Uuid,
    pub sensor_type: SensorType,
    pub value: i32,
}

impl Sensor {
    #[must_use]
    pub fn new(name: String, sensor_type: SensorType, value: i32) -> Self {
        Sensor {
            name,
            id: Uuid::new_v4(),
            sensor_type,
            value,
        }
    }

    #[must_use]
    pub fn id(&self) -> String {
        self.id.to_string()
    }
}

impl Widget for &Sensor {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [name_area, value_area] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(area);

        let render_name = Line::from(self.name.as_str());
        let render_curr_value = Line::from(self.value.to_string());

        Paragraph::new(render_name)
            .alignment(Alignment::Left)
            .render(name_area, buf);
        Paragraph::new(render_curr_value)
            .alignment(Alignment::Right)
            .render(value_area, buf);
    }
}
