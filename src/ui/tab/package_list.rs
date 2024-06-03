use {
    super::Tab,
    crate::{
        project::Project,
        ui::{central_top_bar, widgets::VersionBadge, Gui},
    },
    eframe::egui,
};

pub(crate) fn package_list_ui(project: &Project, ui: &mut egui::Ui, gui: &mut Gui) {
    central_top_bar(ui, gui, project);
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
                        gui.secondary_pkg == Some(key),
                        egui::RichText::new(&pkg.cm_pkg.name)
                            .color(gui.style.colors.highlighted_text),
                    );
                    if re.clicked() {
                        gui.secondary_pkg = Some(key);
                    }
                    if re.double_clicked() {
                        gui.primary_pkg = Some(key);
                        gui.secondary_pkg = None;
                        gui.tab = Tab::ViewSingle;
                    }
                    ui.add(VersionBadge::new(&pkg.cm_pkg.version, &gui.style));
                });
                if let Some(info) = &pkg.cm_pkg.description {
                    ui.label(info);
                }
                ui.end_row();
            }
        });
    });
}
