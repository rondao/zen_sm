use eframe::{
    egui::{Image, Response, Sense, Ui},
    emath::RectTransform,
    epaint::{Color32, ColorImage, Pos2, Rect, Stroke, TextureHandle, Vec2},
};
use zen::graphics::Rgb888;

pub struct Editor {
    name: String,
    texture: Option<TextureHandle>, // Texture for the palette colors.
    pub selected: Option<Pos2>,     // Position to draw the square selection.
}

impl Editor {
    pub fn new(name: String) -> Self {
        Self {
            name,
            texture: None,
            selected: None,
        }
    }

    pub fn ui(
        &mut self,
        ui: &mut Ui,
        colors: Vec<Rgb888>,
        texture_size: [usize; 2],
        selection_size: [f32; 2],
        widget_size: Vec2,
    ) -> Response {
        let (widget_rect, response) = create_texture_area(
            ui,
            self.texture.get_or_insert(ui.ctx().load_texture(
                &self.name,
                ColorImage::from_rgba_unmultiplied(
                    texture_size,
                    &rgb888s_to_texture(&colors),
                    // egui::epaint::textures::TextureFilter::Nearest, // This feature is already merged on master, to be available later.
                ),
            )),
            widget_size,
        );

        // Pointer's screen to palette's color transformations.
        let transform_selection_to_screen = RectTransform::from_to(
            Rect::from_min_size(
                Pos2 { x: 0.0, y: 0.0 },
                Vec2 {
                    x: texture_size[0] as f32 / selection_size[0],
                    y: texture_size[1] as f32 / selection_size[1],
                },
            ),
            widget_rect,
        );
        let transform_screen_to_selection = transform_selection_to_screen.inverse();

        // Handle only positions inside the Widget.
        let hover_position = response.hover_pos().and_then(|hover_pos| {
            let hover_position = (transform_screen_to_selection * hover_pos).floor();
            let in_screen_pos = transform_selection_to_screen * hover_position;

            // Use epsilon to avoid out of bounds.
            widget_rect
                .contains(in_screen_pos + Vec2 { x: 0.1, y: 0.1 })
                .then(|| {
                    if response.secondary_clicked() {
                        self.selected = Some(hover_position);
                    } else if response.clicked() {
                        self.selected = None
                    }
                    hover_position
                })
        });

        // Draw a rectangle around the selected color.
        let current_selection = self.selected.or(hover_position);
        if let Some(current_selection) = current_selection {
            let rect = Rect {
                min: transform_selection_to_screen * current_selection,
                max: transform_selection_to_screen * current_selection
                    + Vec2 {
                        x: (widget_size.x / texture_size[0] as f32) * selection_size[0],
                        y: (widget_size.y / texture_size[1] as f32) * selection_size[1],
                    },
            };

            let painter = ui.painter_at(widget_rect);
            painter.rect_stroke(rect, 1.0, Stroke::new(2.0, Color32::WHITE));
        };

        response
    }

    pub fn invalidate_texture(&mut self) {
        self.texture = None;
    }
}

fn create_texture_area(
    ui: &mut Ui,
    texture: &TextureHandle,
    widget_size: Vec2,
) -> (Rect, Response) {
    let (widget_rect, _) = ui.allocate_exact_size(widget_size, Sense::hover());

    let response = ui
        .put(widget_rect, Image::new(texture, widget_size))
        .interact(Sense::click());

    (widget_rect, response)
}

fn rgb888s_to_texture(colors: &[Rgb888]) -> Vec<u8> {
    colors.into_iter().fold(Vec::new(), |mut pixels, color| {
        pixels.push(color.r);
        pixels.push(color.g);
        pixels.push(color.b);
        pixels.push(255);
        pixels
    })
}
