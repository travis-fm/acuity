pub mod hwmon;

use async_trait::async_trait;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::border,
    text::Line,
    widgets::{Block, Widget},
};

use crate::sensor::Sensor;

pub struct HWModule {
    module: Box<dyn Module + Send>,
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
}

impl Widget for &HWModule {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let module_name = Line::from(self.name());
        let module_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(module_name.centered());
        let mut constraints = vec![];
        self.module
            .sensors()
            .iter()
            .for_each(|_| constraints.push(Constraint::Fill(1)));

        let layout = Layout::vertical(constraints).split(module_block.inner(area));

        module_block.render(area, buf);
        for i in 0..self.module.sensors().len() {
            self.sensors()[i].render(layout[i], buf);
        }
    }
}
