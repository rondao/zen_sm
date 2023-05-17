use eframe::{
    egui::{CursorIcon, Key, Response, Sense, Ui},
    epaint::{Rect, Vec2},
};

use super::zoom_area::ZoomArea;

#[derive(Default)]
pub struct DragArea {
    zoom_area: ZoomArea,
}

impl DragArea {
    pub fn create(&mut self, ui: &mut Ui, widget_size: Vec2) -> (Rect, Response) {
        let paning = ui.input(|i| i.key_down(Key::Space));

        let senses = if paning {
            Sense::focusable_noninteractive()
        } else {
            Sense::click_and_drag()
        };

        let (rect, response) = self.zoom_area.create(ui, widget_size, senses);

        let response = if paning {
            response.on_hover_cursor(CursorIcon::Grab)
        } else {
            response
        };

        (rect, response)
    }
}
