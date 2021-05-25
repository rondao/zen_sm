use eframe::{
    egui::{self, color, color_picker, containers, emath},
    epi,
};
use zen::graphics;

pub struct Palette {
    texture_id: Option<egui::TextureId>,
    color_popup_id: egui::Id,
    selected_color: [f32; 3],
    selected_position: Option<egui::Pos2>,
}

impl Default for Palette {
    fn default() -> Self {
        Self {
            texture_id: None,
            selected_color: [0.0; 3],
            color_popup_id: egui::Id::new("palette_color_popup_id"),
            selected_position: None,
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

            let mut selected_position = None;
            if self.selected_position.is_none() {
                // Select the hovering position.
                if let Some(hover_pos) = response.hover_pos() {
                    let color_pos = (from_screen * hover_pos).floor();
                    // Handle only positions inside the Widget. Use epsilon to avoid on bounds.
                    selected_position = if widget_rect
                        .contains(to_screen * color_pos + egui::Vec2 { x: 0.1, y: 0.1 })
                    {
                        Some(color_pos)
                    } else {
                        None
                    }
                };
            } else {
                selected_position = self.selected_position;
            }

            if let Some(selected_position) = selected_position {
                // Draw a rectangle around the selected color.
                let rect = egui::Rect {
                    min: to_screen * selected_position,
                    max: to_screen * selected_position
                        + egui::Vec2 {
                            x: (size.x / 16.0),
                            y: (size.y / 16.0),
                        },
                };

                let painter = ui.painter_at(widget_rect);
                painter.rect_stroke(rect, 1.0, egui::Stroke::new(2.0, egui::Color32::WHITE));

                // Open a color picker.
                if response.secondary_clicked() {
                    self.selected_position = Some(selected_position);

                    let palette_color: graphics::Rgb888 = palette.sub_palettes
                        [selected_position.y as usize]
                        .colors[selected_position.x as usize]
                        .into();

                    self.selected_color = [
                        palette_color.r as f32 / 255.0,
                        palette_color.g as f32 / 255.0,
                        palette_color.b as f32 / 255.0,
                    ];
                    println!("{:?}", self.selected_color);

                    if !ui.memory().is_popup_open(self.color_popup_id) {
                        ui.memory().toggle_popup(self.color_popup_id);
                    }
                }
            }

            // Color picker is open.
            if ui.memory().is_popup_open(self.color_popup_id) {
                self.ui_color_picker(ui, &mut response);
                if response.changed() {
                    if let Some(selected_position) = self.selected_position {
                        palette.sub_palettes[selected_position.y as usize].colors
                            [selected_position.x as usize] = graphics::Rgb888 {
                            r: (self.selected_color[0] * 255.0) as u8,
                            g: (self.selected_color[1] * 255.0) as u8,
                            b: (self.selected_color[2] * 255.0) as u8,
                        }
                        .into();
                    }
                }
            }
        }

        response
    }

    fn ui_color_picker(&mut self, ui: &mut egui::Ui, response: &mut egui::Response) {
        let area_response = egui::Area::new(self.color_popup_id)
            .order(egui::Order::Foreground)
            .default_pos(response.rect.max)
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
            self.selected_position = None;
        }
    }
}
