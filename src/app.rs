use {
    crate::{config::Config, project::Project, ui::Gui},
    anyhow::Context,
    directories::ProjectDirs,
    eframe::egui,
    std::path::Path,
};

pub struct App {
    pub project: Option<Project>,
    pub gui: Gui,
    pub dirs: ProjectDirs,
    pub config: Config,
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

    pub(crate) fn load_project(&mut self, path: &Path) {
        match Project::load(path) {
            Ok(proj) => {
                self.gui.focused_package = proj.root;
                self.project = Some(proj);
            }
            Err(e) => {
                self.gui
                    .modal
                    .dialog()
                    .with_title("Error loading project")
                    .with_icon(egui_modal::Icon::Error)
                    .with_body(e)
                    .open();
            }
        }
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
