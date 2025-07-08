use glob::glob;
use ratatui::layout::{Constraint, Direction, Flex, Layout};
use std::fs::read_to_string;
use std::io::{self};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Stylize,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget},
    DefaultTerminal,
    Frame
};

#[derive(PartialEq, Debug)]
enum SensorType {
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
struct Sensor {
    display_name: String,
    file_name: String,
    input_file_path: PathBuf,
    sensor_type: SensorType,
    value: i32,
}

#[derive(Debug)]
struct HwMon {
    display_name: String,
    sensors: Vec<Sensor>,
    hwmon_path: PathBuf,
}

impl Sensor {
    fn new(value_path: PathBuf) -> Self {
        let file_name = value_path
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            .to_owned();
        let display_name = file_name.split("_").next().unwrap_or_default().to_owned();

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
        let row = Layout::default().direction(Direction::Horizontal).constraints(vec![
            Constraint::Percentage(25),
            Constraint::Percentage(75),
        ]).split(area);

        let render_name = Line::from(self.display_name.as_str());
        let render_curr_value = Line::from(self.value.to_string());

        Paragraph::new(render_name)
            .render(row[0], buf);
        Paragraph::new(render_curr_value)
            .render(row[1], buf);
    }
}

impl HwMon {
    fn new(hwmon_path: PathBuf) -> io::Result<Self> {
        let hwmon = HwMon {
            display_name: read_to_string(hwmon_path.join("name"))?
                .trim_ascii()
                .to_string(),
            sensors: HwMon::init_sensors(&hwmon_path)?,
            hwmon_path,
        };

        Ok(hwmon)
    }

    fn init_sensors(hwmon_path: &Path) -> io::Result<Vec<Sensor>> {
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

    fn update_sensors(&mut self) {
        for sensor in &mut self.sensors {
            sensor.value = read_to_string(&sensor.input_file_path)
                .unwrap_or_default()
                .trim_ascii()
                .parse::<i32>()
                .unwrap_or_default();
        }
    }
}
impl Widget for &HwMon {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let module_box = Block::bordered()
            .border_set(border::PLAIN);
        let mut constraints = vec![Constraint::Max(16)];
        self.sensors.iter().for_each(|_| constraints.push(Constraint::Fill(1)));

        let layout = Layout::vertical(constraints).split(module_box.inner(area));

        Paragraph::new(Text::from(self.display_name.as_str()))
            .centered()
            .block(Block::bordered())
            .render(layout[0], buf);

        for i in 0..self.sensors.len() {
            self.sensors[i].render(layout[i+1], buf);
        }
    }
}

#[derive(Debug, Default)]
struct App {
    exit: bool,
    modules: Vec<HwMon>
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
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
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }

            _ => {}
        };
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

        let header_footer_size = 16;
        let main_area_size = app_block.inner(area).height - (header_footer_size * 2);
        let [header_area, main_area, footer_area] = Layout::vertical([
            Constraint::Max(header_footer_size),
            Constraint::Length(main_area_size),
            Constraint::Max(header_footer_size),
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

fn main() -> io::Result<()> {
    let mut app = App::default();
    
    match glob("/sys/class/hwmon/hwmon*") {
        Ok(paths) => {
            for path in paths.flatten() {
                if let Ok(module) = HwMon::new(path) {
                    app.modules.push(module);
                }
            }
        }
        Err(..) => {
            println!("Unable to read glob pattern");
        }
    }

    let mut terminal = ratatui::init();
    let app_result = app.run(&mut terminal);
    ratatui::restore();
    app_result

    /*
    loop {
        for module in &mut modules {
            module.update_sensors();
            println!("{:#?}", module);
        }

        sleep(Duration::from_secs(5));
    }
    */
}
