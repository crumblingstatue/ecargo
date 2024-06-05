use {
    crate::{config::Config, project::Project, ui::Gui},
    anyhow::Context,
    directories::ProjectDirs,
    eframe::egui,
    std::path::PathBuf,
};

pub struct App {
    pub project: Option<Project>,
    pub gui: Gui,
    pub dirs: ProjectDirs,
    pub config: Config,
    pub load: Option<LoadState>,
}

pub struct LoadState {
    pub(crate) recv: std::sync::mpsc::Receiver<anyhow::Result<Project>>,
    pub(crate) path: PathBuf,
}

impl App {
    pub fn new(egui_ctx: &egui::Context) -> anyhow::Result<Self> {
        let dirs = ProjectDirs::from("", "crumblingstatue", "ecargo")
            .context("Could not determine project dirs")?;
        let config = Config::load_or_default(dirs.config_dir());
        let style_name = config.style_name.clone();
        let mut app = App {
            project: None,
            gui: Gui::new(egui_ctx),
            dirs,
            config,
            load: None,
        };
        match crate::style::style_fun_by_name(&style_name) {
            Some(fun) => {
                let style = fun();
                crate::style::apply_style(egui_ctx, style.clone());
                app.gui.style = style;
            }
            None => eprintln!("No such style: {}", style_name),
        }
        Ok(app)
    }

    pub(crate) fn load_project_async(&mut self, path: PathBuf, args: crate::Args) {
        let (tx, rx) = std::sync::mpsc::channel();
        self.load = Some(LoadState {
            recv: rx,
            path: path.clone(),
        });
        std::thread::spawn(move || {
            tx.send(Project::load(&path, &args)).unwrap();
        });
    }
}

impl Drop for App {
    fn drop(&mut self) {
        if let Err(e) = self.config.save(self.dirs.config_dir()) {
            eprintln!("Failed to save config: {e}");
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::do_ui(self, ctx);
    }
}
