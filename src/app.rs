use {crate::project::Project, eframe::egui, egui_modal::Modal, std::path::Path};

pub struct App {
    project: Option<Project>,
    gui: Gui,
}

pub struct Gui {
    modal: Modal,
}

impl Gui {
    pub fn new(egui_ctx: &egui::Context) -> Self {
        Self {
            modal: Modal::new(egui_ctx, "modal_dialog"),
        }
    }
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
            Ok(proj) => self.project = Some(proj),
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
        self.gui.modal.show_dialog();
        egui::CentralPanel::default().show(ctx, |ui| match &self.project {
            Some(proj) => {
                crate::ui::project_ui(proj, ui);
            }
            None => {
                ui.label("No project opened");
            }
        });
    }
}
