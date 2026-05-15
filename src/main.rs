mod app;
mod laser_protocol;
mod protocol;
mod rerun_viz;
mod script_runner;
mod tcp_client;
mod theme;
mod udp_client;
mod video_stream;
mod widgets;

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("Radar HUD"),
        ..Default::default()
    };

    eframe::run_native(
        "Radar HUD",
        options,
        Box::new(|_cc| Ok(Box::new(app::RadarApp::default()))),
    )
}
