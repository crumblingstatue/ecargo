use {
    crate::{project::Project, ui::Gui},
    eframe::egui,
    std::path::Path,
};

pub struct App {
    pub project: Option<Project>,
    pub gui: Gui,
}

impl App {
    pub fn new(egui_ctx: &egui::Context) -> Self {
        App {
            project: None,
            gui: Gui::new(egui_ctx),
        }
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

impl eframe::App for App {
    fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::do_ui(self, ctx);
    }
}
