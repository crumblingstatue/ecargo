mod app;
mod project;
mod style;
mod ui;

use {app::App, eframe::NativeOptions};

fn main() {
    eframe::run_native(
        "ecargo",
        NativeOptions::default(),
        Box::new(|cc| {
            Box::new({
                let mut app = App::new(&cc.egui_ctx);
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
