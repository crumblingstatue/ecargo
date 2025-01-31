use {
    crate::{
        project::Project,
        ui::{central_top_bar, Gui},
    },
    eframe::egui,
    std::collections::hash_map::Entry,
};

fn compute_license_hashmap(project: &mut Project) {
    for (key, pkg) in &project.packages {
        let license = pkg.cm_pkg.license.clone().unwrap_or_else(|| "<no license>".into());
        match project.license_map.entry(license) {
            Entry::Occupied(mut occupied) => {
                occupied.get_mut().push(key);
            }
            Entry::Vacant(vacant) => {
                vacant.insert(vec![key]);
            }
        }
    }
}

pub(crate) fn licenses_ui(ui: &mut egui::Ui, gui: &mut Gui, project: &mut Project) {
    central_top_bar(ui, gui, project);
    if project.license_map.is_empty() {
        compute_license_hashmap(project);
    }
    egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
        ui.style_mut().visuals.widgets.inactive.fg_stroke =
            egui::Stroke::new(2.0, egui::Color32::BLACK);
        for (license, pkgs) in &project.license_map {
            ui.collapsing(format!("{license} ({})", pkgs.len()), |ui| {
                for pkg in pkgs {
                    let selected = gui.secondary_pkg == Some(*pkg);
                    let cm_pkg = &project.packages[*pkg].cm_pkg;
                    if ui
                        .selectable_label(selected, format!("{} {}", cm_pkg.name, cm_pkg.version))
                        .clicked()
                    {
                        gui.secondary_pkg = Some(*pkg);
                    }
                }
            });
        }
    });
}
