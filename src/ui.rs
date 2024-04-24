use {
    crate::{
        app::App,
        project::{Pkg, PkgKey, Project},
        style::Style,
    },
    cargo_metadata::{camino::Utf8PathBuf, semver::Version, DependencyKind},
    eframe::egui::{self, Color32},
    egui_modal::Modal,
};

pub struct Gui {
    pub modal: Modal,
    pub selected_dep: Option<PkgKey>,
    pub focused_package: Option<PkgKey>,
    pub settings_window: SettingsWindow,
    pub style: Style,
}

#[derive(Default)]
pub struct SettingsWindow {
    pub open: bool,
}

impl SettingsWindow {
    fn ui(&mut self, ctx: &egui::Context, style: &mut Style) {
        egui::Window::new("Settings")
            .open(&mut self.open)
            .show(ctx, |ui| {
                egui::ComboBox::new("style_combo", "Style")
                    .selected_text(style.name)
                    .show_ui(ui, |ui| {
                        for (name, f) in crate::style::STYLE_LIST {
                            if ui.selectable_label(style.name == *name, *name).clicked() {
                                *style = f();
                                crate::style::apply_style(ctx, style.clone());
                            }
                        }
                    });
            });
    }
}

impl Gui {
    pub fn new(egui_ctx: &egui::Context) -> Self {
        let style = crate::style::crates_io();
        crate::style::apply_style(egui_ctx, style.clone());
        Self {
            modal: Modal::new(egui_ctx, "modal_dialog"),
            selected_dep: None,
            focused_package: None,
            settings_window: SettingsWindow::default(),
            style,
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
            pkg_info_ui(ui, pkg, &gui.style);
        });
    }
    gui.settings_window.ui(ctx, &mut gui.style);
}

struct DepkindBadge(DependencyKind);

impl egui::Widget for DepkindBadge {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (text, bg_color) = match self.0 {
            DependencyKind::Normal => ("normal", egui::Color32::from_rgb(91, 52, 197)),
            DependencyKind::Development => ("dev", egui::Color32::from_rgb(32, 60, 18)),
            DependencyKind::Build => ("build", egui::Color32::from_rgb(78, 40, 25)),
            DependencyKind::Unknown => ("unknown", egui::Color32::from_rgb(115, 115, 115)),
        };
        badge(ui, text, bg_color, Color32::YELLOW)
    }
}

fn badge(ui: &mut egui::Ui, text: &str, bg_color: Color32, text_color: Color32) -> egui::Response {
    let label = egui::Label::new(text);
    let (pos, galley, re) = label.layout_in_ui(ui);
    let painter = ui.painter();
    let rect = re.rect.expand(2.0);
    painter.rect_filled(rect, 2.0, bg_color);
    painter.galley(pos, galley, text_color);
    re.with_new_rect(rect)
}

struct VersionBadge<'a>(&'a Version);

impl<'a> egui::Widget for VersionBadge<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        badge(
            ui,
            &self.0.to_string(),
            Color32::from_rgb(69, 11, 86),
            Color32::WHITE,
        )
    }
}

fn package_ui(
    project: &Project,
    pkg: &Pkg,
    src_path: &Utf8PathBuf,
    ui: &mut egui::Ui,
    gui: &mut Gui,
) {
    ui.horizontal(|ui| {
        if ui.button(gui.style.icons.settings).clicked() {
            gui.settings_window.open ^= true;
        }
        match project.root {
            Some(root) => {
                let pkg = &project.packages[root];
                if ui.link("root").clicked() {
                    gui.focused_package = Some(pkg.key);
                }
            }
            None => {
                ui.add_enabled(false, egui::Link::new("root"));
            }
        }
    });
    ui.label(src_path.to_string());
    pkg_info_ui(ui, pkg, &gui.style);
    ui.add_space(16.0);
    ui.label("Dependencies");
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("deps_grid").striped(true).show(ui, |ui| {
            for dep in pkg.cm_pkg.dependencies.iter() {
                ui.add(DepkindBadge(dep.kind));
                if let Some(pkg) = project
                    .packages
                    .values()
                    .find(|pkg| pkg.cm_pkg.name == dep.name && dep.req.matches(&pkg.cm_pkg.version))
                {
                    ui.scope(|ui| {
                        let re = ui.selectable_label(
                            gui.selected_dep == Some(pkg.key),
                            egui::RichText::new(&pkg.cm_pkg.name)
                                .color(gui.style.colors.highlighted_text)
                                .strong(),
                        );
                        re.context_menu(|ui| {
                            if ui.button("Set as focused package").clicked() {
                                gui.focused_package = Some(pkg.key);
                                ui.close_menu();
                            }
                        });
                        if re.clicked() {
                            gui.selected_dep = Some(pkg.key);
                        }
                        ui.add(VersionBadge(&pkg.cm_pkg.version));
                        additional_dep_info_ui(dep, ui);
                    });
                    if let Some(info) = &pkg.cm_pkg.description {
                        ui.label(info);
                    }
                    ui.end_row();
                } else {
                    ui.scope(|ui| {
                        ui.label(format!("{} {}", dep.name, dep.req));
                        additional_dep_info_ui(dep, ui);
                    });
                    ui.label(egui::RichText::new("Unresolved").italics())
                        .on_hover_text("Couldn't find a package for this dependency.");
                    ui.end_row();
                }
            }
        });
    });
}

fn additional_dep_info_ui(dep: &cargo_metadata::Dependency, ui: &mut egui::Ui) {
    if let Some(target) = &dep.target {
        ui.label(target.to_string());
    }
    if dep.optional {
        badge(
            ui,
            "optional",
            egui::Color32::DARK_GREEN,
            egui::Color32::LIGHT_GREEN,
        );
    }
}

fn pkg_info_ui(ui: &mut egui::Ui, pkg: &Pkg, style: &crate::style::Style) {
    ui.label(
        egui::RichText::new(&pkg.cm_pkg.name)
            .heading()
            .color(style.colors.highlighted_text),
    );
    if let Some(desc) = &pkg.cm_pkg.description {
        ui.label(desc);
    }
    ui.horizontal(|ui| {
        ui.label("version");
        ui.label(
            egui::RichText::new(pkg.cm_pkg.version.to_string())
                .color(style.colors.highlighted_text),
        );
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
    if let Some(info) = &pkg.cm_pkg.documentation {
        ui.horizontal(|ui| {
            ui.label("Docs link");
            ui.hyperlink(info);
        });
    }
    ui.horizontal(|ui| {
        ui.label("docs.rs");
        ui.hyperlink(format!(
            "https://docs.rs/{}/{}",
            pkg.cm_pkg.name, pkg.cm_pkg.version
        ));
    });
    ui.horizontal(|ui| {
        ui.label("License");
        match &pkg.cm_pkg.license {
            Some(license) => ui.label(license),
            None => ui.label("Unknown"),
        };
    });
}
