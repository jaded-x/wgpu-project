use super::state::State;

pub struct UI {
    plot_id: egui::TextureId,
}

impl UI {
    pub fn new(plot_id: egui::TextureId) -> Self {
        Self {
            plot_id,
        }
    }

    pub fn ui(&mut self, context: &egui::Context) {
        egui::SidePanel::left("egui_demo_panel")
        .resizable(false)
        .default_width(150.0)
        .show(&context, |ui| {
            egui::trace!(ui);
            ui.vertical_centered(|ui| {
                ui.heading("✒ egui demos");
            });

            ui.separator();

            use egui::special_emojis::{GITHUB, TWITTER};
            ui.hyperlink_to(
                format!("{} egui on GitHub", GITHUB),
                "https://github.com/emilk/egui",
            );
            ui.hyperlink_to(
                format!("{} @ernerfeldt", TWITTER),
                "https://twitter.com/ernerfeldt",
            );
        });
    }
}