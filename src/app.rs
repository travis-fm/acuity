use std::io;
use std::time::{Duration, Instant};

use glob::glob;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};
use ratatui::{Frame};

use crate::hwmodule::hwmonitor::HWMonitor;

#[derive(Debug)]
pub struct App {
    exit: bool,
    modules: Vec<HWMonitor>,
    sensor_refresh_interval: Duration,
    last_sensor_refresh: Instant,
}

impl App {
    pub fn run() -> io::Result<()> {
        let mut app = App {
            exit: false,
            modules: vec![],
            sensor_refresh_interval: Duration::from_millis(1000),
            last_sensor_refresh: Instant::now()
        };

        app.load_modules();

        let mut terminal = ratatui::init();
        while !app.exit {
            app.update_modules();
            terminal.draw(|f| f.render_widget(&app, f.area()))?;
            app.handle_events()?;
        }

        ratatui::restore();

        Ok(())
    }

    fn load_modules(&mut self) {
        match glob("/sys/class/hwmon/hwmon*") {
            Ok(paths) => {
                for path in paths.flatten() {
                    if let Ok(module) = HWMonitor::new(path) {
                        self.modules.push(module);
                    }
                }
            }
            Err(..) => {
                println!("Unable to read glob pattern");
            }
        }
    }

    fn update_modules(&mut self) {
        if Instant::now() >= self.last_sensor_refresh + self.sensor_refresh_interval {
            for module in &mut self.modules {
                module.update_sensors();
            }

            self.last_sensor_refresh = Instant::now();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_secs(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }

                _ => {}
            };
        }
        
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let app_title = Line::from("Acumen Hardware Monitor");
        let app_version = Line::from("v0.0.1-dev");
        let app_block = Block::bordered()
            .title(app_title.centered())
            .title_bottom(app_version.right_aligned())
            .border_set(border::THICK);

        /*
        let header_footer_size = 16;
        let main_area_size = app_block.inner(area).height - (header_footer_size * 2);
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Max(header_footer_size),
            Constraint::Length(main_area_size),
            Constraint::Max(header_footer_size),
        ]).areas(app_block.inner(area));
        */
        let [main_area] = Layout::vertical([
            Constraint::Fill(1)
        ]).areas(app_block.inner(area));

        let module_col_size = 100 / if self.modules.len() > 0 { self.modules.len() } else { 1 };
        let module_cols = (0..self.modules.len())
            .map(|_| Constraint::Percentage(module_col_size as u16));
        let module_layout = Layout::horizontal(module_cols).spacing(1).split(main_area);

        app_block.render(area, buf);

        for i in 0..self.modules.len() {
            self.modules[i].render(module_layout[i], buf);
        }
    }
}