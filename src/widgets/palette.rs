use eframe::{
    egui::{self, color, color_picker, containers, Context, Id, Response, Ui},
    epaint::{Color32, Vec2},
};
use zen::graphics::{
    self,
    palette::{COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES},
    Palette, Rgb888,
};

use super::editor::Editor;

const PALETTE_SIZE: [usize; 2] = [COLORS_BY_SUB_PALETTE, NUMBER_OF_SUB_PALETTES];
const SELECTION_SIZE: [f32; 2] = [1.0, 1.0];

pub struct PaletteEditor {
    pub editor: Editor,
    color_edit_popup_id: Id, // ID for the Color Picker Popup.
    editing_color: Color32,  // Store the color being edited by the Color Picker Popup.
}

impl Default for PaletteEditor {
    fn default() -> Self {
        Self {
            editor: Editor::new("GraphicsEditor".to_string()),
            color_edit_popup_id: Id::new("palette_color_popup_id"),
            editing_color: Color32::default(),
        }
    }
}

impl PaletteEditor {
    pub fn ui(&mut self, ui: &mut Ui, palette: &mut Palette, widget_size: Vec2) -> Response {
        let mut response = self
            .editor
            .ui(ui, PALETTE_SIZE, SELECTION_SIZE, widget_size);

        // Open a color picker and select the color.
        if response.secondary_clicked() {
            let selected = self.editor.selected.unwrap();

            let palette_color: graphics::Rgb888 =
                palette.sub_palettes[selected.y as usize].colors[selected.x as usize].into();

            self.editing_color =
                color::Color32::from_rgb(palette_color.r, palette_color.g, palette_color.b);

            // Position the color picker popup at the click position.
            egui::Area::new(self.color_edit_popup_id)
                .current_pos(response.interact_pointer_pos().unwrap())
                .show(ui.ctx(), |_ui| {});

            if !ui.memory().is_popup_open(self.color_edit_popup_id) {
                ui.memory().toggle_popup(self.color_edit_popup_id);
            }
        };

        // Color picker is open.
        if ui.memory().is_popup_open(self.color_edit_popup_id) {
            self.ui_color_picker(ui, &mut response);
            if response.changed() {
                if let Some(selected_index) = self.editor.selected {
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
            self.editor.selected = None;
        }
    }

    pub fn load_texture(&mut self, ctx: &Context, colors: Vec<Rgb888>) {
        self.editor.load_texture(ctx, colors, PALETTE_SIZE);
    }
}
