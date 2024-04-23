use {
    crate::{
        app::App,
        project::{Pkg, PkgKey, Project},
    },
    cargo_metadata::{camino::Utf8PathBuf, DependencyKind},
    eframe::egui,
    egui_modal::Modal,
};

pub struct Gui {
    pub modal: Modal,
    pub selected_dep: Option<PkgKey>,
    pub focused_package: Option<PkgKey>,
}

impl Gui {
    pub fn new(egui_ctx: &egui::Context) -> Self {
        Self {
            modal: Modal::new(egui_ctx, "modal_dialog"),
            selected_dep: None,
            focused_package: None,
        }
    }
}

pub fn do_ui(app: &mut App, ctx: &egui::Context) {
    app.gui.modal.show_dialog();
    match &app.project {
        Some(proj) => project_ui(proj, ctx, &mut app.gui),
        None => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("No project loaded");
            });
        }
    }
}

pub fn project_ui(project: &Project, ctx: &egui::Context, gui: &mut Gui) {
    egui::CentralPanel::default().show(ctx, |ui| match gui.focused_package {
        Some(id) => {
            let pkg = &project.packages[id];
            package_ui(project, pkg, &pkg.cm_pkg.manifest_path, ui, gui);
        }
        None => {
            ui.heading(project.metadata.workspace_root.to_string());
        }
    });
    if let Some(key) = gui.selected_dep {
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            let pkg = &project.packages[key];
            pkg_info_ui(ui, pkg);
        });
    }
}

fn package_ui(
    project: &Project,
    pkg: &Pkg,
    src_path: &Utf8PathBuf,
    ui: &mut egui::Ui,
    gui: &mut Gui,
) {
    ui.label(src_path.to_string());
    pkg_info_ui(ui, pkg);
    ui.add_space(16.0);
    ui.label("Dependencies");
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("deps_grid").show(ui, |ui| {
            for dep in pkg
                .cm_pkg
                .dependencies
                .iter()
                .filter(|dep| dep.kind == DependencyKind::Normal)
            {
                if let Some(pkg) = project
                    .packages
                    .values()
                    .find(|pkg| pkg.cm_pkg.name == dep.name && dep.req.matches(&pkg.cm_pkg.version))
                {
                    if ui
                        .selectable_label(
                            gui.selected_dep == Some(pkg.key),
                            egui::RichText::new(format!("{} v{}", dep.name, dep.req))
                                .color(egui::Color32::WHITE)
                                .strong(),
                        )
                        .clicked()
                    {
                        gui.selected_dep = Some(pkg.key);
                    }
                    if let Some(info) = &pkg.cm_pkg.description {
                        ui.label(info);
                    }
                    ui.end_row();
                } else {
                    ui.label(format!("Unresolved dependency: {}", dep.name));
                }
            }
        });
    });
}

fn pkg_info_ui(ui: &mut egui::Ui, pkg: &Pkg) {
    ui.label(
        egui::RichText::new(&pkg.cm_pkg.name)
            .heading()
            .color(egui::Color32::WHITE),
    );
    if let Some(desc) = &pkg.cm_pkg.description {
        ui.label(desc);
    }
    ui.horizontal(|ui| {
        ui.label("version");
        ui.label(egui::RichText::new(pkg.cm_pkg.version.to_string()).color(egui::Color32::WHITE));
    });
    if let Some(info) = &pkg.cm_pkg.homepage {
        ui.horizontal(|ui| {
            ui.label("Homepage");
            ui.hyperlink(info);
        });
    }
    if let Some(info) = &pkg.cm_pkg.repository {
        ui.horizontal(|ui| {
            ui.label("Repository");
            ui.hyperlink(info);
        });
    }
}
