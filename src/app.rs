use std::io;
use std::option::Option;
use std::time::{Duration, Instant};

use color_eyre::Result;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent};
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Widget};
use ratatui::{DefaultTerminal, Frame};

use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::event_stream::{Event, EventStream};
use crate::hwmodule::HWModule;
use crate::hwmodule::hwmon::HWMon;

pub enum Action {
    Quit,
    Render,
    RefreshSensors,
}

pub struct App {
    exit: bool,
    modules: Vec<HWModule>,
    sensor_refresh_interval: Duration,
    action_tx: UnboundedSender<Action>,
    action_rx: UnboundedReceiver<Action>,
}

pub enum AppOptions {
    SensorRefreshInterval(Duration),
}

impl App {
    #[must_use]
    pub fn new(options: Option<Vec<AppOptions>>) -> Self {
        let exit = false;
        let modules = vec![];
        let sensor_refresh_interval = Duration::from_millis(1000);
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        let mut app = App {
            exit,
            modules,
            sensor_refresh_interval,
            action_tx,
            action_rx,
        };

        app.load_options(options);

        app
    }

    #[tokio::main]
    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        //let mut terminal = ratatui::init();
        let mut event_stream = EventStream::new();

        self.init().await?;
        terminal.draw(|f| self.draw(f))?;

        while !self.exit {
            if let Some(e) = event_stream.next().await {
                self.handle_event(&e)
                    .map(|action| self.action_tx.send(action));
            }
            while let Ok(action) = self.action_rx.try_recv() {
                self.handle_action(action, terminal).await?;
                // if matches!(action, Action::Resize(_, _) | Action::Render) {
                //     tui.draw(|frame| self.render(frame))?;
                // }
            }
        }

        ratatui::restore();

        Ok(())
    }

    async fn init(&mut self) -> io::Result<()> {
        self.init_modules().await;

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

    async fn init_modules(&mut self) {
        let modules = HWModule::init::<HWMon>().await;

        for module in modules {
            self.modules.push(module);
        }
    }

    async fn refresh_sensors(&mut self) {
        for module in &mut self.modules {
            module.refresh_sensors().await;
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn push_action(&mut self, action: Action) {
        self.action_tx.send(action);
    }

    fn handle_event(&mut self, event: &Event) -> Option<Action> {
        match event {
            Event::Crossterm(CrosstermEvent::Key(key_event))
                if key_event.kind == KeyEventKind::Press =>
            {
                self.handle_key_event(key_event)
            }
            Event::Crossterm(CrosstermEvent::Mouse(mouse_event)) => {
                self.handle_mouse_event(mouse_event)
            }
            Event::SensorRefresh => Some(Action::RefreshSensors),
            Event::Crossterm(CrosstermEvent::FocusGained | CrosstermEvent::Resize(_, _)) => {
                Some(Action::Render)
            }
            _ => None,
        }
    }

    async fn handle_action(
        &mut self,
        action: Action,
        terminal: &mut DefaultTerminal,
    ) -> Result<()> {
        match action {
            Action::Quit => self.exit(),
            Action::RefreshSensors => {
                self.refresh_sensors().await;
                self.push_action(Action::Render);
            }
            Action::Render => {
                terminal.draw(|f| f.render_widget(&*self, f.area()))?;
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_key_event(&self, key_event: &KeyEvent) -> Option<Action> {
        // Expecting to add more keybindings later on. Leaving default at bottom for future implementations.
        #[allow(clippy::single_match)]
        match key_event.code {
            KeyCode::Char('q') => Some(Action::Quit),
            _ => None,
        }
    }

    fn handle_mouse_event(&self, mouse_event: &MouseEvent) -> Option<Action> {
        None
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
