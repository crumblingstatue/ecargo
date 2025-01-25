use {
    super::Tab,
    crate::{
        project::Project,
        ui::{central_top_bar, widgets::VersionBadge, Gui, PkgFilter},
    },
    eframe::egui,
};

pub(crate) fn package_list_ui(project: &Project, ui: &mut egui::Ui, gui: &mut Gui) {
    central_top_bar(ui, gui, project);
    let mut filtered: Vec<_> = project.packages.keys().collect();
    ui.horizontal(|ui| {
        if ui
            .add(
                egui::TextEdit::singleline(&mut gui.pkg_list_filter_string)
                    .text_color(gui.style.colors.text_edit_text)
                    .hint_text("Filter"),
            )
            .changed()
        {
            gui.pkg_list_compiled_filter = Some(PkgFilter::from_str(&gui.pkg_list_filter_string));
        }
        filtered.retain(|key| {
            let pkg = &project.packages[*key];
            match &gui.pkg_list_compiled_filter {
                Some(filt) => filt.matches(pkg),
                None => true,
            }
        });
        ui.label(format!(
            "{}/{} packages",
            filtered.len(),
            project.packages.len()
        ));
    });
    ui.separator();
    egui::ScrollArea::vertical().auto_shrink(false).show_rows(
        ui,
        22.0,
        filtered.len(),
        |ui, range| {
            egui::Grid::new("pkg_list_grid").show(ui, |ui| {
                for &key in &filtered[range] {
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
                        if let Some(fst_line) = info.lines().next() {
                            ui.label(fst_line).on_hover_text(info);
                        }
                    }
                    ui.end_row();
                }
            });
        },
    );
}
