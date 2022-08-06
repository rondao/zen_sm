use eframe::{
    egui::{self, emath},
    epaint::TextureHandle,
};
use zen::graphics::{self, gfx::TILE_SIZE, Rgb888};

pub struct GraphicsEditor {
    texture: Option<egui::TextureHandle>, // Texture for the palette colors.
    selected_tile_pos: Option<egui::Pos2>, // Position to draw the square selection.
}

impl Default for GraphicsEditor {
    fn default() -> Self {
        Self {
            texture: None,
            selected_tile_pos: None,
        }
    }
}

impl GraphicsEditor {
    pub fn ui(
        &mut self,
        ui: &mut egui::Ui,
        gfx: &graphics::Gfx,
        sub_palette: &graphics::SubPalette,
        size: egui::Vec2,
    ) -> egui::Response {
        let (widget_rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());

        let colors: Vec<Rgb888> = gfx
            .to_indexed_colors()
            .into_iter()
            .map(|idx_color| sub_palette.colors[idx_color as usize].into())
            .collect();

        let texture = self
            .texture
            .get_or_insert(pixels_to_texture(ui.ctx(), &colors, gfx.size()));

        // Add the palette as an image.
        let response = ui
            .put(widget_rect, egui::Image::new(texture, size))
            .interact(egui::Sense::click());

        // Pointer's screen to palette's color transformations.
        let transform_to_screen = emath::RectTransform::from_to(
            egui::Rect::from_min_size(
                egui::Pos2 { x: 0.0, y: 0.0 },
                egui::Vec2 {
                    x: 16.0,
                    y: (gfx.tiles.len() / (2 * TILE_SIZE)) as f32,
                },
            ),
            widget_rect,
        );
        let transform_to_selection = transform_to_screen.inverse();

        // Handle only positions inside the Widget.
        let hover_position = response.hover_pos().and_then(|hover_pos| {
            let hover_position = (transform_to_selection * hover_pos).floor();
            let in_screen_pos = transform_to_screen * hover_position;

            // Use epsilon to avoid out of bounds.
            widget_rect
                .contains(in_screen_pos + egui::Vec2 { x: 0.1, y: 0.1 })
                .then(|| {
                    if response.secondary_clicked() {
                        self.selected_tile_pos = Some(hover_position);
                    } else if response.clicked() {
                        self.selected_tile_pos = None
                    }
                    hover_position
                })
        });

        // Draw a rectangle around the selected color.
        let current_selection = self.selected_tile_pos.or(hover_position);
        if let Some(current_selection) = current_selection {
            let rect = egui::Rect {
                min: transform_to_screen * current_selection,
                max: transform_to_screen * current_selection
                    + egui::vec2(size.x / TILE_SIZE as f32, (size.y / TILE_SIZE as f32) / 4.0),
            };

            let painter = ui.painter_at(widget_rect);
            painter.rect_stroke(rect, 1.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
        };

        response
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
