use std::io;
use std::time::{Duration, Instant};

use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};

use crate::hwmodule::HWModule;
use crate::hwmodule::hwmon::HWMon;

pub struct App {
    exit: bool,
    modules: Vec<HWModule>,
    sensor_refresh_interval: Duration,
    last_sensor_refresh: Instant,
}

pub enum AppOptions {
    SensorRefreshInterval(Duration),
}

impl App {
    #[must_use]
    pub fn new(options: Option<Vec<AppOptions>>) -> Self {
        let mut app = App {
            exit: false,
            modules: vec![],
            sensor_refresh_interval: Duration::from_millis(1000),
            last_sensor_refresh: Instant::now(),
        };

        app.load_options(options);

        app
    }

    pub fn run(&mut self) -> io::Result<()> {
        let mut terminal = ratatui::init();
        self.init_modules();

        while !self.exit {
            self.update_modules();
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }

        ratatui::restore();

        Ok(())
    }

    fn load_options(&mut self, options: Option<Vec<AppOptions>>) {
        if let Some(options) = options {
            for option in options {
                // Expecting to add more app options later on. Leaving default at bottom for future implementations.
                #[allow(clippy::single_match)]
                #[allow(unreachable_patterns)]
                match option {
                    AppOptions::SensorRefreshInterval(interval) => {
                        self.sensor_refresh_interval = interval;
                    }
                    _ => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn init_modules(&mut self) {
        for module in HWModule::init::<HWMon>() {
            self.modules.push(module);
        }
    }

    fn update_modules(&mut self) {
        if Instant::now() >= self.last_sensor_refresh + self.sensor_refresh_interval {
            for module in &mut self.modules {
                module.poll_sensors();
            }

            self.last_sensor_refresh = Instant::now();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        // Expecting to add more keybindings later on. Leaving default at bottom for future implementations.
        #[allow(clippy::single_match)]
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(Duration::from_secs(0))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                }

                _ => {}
            }
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

        let [main_area] = Layout::vertical([Constraint::Fill(1)]).areas(app_block.inner(area));

        // This is temporary while prototyping. Should smartly generate module cells later when more are added.
        // Ignore cast truncation for now.
        let module_col_size = 100
            / if self.modules.is_empty() {
                1
            } else {
                self.modules.len()
            };
        #[allow(clippy::cast_possible_truncation)]
        let module_cols =
            (0..self.modules.len()).map(|_| Constraint::Percentage(module_col_size as u16));

        let module_layout = Layout::horizontal(module_cols).spacing(1).split(main_area);
        app_block.render(area, buf);

        for i in 0..self.modules.len() {
            self.modules[i].render(module_layout[i], buf);
        }
    }
}
