use eframe::{
    egui::{Response, Sense, Ui},
    epaint::{Rect, Vec2},
};

pub struct ZoomArea {
    zoom: f32,
}

impl Default for ZoomArea {
    fn default() -> Self {
        Self { zoom: 2.0 }
    }
}

impl ZoomArea {
    pub fn create(&mut self, ui: &mut Ui, widget_size: Vec2, senses: Sense) -> (Rect, Response) {
        let (widget_rect, widget_response) =
            ui.allocate_exact_size(widget_size * self.zoom, senses);

        if widget_response.hovered() {
            ui.input(|i| for event in &i.events {
                match event {
                    eframe::egui::Event::Zoom(value) => self.zoom *= value,
                    _ => {}
                }
            });
        }

        (widget_rect, widget_response)
    }
}
