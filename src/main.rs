mod app;
mod config;
mod project;
mod ui;

use {
    anyhow::Context,
    app::App,
    clap::Parser,
    eframe::{
        egui::{self, ViewportBuilder},
        NativeOptions,
    },
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
    /// Don't resolve dependencies
    #[arg(long)]
    no_deps: bool,
    /// Show version information and exit
    #[arg(long)]
    version: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.version {
        println!("ecargo version {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let viewport = ViewportBuilder::default()
        .with_title("Trayracer")
        .with_inner_size(egui::vec2(1280.0, 720.0));

    eframe::run_native(
        "ecargo",
        NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| {
            Ok(Box::new({
                let mut app = App::new(&cc.egui_ctx)?;
                if let Some(path) = &args.manifest_path {
                    app.load_project_async(path.to_owned(), args);
                } else {
                    match std::env::current_dir() {
                        Ok(cwd) => app.load_project_async(cwd, args),
                        Err(e) => eprintln!("Could not determine cwd: {e}"),
                    }
                }
                app
            }))
        }),
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))
    .context("Failed to run native")
}
