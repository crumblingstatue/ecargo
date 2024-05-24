mod app;
mod config;
mod project;
mod style;
mod ui;

use {
    app::App,
    clap::Parser,
    eframe::{egui, NativeOptions},
    std::path::PathBuf,
};

#[derive(clap::Parser)]
struct Args {
    /// Resolve without default features
    #[arg(long)]
    no_default_features: bool,
    /// Add features to the list of features to resolve
    #[arg(long)]
    features: Vec<String>,
    /// Use this manifest path instead of the current working directory
    manifest_path: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();
    eframe::run_native(
        "ecargo",
        NativeOptions::default(),
        Box::new(move |cc| {
            Box::new({
                egui_extras::install_image_loaders(&cc.egui_ctx);
                cc.egui_ctx
                    .send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(1280., 720.)));
                let mut app = App::new(&cc.egui_ctx).unwrap();
                if let Some(path) = &args.manifest_path {
                    app.load_project(path, &args);
                } else {
                    match std::env::current_dir() {
                        Ok(cwd) => app.load_project(&cwd, &args),
                        Err(e) => eprintln!("Could not determine cwd: {e}"),
                    }
                }
                app
            })
        }),
    )
    .unwrap();
}
