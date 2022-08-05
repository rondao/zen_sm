use eframe::{
    egui::{self, color, color_picker, containers, emath},
    epaint::TextureHandle,
};
use zen::graphics::{
    self,
    palette::{COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES},
    Rgb888,
};

pub struct PaletteEditor {
    texture: Option<egui::TextureHandle>, // Texture for the palette colors.
    color_edit_popup_id: egui::Id,        // ID for the Color Picker Popup.
    editing_color: color::Color32,        // Store the color being edited by the Color Picker Popup.
    selected_color_pos: Option<egui::Pos2>, // Position to draw the square selection.
}

impl Default for PaletteEditor {
    fn default() -> Self {
        Self {
            texture: None,
            color_edit_popup_id: egui::Id::new("palette_color_popup_id"),
            editing_color: color::Color32::default(),
            selected_color_pos: None,
        }
    }
}

impl PaletteEditor {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        palette: &mut graphics::Palette,
        size: egui::Vec2,
    ) -> egui::Response {
        let (widget_rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

        let texture = self.texture.get_or_insert(pixels_to_texture(
            ui.ctx(),
            &palette.to_colors(),
            [COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES],
        ));

        // Add the palette as an image.
        let mut response = ui
            .put(widget_rect, egui::Image::new(texture, size))
            .interact(egui::Sense::click());

        // Pointer's screen to palette's color transformations.
        let transform_to_screen = emath::RectTransform::from_to(
            egui::Rect::from_min_size(
                egui::Pos2 { x: 0.0, y: 0.0 },
                egui::Vec2 { x: 16.0, y: 8.0 },
            ),
            widget_rect,
        );
        let transform_to_color_palette = transform_to_screen.inverse();

        // Handle only positions inside the Widget.
        let hover_position = response.hover_pos().and_then(|hover_pos| {
            let color_palette_pos = (transform_to_color_palette * hover_pos).floor();
            let color_screen_pos = transform_to_screen * color_palette_pos;

            // Use epsilon to avoid out of bounds.
            widget_rect
                .contains(color_screen_pos + egui::Vec2 { x: 0.1, y: 0.1 })
                .then(|| {
                    // Open a color picker and select the color.
                    if response.secondary_clicked() {
                        self.selected_color_pos = Some(color_palette_pos);

                        let palette_color: graphics::Rgb888 = palette.sub_palettes
                            [color_palette_pos.y as usize]
                            .colors[color_palette_pos.x as usize]
                            .into();

                        self.editing_color = color::Color32::from_rgb(
                            palette_color.r,
                            palette_color.g,
                            palette_color.b,
                        );

                        // Position the color picker popup at the click position.
                        egui::Area::new(self.color_edit_popup_id)
                            .current_pos(color_screen_pos + egui::vec2(size.x / 16.0, size.y / 8.0))
                            .show(ui.ctx(), |_ui| {});

                        if !ui.memory().is_popup_open(self.color_edit_popup_id) {
                            ui.memory().toggle_popup(self.color_edit_popup_id);
                        }
                    };
                    color_palette_pos
                })
        });

        // Draw a rectangle around the selected color.
        let current_selection = self.selected_color_pos.or(hover_position);
        if let Some(current_selection) = current_selection {
            let rect = egui::Rect {
                min: transform_to_screen * current_selection,
                max: transform_to_screen * current_selection
                    + egui::vec2(size.x / 16.0, size.y / 8.0),
            };

            let painter = ui.painter_at(widget_rect);
            painter.rect_stroke(rect, 1.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
        };

        // Color picker is open.
        if ui.memory().is_popup_open(self.color_edit_popup_id) {
            self.ui_color_picker(ui, &mut response);
            if response.changed() {
                self.invalidate_texture();
                if let Some(selected_index) = self.selected_color_pos {
                    palette.sub_palettes[selected_index.y as usize].colors
                        [selected_index.x as usize] = graphics::Rgb888 {
                        r: self.editing_color.r(),
                        g: self.editing_color.g(),
                        b: self.editing_color.b(),
                    }
                    .into();
                }
            }
        };

        response
    }

    fn ui_color_picker(&mut self, ui: &mut egui::Ui, response: &mut egui::Response) {
        let area_response = egui::Area::new(self.color_edit_popup_id)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                ui.spacing_mut().slider_width = 256.0;
                containers::Frame::popup(ui.style()).show(ui, |ui| {
                    let mut hsva = self.editing_color.into();
                    if color_picker::color_picker_hsva_2d(
                        ui,
                        &mut hsva,
                        color_picker::Alpha::Opaque,
                    ) {
                        self.editing_color = hsva.into();
                        response.mark_changed();
                    }
                });
            });

        if !response.secondary_clicked()
            && (ui.input().key_pressed(egui::Key::Escape)
                || area_response.response.clicked_elsewhere())
        {
            ui.memory().close_popup();
            self.selected_color_pos = None;
        }
    }

    pub fn invalidate_texture(&mut self) {
        self.texture = None;
    }
}

fn pixels_to_texture(ctx: &egui::Context, colors: &[Rgb888], size: [usize; 2]) -> TextureHandle {
    let pixels = colors.into_iter().fold(Vec::new(), |mut pixels, color| {
        pixels.push(color.r);
        pixels.push(color.g);
        pixels.push(color.b);
        pixels.push(255);
        pixels
    });

    ctx.load_texture(
        "palette",
        egui::ColorImage::from_rgba_unmultiplied(
            size,
            &pixels,
            // egui::epaint::textures::TextureFilter::Nearest, // This feature is already merged on master, to be available later.
        ),
    )
}
