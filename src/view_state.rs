use ratatui::layout::Rect;

pub struct ViewState {
    area: Rect,
}

impl ViewState {
    pub fn new() -> Self {
        Self {
            area: Rect::new(0, 0, 0, 0),
        }
    }

    pub fn with_area(mut self, area: Rect) -> Self {
        self.area = area;

        self
    }

    pub fn area(&self) -> Rect {
        self.area
    }

    pub fn set_area(&mut self, area: Rect) {
        self.area = area;
    }
}
