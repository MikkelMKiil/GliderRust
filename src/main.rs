use eframe::NativeOptions;

use glider_rust::ui::GliderApp;

fn main() -> eframe::Result<()> {
    glider_rust::init_logging();

    let options = NativeOptions::default();
    eframe::run_native(
        "GliderRust",
        options,
        Box::new(|_cc| Ok(Box::new(GliderApp::default()))),
    )
}
