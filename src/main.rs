mod files;
mod viewer;

fn main() -> eframe::Result {
    let start = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    let opts = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1100.0, 760.0])
            .with_min_inner_size([420.0, 300.0])
            .with_title("Penguin Viewer")
            .with_app_id("penguin-viewer")
            .with_drag_and_drop(true)
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon.png"))
                    .expect("icon"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Penguin Viewer",
        opts,
        Box::new(move |_cc| Ok(Box::new(viewer::Viewer::new(start)))),
    )
}
