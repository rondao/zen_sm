use std::borrow::Borrow;

use eframe::{
    egui::{Response, Ui},
    epaint::{Color32, Pos2, Rect, Stroke},
};

use super::selectable_area::{Selectable, SelectableArea};

pub struct PaintedSelectableArea {
    selectable: SelectableArea,
}

impl PaintedSelectableArea {
    pub fn new(area: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            selectable: SelectableArea::new(area, size),
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        widget_rect: Rect,
        widget_response: &Response,
    ) -> Option<Selectable> {
        let selection = self.selectable.ui(widget_rect, &widget_response);

        if let Some(selection) = selection.borrow() {
            match selection {
                Selectable::UnselectedHovering(rect) => {
                    self.paint_selection(ui, widget_rect, *rect)
                }
                Selectable::Dragging(rect) => self.paint_selection(ui, widget_rect, *rect),
                Selectable::SelectedHovering(_) => (),
                Selectable::Selected(_) => (),
                Selectable::Clicked(_) => (),
            }
        }

        selection
    }

    pub fn position(&self) -> Option<Pos2> {
        self.selectable.position()
    }

    pub fn unselect(&mut self) {
        self.selectable.unselect();
    }

    pub fn set_sizes(&mut self, area: [f32; 2], size: [f32; 2]) {
        self.selectable.set_sizes(area, size);
    }

    fn paint_selection(&self, ui: &mut Ui, widget_rect: Rect, selection: Rect) {
        ui.painter_at(widget_rect)
            .rect_stroke(selection, 1.0, Stroke::new(2.0, Color32::WHITE));
    }
}
