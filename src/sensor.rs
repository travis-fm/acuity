use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Paragraph, StatefulWidget, Widget},
};
use uuid::Uuid;

use crate::view_state::ViewState;

#[derive(PartialEq)]
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

struct SensorWidget;
pub struct Sensor {
    view_state: ViewState,
    pub name: String,
    id: Uuid,
    pub sensor_type: SensorType,
    pub value: i32,
}

impl Sensor {
    #[must_use]
    pub fn new(name: String, sensor_type: SensorType, value: i32) -> Self {
        let view_state = ViewState::new();
        let id = Uuid::new_v4();
        Sensor {
            view_state,
            name,
            id,
            sensor_type,
            value,
        }
    }

    #[must_use]
    pub fn id(&self) -> String {
        self.id.to_string()
    }

    pub fn view_state(&mut self) -> &mut ViewState {
        &mut self.view_state
    }

    pub fn render(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(SensorWidget, self.view_state.area(), self);
    }
}

impl StatefulWidget for SensorWidget {
    type State = Sensor;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [name_area, value_area] =
            Layout::horizontal([Constraint::Percentage(25), Constraint::Percentage(75)])
                .areas(area);

        let render_name = Line::from(state.name.as_str());
        let render_curr_value = Line::from(state.value.to_string());

        Paragraph::new(render_name)
            .alignment(Alignment::Left)
            .render(name_area, buf);
        Paragraph::new(render_curr_value)
            .alignment(Alignment::Right)
            .render(value_area, buf);
    }
}
