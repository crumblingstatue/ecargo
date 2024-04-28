mod app;
mod config;
mod project;
mod style;
mod ui;

use {
    app::App,
    eframe::{egui, NativeOptions},
};

fn main() {
    eframe::run_native(
        "ecargo",
        NativeOptions::default(),
        Box::new(|cc| {
            Box::new({
                egui_extras::install_image_loaders(&cc.egui_ctx);
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1280., 720.)));
                let mut app = App::new(&cc.egui_ctx).unwrap();
                if let Some(arg) = std::env::args_os().nth(1) {
                    app.load_project(arg.as_ref());
                } else {
                    match std::env::current_dir() {
                        Ok(cwd) => app.load_project(&cwd),
                        Err(e) => eprintln!("Could not determine cwd: {e}"),
                    }
                }
                app
            })
        }),
    )
    .unwrap();
}
