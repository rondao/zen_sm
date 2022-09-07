use eframe::{
    egui::{Response, Ui},
    emath::RectTransform,
    epaint::{Color32, Pos2, Rect, Stroke, Vec2},
};

pub struct SelectableArea {
    area_by_selection: Rect,
    selection: Option<Rect>,
}

pub enum Selectable {
    Hover(Rect),
    Dragging(Rect),
    Selected(Rect),
}

impl SelectableArea {
    pub fn new(area: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            area_by_selection: Self::sizes(area, size),
            selection: None,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        widget_rect: Rect,
        widget_response: &Response,
    ) -> Option<Selectable> {
        let transform_area_to_screen = RectTransform::from_to(self.area_by_selection, widget_rect);
        let transform_screen_to_area = transform_area_to_screen.inverse();

        widget_response.hover_pos().and_then(|hover_pos| {
            let pointer_selection = (transform_screen_to_area * hover_pos).floor();
            let selection_position = transform_area_to_screen * pointer_selection;

            if !widget_rect.contains(selection_position + Vec2 { x: 0.1, y: 0.1 }) {
                return None;
            }

            let single_selection = Rect {
                min: pointer_selection,
                max: pointer_selection + Vec2::DOWN + Vec2::RIGHT,
            };

            if widget_response.drag_started() || widget_response.secondary_clicked() {
                self.selection = Some(single_selection);
            } else if widget_response.dragged() {
                if let Some(selection) = self.selection.as_mut() {
                    selection.max = pointer_selection + Vec2::DOWN + Vec2::RIGHT;
                };
            }

            // Draw a rectangle around the selection.
            self.selection
                .or(Some(single_selection))
                .and_then(|selection| {
                    let screen_selection = transform_area_to_screen.transform_rect(selection);
                    let painter = ui.painter_at(widget_rect);

                    painter.rect_stroke(screen_selection, 1.0, Stroke::new(2.0, Color32::WHITE));

                    if widget_response.drag_released() || widget_response.secondary_clicked() {
                        Some(Selectable::Selected(selection))
                    } else if widget_response.dragged() {
                        Some(Selectable::Dragging(Rect::from_min_size(
                            selection_position,
                            screen_selection.size(),
                        )))
                    } else {
                        Some(Selectable::Hover(Rect::from_min_size(
                            selection_position,
                            screen_selection.size(),
                        )))
                    }
                })
        })
    }

    pub fn set_sizes(&mut self, area: [f32; 2], size: [f32; 2]) {
        self.area_by_selection = Self::sizes(area, size);
    }

    pub fn position(&self) -> Option<Pos2> {
        self.selection.and_then(|selection| Some(selection.min))
    }

    pub fn unselect(&mut self) {
        self.selection = None;
    }

    fn sizes(area: [f32; 2], size: [f32; 2]) -> Rect {
        Rect::from_min_size(
            Pos2::ZERO,
            Vec2 {
                x: area[0] / size[0],
                y: area[1] / size[1],
            },
        )
    }
}
