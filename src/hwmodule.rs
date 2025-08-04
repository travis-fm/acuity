pub mod hwmon;

use async_trait::async_trait;
use ratatui::{
    Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::border,
    text::Line,
    widgets::{Block, StatefulWidget, Widget},
};

use crate::{sensor::Sensor, view_state::ViewState};

struct HWModuleWidget;

pub struct HWModule {
    module: Box<dyn Module + Send>,
    view_state: ViewState,
}

#[async_trait]
pub trait Module {
    async fn init() -> Vec<Self>
    where
        Self: Sized;
    fn name(&self) -> &str;
    fn set_name(&mut self, name: String);
    fn sensors(&self) -> Vec<&Sensor>;
    async fn refresh_sensors(&mut self);
}

impl HWModule {
    #[must_use]
    pub async fn init<T: Module + Send + 'static>() -> Vec<Self> {
        let modules = T::init().await;
        let mut hwmodules: Vec<HWModule> = vec![];

        for module in modules {
            let hwmodule = HWModule {
                module: Box::new(module),
                view_state: ViewState::new(),
            };

            hwmodules.push(hwmodule);
        }

        hwmodules
    }

    pub async fn refresh_sensors(&mut self) {
        self.module.refresh_sensors().await;
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.module.name()
    }

    #[must_use]
    pub fn sensors(&self) -> Vec<&Sensor> {
        self.module.sensors()
    }

    pub fn render(&mut self, frame: &mut Frame) {
        frame.render_stateful_widget(HWModuleWidget, self.view_state.area(), self);
    }

    pub fn view_state(&mut self) -> &mut ViewState {
        &mut self.view_state
    }
}

impl StatefulWidget for HWModuleWidget {
    type State = HWModule;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let module_name = Line::from(state.name());
        let module_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(module_name.centered());
        let mut constraints = vec![];
        state
            .module
            .sensors()
            .iter()
            .for_each(|_| constraints.push(Constraint::Fill(1)));

        let layout = Layout::vertical(constraints).split(module_block.inner(area));

        module_block.render(area, buf);
        for i in 0..state.module.sensors().len() {
            state.sensors()[i].render(layout[i], buf);
        }
    }
}
