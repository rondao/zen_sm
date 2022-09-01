use eframe::{
    egui::{Response, Ui},
    emath::RectTransform,
    epaint::{Color32, Pos2, Rect, Stroke, Vec2},
};

pub struct Selection {
    pub selection: Option<Rect>,
    pub widget_size: [f32; 2],
    selection_size: [f32; 2],
}

impl Selection {
    pub fn new(area: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            selection: None,
            widget_size: area,
            selection_size: size,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        widget_rect: Rect,
        widget_response: &Response,
    ) -> Option<Rect> {
        // Pointer's screen to area transformations.
        let transform_area_to_screen = RectTransform::from_to(
            Rect::from_min_size(
                Pos2::ZERO,
                Vec2 {
                    x: self.widget_size[0] / self.selection_size[0],
                    y: self.widget_size[1] / self.selection_size[1],
                },
            ),
            widget_rect,
        );
        let transform_screen_to_area = transform_area_to_screen.inverse();

        let mut in_screen_pos = Pos2::ZERO;
        // Handle only positions inside the Widget.
        let hover_selection = widget_response.hover_pos().and_then(|hover_pos| {
            let hover_position = (transform_screen_to_area * hover_pos).floor();
            in_screen_pos = transform_area_to_screen * hover_position;

            // Use epsilon to avoid out of bounds.
            widget_rect
                .contains(in_screen_pos + Vec2 { x: 0.1, y: 0.1 })
                .then(|| {
                    if widget_response.drag_started() || widget_response.secondary_clicked() {
                        self.selection = Some(Rect {
                            min: hover_position,
                            max: hover_position + Vec2::DOWN + Vec2::RIGHT,
                        });
                    } else if widget_response.dragged() {
                        if let Some(selection) = self.selection.as_mut() {
                            selection.max = hover_position + Vec2::DOWN + Vec2::RIGHT;
                        };
                    }

                    Rect {
                        min: hover_position,
                        max: hover_position + Vec2::DOWN + Vec2::RIGHT,
                    }
                })
        });

        // Draw a rectangle around the selection.
        let current_selection = self.selection.or(hover_selection);
        current_selection.and_then(|current_selection| {
            let transformed_selection = transform_area_to_screen.transform_rect(current_selection);

            let painter = ui.painter_at(widget_rect);
            painter.rect_stroke(transformed_selection, 1.0, Stroke::new(2.0, Color32::WHITE));

            Some(Rect::from_min_size(
                in_screen_pos,
                transformed_selection.size(),
            ))
        })
    }

    pub fn position(&self) -> Option<Pos2> {
        self.selection.and_then(|selection| Some(selection.min))
    }
}
