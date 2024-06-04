use {
    self::widgets::{badge, DepkindBadge, VersionBadge},
    crate::{
        app::App,
        config::Config,
        project::{dep_matches_pkg, Pkg, PkgKey, PkgSlotMap, Project},
        style::{Colors, Style},
    },
    eframe::egui::{self, Align2},
    egui_commonmark::CommonMarkCache,
    egui_modal::Modal,
    tab::Tab,
};

mod tab;
mod widgets;

pub struct Gui {
    pub modal: Modal,
    /// Primarily viewed package (i.e. main view)
    pub primary_pkg: Option<PkgKey>,
    /// Secondarily viewer package (i.e. sidebar)
    pub secondary_pkg: Option<PkgKey>,
    pub settings_window: SettingsWindow,
    pub style: Style,
    pub tab: Tab,
    pub right_panel_left: f32,
    pub pkg_list_filter: String,
    md: MdContent,
    pub cm_cache: CommonMarkCache,
    pub show_sidebar: bool,
}

/// Markdown content
#[derive(Default)]
struct MdContent {
    md: String,
    kind: MdContentKind,
    key: PkgKey,
}

impl MdContent {
    fn new(md: String, kind: MdContentKind, key: PkgKey) -> Self {
        Self { md, kind, key }
    }
}

#[derive(Default, Clone, Copy)]
enum MdContentKind {
    #[default]
    Readme,
    Changelog,
    CargoToml,
}

#[derive(Default)]
pub struct SettingsWindow {
    pub open: bool,
}

impl SettingsWindow {
    fn ui(&mut self, ctx: &egui::Context, style: &mut Style, cfg: &mut Config) {
        egui::Window::new("Settings")
            .open(&mut self.open)
            .anchor(Align2::RIGHT_TOP, egui::vec2(0.0, 0.0))
            .resizable(false)
            .collapsible(false)
            .show(ctx, |ui| {
                egui::Grid::new("settings_grid").show(ui, |ui| {
                    ui.label("Style");
                    egui::ComboBox::new("style_combo", "").selected_text(style.name).show_ui(
                        ui,
                        |ui| {
                            for (name, f) in crate::style::STYLE_LIST {
                                if ui.selectable_label(style.name == *name, *name).clicked() {
                                    *style = f();
                                    crate::style::apply_style(ctx, style.clone());
                                    cfg.style_name = name.to_string();
                                }
                            }
                        },
                    );
                    ui.end_row();
                    ui.label("Terminal")
                        .on_hover_text("The terminal to use for \"Open in terminal\" action");
                    ui.add(
                        egui::TextEdit::singleline(&mut cfg.terminal_app)
                            .text_color(style.colors.text_edit_text),
                    );
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
            secondary_pkg: None,
            primary_pkg: None,
            settings_window: SettingsWindow::default(),
            style,
            tab: Tab::default(),
            right_panel_left: egui_ctx.available_rect().width(),
            pkg_list_filter: String::new(),
            md: MdContent::default(),
            cm_cache: CommonMarkCache::default(),
            show_sidebar: true,
        }
    }
}

pub fn do_ui(app: &mut App, ctx: &egui::Context) {
    app.gui.modal.show_dialog();
    match &app.project {
        Some(proj) => project_ui(proj, ctx, &mut app.gui, &mut app.config),
        None => {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.label("No project loaded");
            });
        }
    }
}

pub fn project_ui(project: &Project, ctx: &egui::Context, gui: &mut Gui, cfg: &mut Config) {
    egui::CentralPanel::default().show(ctx, |ui| match gui.tab {
        Tab::ViewSingle => tab::view_single_ui(ui, gui, project, cfg),
        Tab::PackageList => tab::package_list_ui(project, ui, gui),
        Tab::Markdown => tab::markdown_ui(ui, gui, project),
    });
    if let (Some(key), true) = (gui.secondary_pkg, gui.show_sidebar) {
        let re = egui::SidePanel::right("right_panel")
            .max_width(ctx.available_rect().width() / 2.5)
            .show(ctx, |ui| {
                let pkg = &project.packages[key];
                pkg_info_ui(ui, pkg, &project.packages, gui, cfg);
            });
        gui.right_panel_left = re.response.rect.left();
    } else {
        gui.right_panel_left = ctx.available_rect().width();
    }
    gui.settings_window.ui(ctx, &mut gui.style, cfg);
}

fn markdown_tab_label(kind: MdContentKind, pkgname: &str) -> String {
    let tabkind = match kind {
        MdContentKind::Readme => "Readme",
        MdContentKind::Changelog => "Changelog",
        MdContentKind::CargoToml => "Cargo.toml",
    };
    format!("{tabkind} - {pkgname}")
}

fn central_top_bar(ui: &mut egui::Ui, gui: &mut Gui, project: &Project) {
    ui.horizontal(|ui| {
        ui.set_width(gui.right_panel_left - 16.0);
        let active_pkg = gui.primary_pkg.map(|key| &project.packages[key]);
        let tab_str_buf;
        for (tab, tabname) in [
            (
                Tab::ViewSingle,
                active_pkg.map(|pkg| pkg.cm_pkg.name.as_str()).unwrap_or("Single view"),
            ),
            (Tab::PackageList, "Packages"),
            (Tab::Markdown, {
                if gui.md.md.is_empty() {
                    "Markdown"
                } else {
                    tab_str_buf = markdown_tab_label(
                        gui.md.kind,
                        project
                            .packages
                            .get(gui.md.key)
                            .map(|pkg| pkg.cm_pkg.name.as_str())
                            .unwrap_or("Unknown"),
                    );
                    &tab_str_buf
                }
            }),
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
            let [icon, tooltip] = if gui.show_sidebar {
                ["⏵", "Hide sidebar"]
            } else {
                ["⏴", "Show sidebar"]
            };
            if ui.button(icon).on_hover_text(tooltip).clicked() {
                gui.show_sidebar ^= true;
            }
            if ui.button(gui.style.icons.settings).on_hover_text("Settings").clicked() {
                gui.settings_window.open ^= true;
            }
            match project.root {
                Some(root) => {
                    let pkg = &project.packages[root];
                    if ui.link(format!("go to root ({})", pkg.cm_pkg.name)).clicked() {
                        gui.primary_pkg = Some(pkg.key);
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
    if let Some(source) = &dep.source {
        if let Some(suffix) = source.strip_prefix("git+") {
            badge(ui, "git", egui::Color32::DARK_GREEN, egui::Color32::YELLOW)
                .on_hover_text(suffix);
        }
    }
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

fn pkg_info_ui(ui: &mut egui::Ui, pkg: &Pkg, packages: &PkgSlotMap, gui: &mut Gui, cfg: &Config) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(&pkg.cm_pkg.name)
                .heading()
                .color(gui.style.colors.highlighted_text),
        );
        if gui.primary_pkg != Some(pkg.key)
            && ui.button("👁").on_hover_text("Open in main view").clicked()
        {
            gui.primary_pkg = Some(pkg.key);
            gui.secondary_pkg = None;
            gui.tab = Tab::ViewSingle;
        }
        if ui.button("🖹").on_hover_text("View Cargo.toml.orig").clicked() {
            match std::fs::read_to_string(pkg.manifest_dir.join("Cargo.toml.orig")) {
                Ok(data) => {
                    gui.md = MdContent::new(
                        format!("```toml\n{data}\n```"),
                        MdContentKind::CargoToml,
                        pkg.key,
                    );
                    gui.tab = Tab::Markdown;
                }
                Err(e) => {
                    gui.modal
                        .dialog()
                        .with_title("Error")
                        .with_icon(egui_modal::Icon::Error)
                        .with_body(format!("Could not open Cargo.toml.orig: {e}"))
                        .open();
                }
            }
        }
        if ui
            .button("🗁")
            .on_hover_text(format!("{}\nOpen directory", pkg.manifest_dir.as_str()))
            .clicked()
        {
            let _ = open::that(&pkg.manifest_dir);
        }
        if ui
            .add_enabled(!cfg.terminal_app.is_empty(), egui::Button::new("🖳"))
            .on_hover_text("Open in terminal")
            .on_disabled_hover_text("No terminal configured")
            .clicked()
        {
            let result = std::process::Command::new(&cfg.terminal_app)
                .current_dir(&pkg.manifest_dir)
                .spawn();
            if let Err(e) = result {
                gui.modal
                    .dialog()
                    .with_title("Error")
                    .with_icon(egui_modal::Icon::Error)
                    .with_body(format!("Error spawning terminal {e}"))
                    .open();
            }
        }
    });
    if let Some(desc) = &pkg.cm_pkg.description {
        ui.label(desc);
    }
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("version");
        ui.label(
            egui::RichText::new(pkg.cm_pkg.version.to_string())
                .color(gui.style.colors.highlighted_text),
        );
    });
    if !pkg.cm_pkg.keywords.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Keywords");
            for kw in &pkg.cm_pkg.keywords {
                badge(
                    ui,
                    kw,
                    gui.style.colors.active_weak_bg_fill,
                    gui.style.colors.highlighted_text,
                );
            }
        });
    }
    if pkg.cm_pkg.authors.len() == 1 {
        ui.label(format!("Author: {}", pkg.cm_pkg.authors.first().unwrap()));
    } else if !pkg.cm_pkg.authors.is_empty() {
        cheader("Authors", &gui.style).show(ui, |ui| {
            for author in &pkg.cm_pkg.authors {
                ui.label(author);
            }
        });
    }
    if pkg.cm_pkg.source.as_ref().is_some_and(|src| src.is_crates_io()) {
        ui.horizontal(|ui| {
            ui.label("crates.io");
            ui.hyperlink(format!("https://crates.io/crates/{}", &pkg.cm_pkg.name));
        });
        ui.horizontal(|ui| {
            ui.label("docs.rs");
            ui.hyperlink(format!(
                "https://docs.rs/{}/{}",
                pkg.cm_pkg.name, pkg.cm_pkg.version
            ));
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
    if let Some(path) = &pkg.readme_path {
        ui.horizontal(|ui| {
            if ui.link("Readme").clicked() {
                gui.md = MdContent::new(
                    std::fs::read_to_string(path).unwrap(),
                    MdContentKind::Readme,
                    pkg.key,
                );
                gui.tab = Tab::Markdown;
            }
        });
    }
    if let Some(path) = &pkg.changelog_path {
        ui.horizontal(|ui| {
            if ui.link("Changelog").clicked() {
                gui.md = MdContent::new(
                    std::fs::read_to_string(path).unwrap(),
                    MdContentKind::Changelog,
                    pkg.key,
                );
                gui.tab = Tab::Markdown;
            }
        });
    }
    ui.horizontal(|ui| {
        ui.label("License");
        match &pkg.cm_pkg.license {
            Some(license) => ui.label(license),
            None => ui.label("Unknown"),
        };
    });
    ui.separator();
    egui::ScrollArea::vertical().show(ui, |ui| {
        pkg_info_collapsibles_ui(pkg, gui, ui, packages);
    });
}

fn pkg_info_collapsibles_ui(pkg: &Pkg, gui: &mut Gui, ui: &mut egui::Ui, packages: &PkgSlotMap) {
    if !pkg.cm_pkg.features.is_empty() {
        cheader("Features", &gui.style).show(ui, |ui| {
            egui::Grid::new("feat_grid").striped(true).show(ui, |ui| {
                for (name, reqs) in &pkg.cm_pkg.features {
                    let enabled = pkg.enabled_features.contains(name);
                    if enabled {
                        ui.label("☑").on_hover_text("enabled");
                    } else {
                        ui.label("☐").on_hover_text("disabled");
                    }
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
    }
    if !pkg.dependents.is_empty() {
        cheader("Dependents", &gui.style).show(ui, |ui| {
            for link in &pkg.dependents {
                ui.horizontal(|ui| {
                    let dpkg = &packages[link.pkg_key];
                    let re = ui.selectable_label(
                        false,
                        egui::RichText::new(&dpkg.cm_pkg.name)
                            .color(gui.style.colors.highlighted_text),
                    );
                    if re.clicked() {
                        gui.secondary_pkg = Some(link.pkg_key);
                        gui.show_sidebar = true;
                    }
                    if re.double_clicked() {
                        gui.primary_pkg = Some(link.pkg_key);
                        gui.show_sidebar = false;
                    }
                    ui.add(VersionBadge::new(&dpkg.cm_pkg.version, &gui.style));
                    ui.add(DepkindBadge::new(link.kind, &gui.style));
                    if let Some(platform) = &link.target {
                        ui.label(platform.to_string());
                    }
                });
            }
        });
    }
    if !pkg.dependencies.is_empty() {
        cheader("Dependencies", &gui.style).show(ui, |ui| {
            egui::Grid::new("deps_grid").striped(true).show(ui, |ui| {
                for dep in pkg.cm_pkg.dependencies.iter() {
                    ui.add(DepkindBadge::new(dep.kind, &gui.style));
                    if let Some(pkg) = packages.values().find(|pkg| dep_matches_pkg(dep, pkg)) {
                        ui.scope(|ui| {
                            let re = ui.selectable_label(
                                gui.secondary_pkg == Some(pkg.key),
                                egui::RichText::new(&pkg.cm_pkg.name)
                                    .color(gui.style.colors.highlighted_text)
                                    .strong(),
                            );
                            re.context_menu(|ui| {
                                if ui
                                    .button("Focus")
                                    .on_hover_text(
                                        "Focus in main view.\nDouble clicking has same effect.",
                                    )
                                    .clicked()
                                {
                                    gui.primary_pkg = Some(pkg.key);
                                    ui.close_menu();
                                }
                            });
                            if re.clicked() {
                                gui.secondary_pkg = Some(pkg.key);
                                gui.show_sidebar = true;
                            }
                            if re.double_clicked() {
                                gui.primary_pkg = Some(pkg.key);
                                gui.show_sidebar = false;
                            }
                            ui.add(VersionBadge::new(&pkg.cm_pkg.version, &gui.style));
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
