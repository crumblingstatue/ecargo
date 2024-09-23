use {
    crate::{config::Config, project::Project, ui::style, ui::Gui},
    anyhow::Context,
    directories::ProjectDirs,
    eframe::egui,
    std::{path::PathBuf, sync::mpsc},
};

pub struct App {
    pub project: Option<Project>,
    pub gui: Gui,
    pub dirs: ProjectDirs,
    pub config: Config,
    pub load: Option<LoadState>,
}

pub enum LoadStage {
    MetadataQuery,
    Finished(Project),
    Error(anyhow::Error),
    PkgInfoCollect,
    Resolve,
    GenDepGraph,
}

pub type LoadRecv = mpsc::Receiver<LoadStage>;
pub type LoadSend = mpsc::Sender<LoadStage>;

pub struct LoadState {
    pub(crate) recv: LoadRecv,
    pub(crate) path: PathBuf,
    /// User-facing load status message
    pub(crate) msg: String,
}

impl App {
    pub fn new(egui_ctx: &egui::Context) -> anyhow::Result<Self> {
        egui_extras::install_image_loaders(egui_ctx);
        let dirs = ProjectDirs::from("", "crumblingstatue", "ecargo")
            .context("Could not determine project dirs")?;
        let config = Config::load_or_default(dirs.config_dir());
        let style_name = config.style_name.clone();
        let mut app = Self {
            project: None,
            gui: Gui::new(egui_ctx),
            dirs,
            config,
            load: None,
        };
        match style::style_fun_by_name(&style_name) {
            Some(fun) => {
                let style = fun();
                style::apply_style(egui_ctx, style.clone());
                app.gui.style = style;
            }
            None => eprintln!("No such style: {style_name}"),
        }
        Ok(app)
    }

    pub(crate) fn load_project_async(&mut self, path: PathBuf, args: crate::Args) {
        let (tx, rx) = mpsc::channel();
        self.load = Some(LoadState {
            recv: rx,
            path: path.clone(),
            msg: "Preparing...".into(),
        });
        std::thread::spawn(move || {
            if let Err(e) = Project::load(&path, &args, &tx) {
                tx.send(LoadStage::Error(e)).expect("Could not load project");
            }
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
