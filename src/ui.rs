use {
    crate::{
        app::App,
        project::{dep_matches_pkg, Pkg, PkgKey, PkgSlotMap, Project},
        style::{Colors, Style},
    },
    cargo_metadata::{camino::Utf8PathBuf, semver::Version, DependencyKind},
    eframe::egui::{self, Color32},
    egui_modal::Modal,
};

pub struct Gui {
    pub modal: Modal,
    pub sidebar_pkg: Option<PkgKey>,
    pub focused_package: Option<PkgKey>,
    pub settings_window: SettingsWindow,
    pub style: Style,
    pub tab: Tab,
    pub right_panel_left: f32,
    pub pkg_list_filter: String,
}

#[derive(Default, PartialEq)]
pub enum Tab {
    #[default]
    ViewSingle,
    PackageList,
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
            sidebar_pkg: None,
            focused_package: None,
            settings_window: SettingsWindow::default(),
            style,
            tab: Tab::default(),
            right_panel_left: egui_ctx.input(|inp| {
                inp.viewport()
                    .inner_rect
                    .map(|r| r.width())
                    .unwrap_or(1000.0)
            }),
            pkg_list_filter: String::new(),
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
        Some(id) => match gui.tab {
            Tab::ViewSingle => {
                let pkg = &project.packages[id];
                package_ui(project, pkg, &pkg.cm_pkg.manifest_path, ui, gui);
            }
            Tab::PackageList => {
                package_list_ui(project, ui, gui);
            }
        },
        None => {
            ui.heading(project.metadata.workspace_root.to_string());
        }
    });
    if let Some(key) = gui.sidebar_pkg {
        let re = egui::SidePanel::right("right_panel")
            .max_width(400.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("X").clicked() {
                            gui.sidebar_pkg = None;
                        }
                    });
                });
                let pkg = &project.packages[key];
                pkg_info_ui(
                    ui,
                    pkg,
                    &project.packages,
                    &gui.style,
                    &mut gui.focused_package,
                    &mut gui.sidebar_pkg,
                );
            });
        gui.right_panel_left = re.response.rect.left();
    } else {
        gui.right_panel_left = ctx.input(|inp| inp.viewport().inner_rect.unwrap().width());
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
    central_top_bar(ui, gui, Some(pkg), project);
    ui.label(src_path.to_string());
    pkg_info_ui(
        ui,
        pkg,
        &project.packages,
        &gui.style,
        &mut gui.focused_package,
        &mut gui.sidebar_pkg,
    );
    ui.add_space(16.0);
    ui.label("Dependencies");
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("deps_grid").striped(true).show(ui, |ui| {
            for dep in pkg.cm_pkg.dependencies.iter() {
                ui.add(DepkindBadge(dep.kind));
                if let Some(pkg) = project
                    .packages
                    .values()
                    .find(|pkg| dep_matches_pkg(dep, pkg))
                {
                    ui.scope(|ui| {
                        let re = ui.selectable_label(
                            gui.sidebar_pkg == Some(pkg.key),
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
                            gui.sidebar_pkg = Some(pkg.key);
                        }
                        if re.double_clicked() {
                            gui.focused_package = Some(pkg.key);
                            gui.sidebar_pkg = None;
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

fn central_top_bar(ui: &mut egui::Ui, gui: &mut Gui, active_pkg: Option<&Pkg>, project: &Project) {
    ui.horizontal(|ui| {
        ui.set_width(gui.right_panel_left - 16.0);
        for (tab, tabname) in [
            (
                Tab::ViewSingle,
                active_pkg
                    .map(|pkg| pkg.cm_pkg.name.as_str())
                    .unwrap_or("Single view"),
            ),
            (Tab::PackageList, "Packages"),
        ] {
            if ui
                .selectable_label(
                    gui.tab == tab,
                    egui::RichText::new(tabname).color(gui.style.colors.highlighted_text),
                )
                .clicked()
            {
                gui.tab = tab;
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button(gui.style.icons.settings).clicked() {
                gui.settings_window.open ^= true;
            }
            match project.root {
                Some(root) => {
                    let pkg = &project.packages[root];
                    if ui
                        .link(format!("go to root ({})", pkg.cm_pkg.name))
                        .clicked()
                    {
                        gui.focused_package = Some(pkg.key);
                        gui.tab = Tab::ViewSingle;
                    }
                }
                None => {
                    ui.add_enabled(false, egui::Link::new("root"));
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

fn pkg_info_ui(
    ui: &mut egui::Ui,
    pkg: &Pkg,
    packages: &PkgSlotMap,
    style: &crate::style::Style,
    focused_pkg: &mut Option<PkgKey>,
    sidebar_pkg: &mut Option<PkgKey>,
) {
    ui.label(
        egui::RichText::new(&pkg.cm_pkg.name)
            .heading()
            .color(style.colors.highlighted_text),
    );
    if let Some(desc) = &pkg.cm_pkg.description {
        ui.label(desc);
    }
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("version");
        ui.label(
            egui::RichText::new(pkg.cm_pkg.version.to_string())
                .color(style.colors.highlighted_text),
        );
    });
    if !pkg.cm_pkg.keywords.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Keywords");
            for kw in &pkg.cm_pkg.keywords {
                badge(
                    ui,
                    kw,
                    style.colors.active_weak_bg_fill,
                    style.colors.highlighted_text,
                );
            }
        });
    }
    if pkg.cm_pkg.authors.len() == 1 {
        ui.label(format!("Author: {}", pkg.cm_pkg.authors.first().unwrap()));
    } else if !pkg.cm_pkg.authors.is_empty() {
        cheader("Authors", style).show(ui, |ui| {
            for author in &pkg.cm_pkg.authors {
                ui.label(author);
            }
        });
    }
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
    ui.add_space(2.0);

    cheader("Features", style).show(ui, |ui| {
        egui::Grid::new("feat_grid").striped(true).show(ui, |ui| {
            for (name, reqs) in &pkg.cm_pkg.features {
                ui.label(name);
                ui.scope(|ui| {
                    for req in reqs {
                        ui.label(req);
                    }
                });
                ui.end_row();
            }
        });
    });
    if !pkg.dependents.is_empty() {
        cheader("Dependents", style).show(ui, |ui| {
            for link in &pkg.dependents {
                ui.horizontal(|ui| {
                    let dpkg = &packages[link.pkg_key];
                    let re = ui.button(&dpkg.cm_pkg.name);
                    if re.clicked() {
                        *sidebar_pkg = Some(link.pkg_key);
                    }
                    if re.double_clicked() {
                        *focused_pkg = Some(link.pkg_key);
                        *sidebar_pkg = None;
                    }
                    ui.add(VersionBadge(&dpkg.cm_pkg.version));
                    ui.add(DepkindBadge(link.kind));
                    if let Some(platform) = &link.target {
                        ui.label(platform.to_string());
                    }
                });
            }
        });
    }
}

fn cheader(label: &str, style: &crate::style::Style) -> egui::CollapsingHeader {
    let colors = style.colors;
    egui::CollapsingHeader::new(egui::RichText::new(label).color(style.colors.highlighted_text))
        .icon(move |ui, openness, re| header_icon(ui, openness, re, colors))
}

// Stolen code from egui, because I need to specify the right color for the icon
fn header_icon(ui: &mut egui::Ui, openness: f32, response: &egui::Response, colors: Colors) {
    let visuals = ui.style().interact(response);

    let rect = response.rect;

    // Draw a pointy triangle arrow:
    let rect = egui::Rect::from_center_size(
        rect.center(),
        egui::vec2(rect.width(), rect.height()) * 0.75,
    );
    let rect = rect.expand(visuals.expansion);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    use std::f32::consts::TAU;
    let rotation =
        egui::emath::Rot2::from_angle(egui::remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }

    ui.painter().add(egui::Shape::convex_polygon(
        points,
        colors.highlighted_text,
        egui::Stroke::NONE,
    ));
}

fn package_list_ui(project: &Project, ui: &mut egui::Ui, gui: &mut Gui) {
    central_top_bar(
        ui,
        gui,
        gui.focused_package.map(|key| &project.packages[key]),
        project,
    );
    let mut filtered: Vec<_> = project.packages.keys().collect();
    ui.horizontal(|ui| {
        ui.add(
            egui::TextEdit::singleline(&mut gui.pkg_list_filter)
                .text_color(gui.style.colors.text_edit_text)
                .hint_text("Filter"),
        );
        filtered.retain(|key| {
            let pkg = &project.packages[*key];
            pkg.cm_pkg.name.contains(&gui.pkg_list_filter)
                || pkg
                    .cm_pkg
                    .description
                    .as_ref()
                    .is_some_and(|desc| desc.to_ascii_lowercase().contains(&gui.pkg_list_filter))
                || pkg
                    .cm_pkg
                    .keywords
                    .iter()
                    .any(|kw| kw.contains(&gui.pkg_list_filter))
        });
        ui.label(format!(
            "{}/{} packages",
            filtered.len(),
            project.packages.len()
        ));
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("pkg_list_grid").show(ui, |ui| {
            for key in filtered {
                let pkg = &project.packages[key];
                ui.scope(|ui| {
                    let re = ui.selectable_label(
                        gui.sidebar_pkg == Some(key),
                        egui::RichText::new(&pkg.cm_pkg.name)
                            .color(gui.style.colors.highlighted_text),
                    );
                    if re.clicked() {
                        gui.sidebar_pkg = Some(key);
                    }
                    if re.double_clicked() {
                        gui.focused_package = Some(key);
                        gui.sidebar_pkg = None;
                        gui.tab = Tab::ViewSingle;
                    }
                    ui.add(VersionBadge(&pkg.cm_pkg.version));
                });
                if let Some(info) = &pkg.cm_pkg.description {
                    ui.label(info);
                }
                ui.end_row();
            }
        });
    });
}
