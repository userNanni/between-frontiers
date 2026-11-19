//! Between Frontiers game client.

fn main() -> eframe::Result {
    let paths = bf_client::paths::Paths::discover();
    // The window as the player left it: full screen, borderless or a size; the app applies it
    // again at the first frame (a size that no longer fits the monitor is the system's to cut).
    let display = bf_client::settings::Settings::load(&paths.settings_file()).display;
    let [w, h] = display.size.unwrap_or(bf_client::display::DEFAULT_SIZE);
    use bf_client::display::Mode;
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Between Frontiers")
            .with_inner_size([w as f32, h as f32])
            .with_maximized(display.size.is_none() || display.mode == Mode::Borderless)
            .with_decorations(display.mode != Mode::Borderless)
            .with_fullscreen(display.mode == Mode::Fullscreen)
            // The floor every screen is proven to fit (`tests/layout_matrix.rs`).
            .with_min_inner_size(bf_client::display::MIN_SIZE),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "between-frontiers",
        options,
        Box::new(move |cc| Ok(Box::new(bf_client::App::new(cc, paths)))),
    )
}
