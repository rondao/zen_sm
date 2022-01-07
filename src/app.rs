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

    fn setup(&mut self, _ctx: &egui::CtxRef) {
        self.sm = super_metroid::load_unheadered_rom(
            "/home/rondao/dev/snes_data/Super Metroid (JU) [!].smc",
        )
        .unwrap();
    }

    fn update(&mut self, ctx: &egui::CtxRef, frame: &mut epi::Frame<'_>) {
        if !self.palette.is_texture_loaded() {
            self.palette
                .load_texture(frame, &self.sm.palettes[&0xC2AE5D]);
        }

        if self.palette.is_texture_loaded() {
            egui::Area::new(1).show(ctx, |ui| {
                let response = self.palette.ui(
                    ui,
                    self.sm.palettes.get_mut(&0xC2AE5D).unwrap(),
                    Vec2 {
                        x: 600_f32,
                        y: 600_f32,
                    },
                );
                if response.changed() {
                    self.palette
                        .load_texture(frame, &self.sm.palettes[&0xC2AE5D]);
                }
            });
        } else {
            egui::CentralPanel::default().show(ctx, |ui| ui.label("Load a ROM."));
        }
    }
}
