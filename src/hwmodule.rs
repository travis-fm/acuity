pub mod hwmon;

use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    symbols::border, 
    text::Line, 
    widgets::{Block, Widget}
};

use crate::sensor::Sensor;

pub struct HWModule {
    module: Box<dyn Module>, 
    pub name: String,
    pub sensors: Vec<Sensor>,
}

pub trait Module {
    fn init() -> Vec<Self> where Self: Sized;
    fn get_name(&self) -> String;
    fn set_name(&mut self, name: String);
    fn get_sensors(&self) -> Vec<Sensor>;
    fn poll_sensors(&mut self);
}

impl HWModule {
    pub fn init<T: Module + 'static>() -> Vec<Self> { 
        let modules = T::init();
        let mut hwmodules: Vec<HWModule> = vec![];

        for module in modules {
            let name = module.get_name();
            let sensors = module.get_sensors();
            let hwmodule = HWModule { 
                module: Box::new(module),
                name,
                sensors,
            };

            hwmodules.push(hwmodule);
        }

        hwmodules
    }

    pub fn poll_sensors(&mut self) {
        self.module.poll_sensors();
        self.sensors = self.module.get_sensors();
    }
}

impl Widget for &HWModule {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let module_name = Line::from(self.name.as_str());
        let module_block = Block::bordered()
            .border_set(border::PLAIN)
            .title(module_name.centered());
        let mut constraints = vec![];
        self.module.get_sensors().iter().for_each(|_| constraints.push(Constraint::Fill(1)));

        let layout = Layout::vertical(constraints).split(module_block.inner(area));

        module_block.render(area, buf);
        for i in 0..self.module.get_sensors().len() {
            self.sensors[i].render(layout[i], buf);
        }
    }
}