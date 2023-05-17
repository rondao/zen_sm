use eframe::{
    egui::Response,
    emath::RectTransform,
    epaint::{Pos2, Rect, Vec2},
};

pub struct SelectableArea {
    area_by_selection: Rect,
    selection: Option<Rect>,
}

pub enum Selectable {
    SelectedHovering(Rect),
    UnselectedHovering(Rect),
    Dragging(Rect),
    Selected(Rect),
    Clicked(Pos2),
}

impl SelectableArea {
    pub fn new(area: [f32; 2], size: [f32; 2]) -> Self {
        Self {
            area_by_selection: Self::sizes(area, size),
            selection: None,
        }
    }

    pub fn ui(&mut self, widget_rect: Rect, widget_response: &Response) -> Option<Selectable> {
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

            // Return proper Selectable action.
            if widget_response.secondary_clicked() {
                self.selection = Some(single_selection);
                Some(Selectable::Selected(single_selection))
            } else if widget_response.drag_started_by(eframe::egui::PointerButton::Secondary) {
                self.selection = Some(single_selection);
                Some(Selectable::Dragging(
                    transform_area_to_screen.transform_rect(single_selection),
                ))
            } else if widget_response.dragged_by(eframe::egui::PointerButton::Secondary) {
                self.selection.and_then(|selection| {
                    Some(Selectable::Dragging(
                        transform_area_to_screen.transform_rect(Self::make_selection_area(
                            selection.min,
                            pointer_selection,
                        )),
                    ))
                })
            } else if widget_response.drag_released_by(eframe::egui::PointerButton::Secondary) {
                self.selection.as_mut().and_then(|selection| {
                    *selection = Self::make_selection_area(selection.min, pointer_selection);
                    Some(Selectable::Selected(*selection))
                })
            } else if widget_response.clicked() {
                Some(Selectable::Clicked(pointer_selection))
            } else {
                if self.selection.is_some() {
                    Some(Selectable::SelectedHovering(Rect::from_min_size(
                        selection_position,
                        transform_area_to_screen
                            .transform_rect(self.selection.unwrap())
                            .size(),
                    )))
                } else {
                    Some(Selectable::UnselectedHovering(
                        transform_area_to_screen.transform_rect(single_selection),
                    ))
                }
            }
        })
    }

    pub fn set_sizes(&mut self, area: [f32; 2], size: [f32; 2]) {
        self.area_by_selection = Self::sizes(area, size);
    }

    pub fn set_selection(&mut self, selection: Rect) {
        self.selection = Some(selection);
    }

    pub fn position(&self) -> Option<Pos2> {
        self.selection.and_then(|selection| Some(selection.min))
    }

    pub fn unselect(&mut self) {
        self.selection = None;
    }

    fn make_selection_area(first_click: Pos2, second_click: Pos2) -> Rect {
        let top_left = Pos2 {
            x: first_click.x.min(second_click.x),
            y: first_click.y.min(second_click.y),
        };
        let bottom_right = Pos2 {
            x: if first_click.x > second_click.x {
                first_click.x + 1.0
            } else {
                second_click.x + 1.0
            },
            y: if first_click.y > second_click.y {
                first_click.y + 1.0
            } else {
                second_click.y + 1.0
            },
        };

        Rect::from_min_max(top_left, bottom_right)
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
