use {
    crate::{
        config::Config,
        project::{Pkg, Project},
        ui::{central_top_bar, pkg_info_ui, Gui},
    },
    eframe::egui,
};

pub fn view_single_ui(ui: &mut egui::Ui, gui: &mut Gui, project: &Project, cfg: &Config) {
    if let Some(id) = gui.primary_pkg {
        let pkg = &project.packages[id];
        package_ui(project, pkg, ui, gui, cfg);
    } else {
        central_top_bar(ui, gui, project);
        ui.label("No primary package. Select one from the `Packages` tab.");
    }
}

fn package_ui(project: &Project, pkg: &Pkg, ui: &mut egui::Ui, gui: &mut Gui, cfg: &Config) {
    central_top_bar(ui, gui, project);
    pkg_info_ui(ui, pkg, &project.packages, gui, cfg);
}
