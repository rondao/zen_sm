use eframe::{
    egui::{self, color, color_picker, containers, emath},
    epi,
};
use zen::graphics;

pub struct Palette {
    texture_id: Option<egui::TextureId>,
    color_popup_id: egui::Id,
    selected_color: [f32; 3],
    selected_index: Option<egui::Pos2>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            texture_id: None,
            color_popup_id: egui::Id::new("palette_color_popup_id"),
            selected_color: [0.0; 3],
            selected_index: None,
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
            let to_color_palette = to_screen.inverse();

            // Check the hover or click selection.
            let current_selection = if response.hover_pos().is_some() {
                let screen_pos = response.hover_pos().unwrap();
                let color_palette_pos = (to_color_palette * screen_pos).floor();

                // Handle only positions inside the Widget. Use epsilon to avoid on bounds.
                if widget_rect
                    .contains(to_screen * color_palette_pos + egui::Vec2 { x: 0.1, y: 0.1 })
                {
                    // Open a color picker and select the color.
                    if response.secondary_clicked() {
                        self.selected_index = Some(color_palette_pos);

                        let palette_color: graphics::Rgb888 = palette.sub_palettes
                            [color_palette_pos.y as usize]
                            .colors[color_palette_pos.x as usize]
                            .into();

                        self.selected_color = [
                            palette_color.r as f32 / 255.0,
                            palette_color.g as f32 / 255.0,
                            palette_color.b as f32 / 255.0,
                        ];
                        println!("{:?}", self.selected_color);

                        // Position the color picker popup at the click position.
                        egui::Area::new(self.color_popup_id)
                            .current_pos(to_screen * color_palette_pos)
                            .show(ui.ctx(), |_ui| {});

                        if !ui.memory().is_popup_open(self.color_popup_id) {
                            ui.memory().toggle_popup(self.color_popup_id);
                        }
                    };
                    // If we don't have a current selection, use the hover selection.
                    self.selected_index.or(Some(color_palette_pos))
                } else {
                    // Otherwise, use the current selection.
                    self.selected_index
                }
            } else {
                // If we are outsize the widget area, use the current selection.
                self.selected_index
            };

            // Draw a rectangle around the selected color.
            if let Some(current_selection) = current_selection {
                let rect = egui::Rect {
                    min: to_screen * current_selection,
                    max: to_screen * current_selection
                        + egui::Vec2 {
                            x: (size.x / 16.0),
                            y: (size.y / 16.0),
                        },
                };

                let painter = ui.painter_at(widget_rect);
                painter.rect_stroke(rect, 1.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
            };

            // Color picker is open.
            if ui.memory().is_popup_open(self.color_popup_id) {
                self.ui_color_picker(ui, &mut response);
                if response.changed() {
                    if let Some(selected_index) = self.selected_index {
                        palette.sub_palettes[selected_index.y as usize].colors
                            [selected_index.x as usize] = graphics::Rgb888 {
                            r: (self.selected_color[0] * 255.0) as u8,
                            g: (self.selected_color[1] * 255.0) as u8,
                            b: (self.selected_color[2] * 255.0) as u8,
                        }
                        .into();
                    }
                }
            };
        }

        response
    }

    fn ui_color_picker(&mut self, ui: &mut egui::Ui, response: &mut egui::Response) {
        let area_response = egui::Area::new(self.color_popup_id)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().slider_width = 256.0;
                containers::Frame::popup(ui.style()).show(ui, |ui| {
                    let mut hsva = color::Hsva::from_rgb(self.selected_color);
                    if color_picker::color_picker_hsva_2d(
                        ui,
                        &mut hsva,
                        color_picker::Alpha::Opaque,
                    ) {
                        self.selected_color = hsva.to_rgb();
                        response.mark_changed();
                    }
                });
            });

        if !response.secondary_clicked()
            && (ui.input().key_pressed(egui::Key::Escape) || area_response.clicked_elsewhere())
        {
            ui.memory().close_popup();
            self.selected_index = None;
        }
    }
}
