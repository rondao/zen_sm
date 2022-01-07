use eframe::{
    egui::{self, Vec2},
    epi,
};
use zen::super_metroid::{self, SuperMetroid};

use crate::widgets;

#[derive(Default)]
pub struct ZenSM {
    sm: SuperMetroid,
    palette: widgets::Palette,
}

impl epi::App for ZenSM {
    fn name(&self) -> &str {
        "Zen SM"
    }

    fn setup(
        &mut self,
        _ctx: &egui::CtxRef,
        _frame: &epi::Frame,
        _storage: Option<&dyn epi::Storage>,
    ) {
        self.sm = super_metroid::load_unheadered_rom(
            "/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc",
        )
        .unwrap();
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &epi::Frame) {
        if !self.palette.is_texture_loaded() {
            self.palette
                .load_texture(frame, &self.sm.palettes[&0xC2AE5D]);
        }

        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .default_height(300.0)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    if self.palette.is_texture_loaded() {
                        let response = self.palette.ui(
                            ui,
                            self.sm.palettes.get_mut(&0xC2AE5D).unwrap(),
                            Vec2 { x: 300.0, y: 150.0 },
                        );
                        if response.changed() {
                            self.palette
                                .load_texture(frame, &self.sm.palettes[&0xC2AE5D]);
                        }
                    }
                });
            });
    }
}
