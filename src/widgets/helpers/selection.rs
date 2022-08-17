use eframe::{
    egui::{Response, Ui},
    emath::RectTransform,
    epaint::{Color32, Pos2, Rect, Stroke, Vec2},
};

pub struct Selection {
    pub position: Option<Pos2>,
    area: [f32; 2],
    size: [f32; 2],
}

impl Selection {
    pub fn new(area: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            position: None,
            area,
            size,
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, widget_rect: Rect, widget_response: &Response) {
        // Pointer's screen to area transformations.
        let transform_area_to_screen = RectTransform::from_to(
            Rect::from_min_size(
                Pos2 { x: 0.0, y: 0.0 },
                Vec2 {
                    x: self.area[0] / self.size[0],
                    y: self.area[1] / self.size[1],
                },
            ),
            widget_rect,
        );
        let transform_screen_to_area = transform_area_to_screen.inverse();

        // Handle only positions inside the Widget.
        let hover_position = widget_response.hover_pos().and_then(|hover_pos| {
            let hover_position = (transform_screen_to_area * hover_pos).floor();
            let in_screen_pos = transform_area_to_screen * hover_position;

            // Use epsilon to avoid out of bounds.
            widget_rect
                .contains(in_screen_pos + Vec2 { x: 0.1, y: 0.1 })
                .then(|| {
                    if widget_response.secondary_clicked() {
                        self.position = Some(hover_position);
                    } else if widget_response.clicked() {
                        self.position = None
                    }
                    hover_position
                })
        });

        // Draw a rectangle around the selected color.
        let current_selection = self.position.or(hover_position);
        if let Some(current_selection) = current_selection {
            let rect = Rect {
                min: transform_area_to_screen * current_selection,
                max: transform_area_to_screen * current_selection
                    + Vec2 {
                        x: (widget_rect.width() / self.area[0]) * self.size[0],
                        y: (widget_rect.height() / self.area[1]) * self.size[1],
                    },
            };

            let painter = ui.painter_at(widget_rect);
            painter.rect_stroke(rect, 1.0, Stroke::new(2.0, Color32::WHITE));
        };
    }
}
