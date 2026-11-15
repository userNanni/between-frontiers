//! Between Frontiers game client.

fn main() -> eframe::Result {
    let paths = bf_client::paths::Paths::discover();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Between Frontiers")
            .with_inner_size([1600.0, 900.0])
            .with_min_inner_size([1100.0, 680.0]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "between-frontiers",
        options,
        Box::new(move |cc| Ok(Box::new(bf_client::App::new(cc, paths)))),
    )
}
