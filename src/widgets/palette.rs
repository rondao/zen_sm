use eframe::{
    egui::{self, color, color_picker, containers, emath},
    epi,
};
use zen::graphics;

pub struct Palette {
    texture_id: Option<egui::TextureId>,
    color: [f32; 3],
    color_popup_id: egui::Id,
    selected_color: Option<egui::Pos2>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            texture_id: None,
            color: [0.0; 3],
            color_popup_id: egui::Id::new("palette_color_popup_id"),
            selected_color: None,
        }
    }
}

impl Palette {
    pub fn load_texture(&mut self, frame: &mut epi::Frame<'_>, palette: &graphics::Palette) {
        if let Some(texture_id) = self.texture_id {
            frame.tex_allocator().free(texture_id);
        }

        let pixels: Vec<egui::Color32> = palette
            .to_colors()
            .into_iter()
            .map(|color| egui::Color32::from_rgb(color.r, color.g, color.b))
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
        palette: &mut graphics::Palette,
        size: egui::Vec2,
    ) -> egui::Response {
        // Attach some meta-data to the response which can be used by screen readers:
        // response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, *on, ""));
        let (widget_rect, mut response) = ui.allocate_exact_size(size, egui::Sense::hover());

        if let Some(texture_id) = self.texture_id {
            // Add the palette as an image.
            response = ui
                .put(widget_rect, egui::Image::new(texture_id, size))
                .interact(egui::Sense::click());

            // Pointer's screen to palette's color transformations.
            let to_screen = emath::RectTransform::from_to(
                egui::Rect::from_min_size(
                    egui::Pos2 { x: 0.0, y: 0.0 },
                    egui::Vec2 { x: 16.0, y: 16.0 },
                ),
                widget_rect,
            );
            let from_screen = to_screen.inverse();

            // Draw a rectangle on hovered color.
            if let Some(hover_pos) = response.hover_pos() {
                let palette_pos = (from_screen * hover_pos).floor();
                let selection_pos = to_screen * palette_pos;

                // Handle positions inside the Widget. Use epsilon to avoid on bound.
                if widget_rect.contains(selection_pos + egui::Vec2 { x: 0.1, y: 0.1 }) {
                    let rect = egui::Rect {
                        min: selection_pos,
                        max: selection_pos
                            + egui::Vec2 {
                                x: (size.x / 16.0),
                                y: (size.y / 16.0),
                            },
                    };

                    let painter = ui.painter_at(widget_rect);
                    painter.rect_stroke(rect, 1.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

                    if response.secondary_clicked() {
                        let palette_color: graphics::Rgb888 = palette.sub_palettes
                            [palette_pos.y as usize]
                            .colors[palette_pos.x as usize]
                            .into();
                        self.color = [
                            palette_color.r as f32 / 255.0,
                            palette_color.g as f32 / 255.0,
                            palette_color.b as f32 / 255.0,
                        ];
                        println!("{:?}", self.color);
                        self.selected_color = Some(palette_pos);

                        if !ui.memory().is_popup_open(self.color_popup_id) {
                            ui.memory().toggle_popup(self.color_popup_id);
                        }
                    }
                }
            }

            self.ui_color_picker(ui, &mut response);
            if response.changed() {
                if let Some(selected_color) = self.selected_color {
                    palette.sub_palettes[selected_color.y as usize].colors
                        [selected_color.x as usize] = graphics::Rgb888 {
                        r: (self.color[0] * 255.0) as u8,
                        g: (self.color[1] * 255.0) as u8,
                        b: (self.color[2] * 255.0) as u8,
                    }
                    .into();
                }
            }
        }

        response
    }

    fn ui_color_picker(&mut self, ui: &mut egui::Ui, response: &mut egui::Response) {
        if ui.memory().is_popup_open(self.color_popup_id) {
            let area_response = egui::Area::new(self.color_popup_id)
                .order(egui::Order::Foreground)
                .default_pos(response.rect.max)
                .show(ui.ctx(), |ui| {
                    ui.spacing_mut().slider_width = 256.0;
                    containers::Frame::popup(ui.style()).show(ui, |ui| {
                        let mut hsva = color::Hsva::from_rgb(self.color);
                        if color_picker::color_picker_hsva_2d(
                            ui,
                            &mut hsva,
                            color_picker::Alpha::Opaque,
                        ) {
                            self.color = hsva.to_rgb();
                            response.mark_changed();
                        }
                    });
                });

            if !response.secondary_clicked()
                && (ui.input().key_pressed(egui::Key::Escape) || area_response.clicked_elsewhere())
            {
                ui.memory().close_popup();
            }
        }
    }
}
