use std::sync::Arc;

use eframe::{egui_wgpu, wgpu, NativeOptions, Renderer};

use glider_rust::ui::GliderApp;

fn main() -> eframe::Result<()> {
    glider_rust::init_logging();

    let options = build_native_options();

    tracing::info!("Starting GliderRust — wgpu renderer");

    eframe::run_native(
        "GliderRust",
        options,
        Box::new(|cc| Ok(Box::new(GliderApp::from_creation_context(cc)))),
    )
}

fn build_native_options() -> NativeOptions {
    let mut options = NativeOptions::default();
    options.renderer = Renderer::Wgpu;

    // Fixed 540 × 960 (9:16) logical-pixel window — scales automatically on high-DPI displays
    options.viewport = egui::ViewportBuilder::default()
        .with_title("GliderRust")
        .with_inner_size([540.0, 960.0])
        .with_min_inner_size([540.0, 960.0])
        .with_resizable(false);

    let mut wgpu_options = egui_wgpu::WgpuConfiguration::default();
    if let egui_wgpu::WgpuSetup::CreateNew(create_new) = &mut wgpu_options.wgpu_setup {
        create_new.instance_descriptor.backends =
            wgpu::Backends::VULKAN | wgpu::Backends::DX12;
        create_new.power_preference = wgpu::PowerPreference::HighPerformance;
        create_new.native_adapter_selector = Some(Arc::new(|adapters, _surface| {
            adapters
                .iter()
                .find(|a| a.get_info().backend == wgpu::Backend::Vulkan)
                .cloned()
                .or_else(|| adapters.first().cloned())
                .ok_or_else(|| "No compatible graphics adapter".to_string())
        }));
    }
    options.wgpu_options = wgpu_options;

    options
}
