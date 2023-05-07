use eframe::{
    egui::{Event, PointerButton, Response, Sense, Ui},
    epaint::{Rect, Vec2},
};

use super::zoom_area::ZoomArea;

#[derive(Default)]
pub struct DragArea {
    zoom_area: ZoomArea,
    dragging: bool,
}

impl DragArea {
    pub fn create(&mut self, ui: &mut Ui, widget_size: Vec2) -> (Rect, Response) {
        ui.input(|i| for event in &i.events {
            match event {
                Event::PointerButton {
                    pos: _,
                    button: PointerButton::Secondary,
                    pressed,
                    modifiers: _,
                } => self.dragging = *pressed,
                _ => {}
            }
        });

        self.zoom_area.create(
            ui,
            widget_size,
            if self.dragging {
                Sense::click_and_drag()
            } else {
                Sense::click()
            },
        )
    }
}
