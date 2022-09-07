use eframe::{
    egui::{self, color, color_picker, containers, Context, Id, Response, Sense, Ui},
    epaint::{Color32, Rect, Vec2},
};
use zen::graphics::{
    self,
    palette::{COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES},
    Palette, Rgb888,
};

use super::helpers::{painted_selectable_area::PaintedSelectableArea, texture::Texture};

const PALETTE_SIZE: [usize; 2] = [COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES];
const SELECTION_SIZE: [f32; 2] = [1.0, 1.0];

pub struct PaletteEditor {
    pub texture: Texture,
    selectable_area: PaintedSelectableArea,
    color_edit_popup_id: Id, // ID for the Color Picker Popup.
    editing_color: Color32,  // Store the color being edited by the Color Picker Popup.
}

impl Default for PaletteEditor {
    fn default() -> Self {
        Self {
            texture: Texture::new("PaletteEditor".to_string()),
            selectable_area: PaintedSelectableArea::new(
                [PALETTE_SIZE[0] as f32, PALETTE_SIZE[1] as f32],
                SELECTION_SIZE,
            ),
            color_edit_popup_id: Id::new("palette_color_popup_id"),
            editing_color: Color32::default(),
        }
    }
}

impl PaletteEditor {
    pub fn ui(&mut self, ui: &mut Ui, palette: &mut Palette) -> (Response, Rect) {
        let (widget_rect, mut widget_response) =
            ui.allocate_exact_size(Vec2 { x: 300.0, y: 150.0 }, Sense::click());

        self.texture.ui(ui, widget_rect);
        self.selectable_area.ui(ui, widget_rect, &widget_response);

        // Open a color picker and select the color.
        if widget_response.secondary_clicked() {
            let selected = self.selectable_area.position().unwrap();

            let palette_color: graphics::Rgb888 =
                palette.sub_palettes[selected.y as usize].colors[selected.x as usize].into();

            self.editing_color =
                color::Color32::from_rgb(palette_color.r, palette_color.g, palette_color.b);

            // Position the color picker popup at the click position.
            egui::Area::new(self.color_edit_popup_id)
                .current_pos(widget_response.interact_pointer_pos().unwrap())
                .show(ui.ctx(), |_ui| {});

            if !ui.memory().is_popup_open(self.color_edit_popup_id) {
                ui.memory().toggle_popup(self.color_edit_popup_id);
            }
        };

        // Color picker is open.
        if ui.memory().is_popup_open(self.color_edit_popup_id) {
            self.ui_color_picker(ui, &mut widget_response);
            if widget_response.changed() {
                if let Some(selected_index) = self.selectable_area.position() {
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

        (widget_response, widget_rect)
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
            self.selectable_area.unselect();
        }
    }

    pub fn load_texture(&mut self, ctx: &Context, colors: Vec<Rgb888>) {
        self.texture.load_colors(ctx, colors, PALETTE_SIZE);
    }
}
