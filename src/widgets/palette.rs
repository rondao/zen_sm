use eframe::{
    egui::{self, emath, Color32, Image, Pos2, Rect, Stroke, TextureId, Vec2},
    epi,
};
use zen::graphics::Palette;

pub struct Widget<'a, Palette> {
    pub palette: &'a Palette,
    pub texture_id: Option<TextureId>,
}

impl Widget<'_, Palette> {
    pub fn load_texture(&mut self, frame: &mut epi::Frame<'_>) {
        if let Some(texture_id) = self.texture_id {
            frame.tex_allocator().free(texture_id);
        }

        let pixels: Vec<Color32> = self
            .palette
            .to_colors()
            .into_iter()
            .map(|color| Color32::from_rgb(color.r, color.g, color.b))
            .collect();

        self.texture_id = Some(
            frame
                .tex_allocator()
                .alloc_srgba_premultiplied((16, 16), &pixels),
        );
    }

    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        frame: &mut epi::Frame<'_>,
        size: egui::Vec2,
    ) -> egui::Response {
        if self.texture_id.is_none() {
            self.load_texture(frame);
        }
        let texture_id = self.texture_id.unwrap();

        // Attach some meta-data to the response which can be used by screen readers:
        // response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));
        let (widget_rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

        // Add the palette as an image.
        let response = ui
            .put(widget_rect, Image::new(texture_id, size))
            .interact(egui::Sense::click());

        // Pointer's screen to palette's color transformations.
        let to_screen = emath::RectTransform::from_to(
            Rect::from_min_size(Pos2 { x: 0.0, y: 0.0 }, Vec2 { x: 16.0, y: 16.0 }),
            widget_rect,
        );
        let from_screen = to_screen.inverse();

        // Draw a rectangle on hovered color.
        if let Some(hover_pos) = response.hover_pos() {
            let interact_pos = to_screen * (from_screen * hover_pos).floor();
            // Handle positions inside the Widget. Use epsilon to avoid on bound.
            if widget_rect.contains(interact_pos + Vec2 { x: 0.1, y: 0.1 }) {
                let rect = Rect {
                    min: interact_pos,
                    max: interact_pos
                        + Vec2 {
                            x: (size.x / 16.0),
                            y: (size.y / 16.0),
                        },
                };

                let painter = ui.painter_at(widget_rect);
                painter.rect_stroke(rect, 1.0, Stroke::new(2.0, Color32::WHITE));
            }
        }

        response
    }
}
