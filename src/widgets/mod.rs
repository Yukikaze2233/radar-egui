mod laser_panel;
mod minimap;
mod panels;

pub use laser_panel::LaserPanel;
pub use minimap::MinimapWidget;
pub use panels::StatusPanels;

use crate::theme;

pub fn card_frame<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    egui::Frame::new()
        .fill(theme::CARD_BG)
        .corner_radius(theme::CARD_RADIUS)
        .stroke(egui::Stroke::new(1.0, theme::CARD_BORDER))
        .inner_margin(egui::Margin::symmetric(18, 14))
        .show(ui, add_contents)
        .inner
}
