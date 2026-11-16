mod app;
mod i18n;
mod views;

use app::AmpApp;

fn main() -> eframe::Result<()> {
    env_logger::init();

    let rt = tokio::runtime::Runtime::new().expect("Falha ao inicializar o runtime Tokio");
    let handle = rt.handle().clone();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1020.0, 700.0])
            .with_min_inner_size([840.0, 580.0])
            .with_title("AMP Control - Cross Platform Environment"),
        ..Default::default()
    };

    eframe::run_native(
        "AMP Control",
        native_options,
        Box::new(move |cc| Ok(Box::new(AmpApp::new(cc, handle)))),
    )
}
