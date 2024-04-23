use {
    crate::project::Project,
    cargo_metadata::{camino::Utf8PathBuf, DependencyKind, Package},
    eframe::egui,
};

pub fn project_ui(project: &Project, ui: &mut egui::Ui) {
    match project.metadata.root_package() {
        Some(pkg) => {
            package_ui(pkg, &pkg.manifest_path, ui);
        }
        None => {
            ui.heading(project.metadata.workspace_root.to_string());
        }
    }
}

fn package_ui(pkg: &Package, src_path: &Utf8PathBuf, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading(&pkg.name);
        ui.add_space(8.0);
        ui.label(src_path.to_string())
    });
    if let Some(desc) = &pkg.description {
        ui.label(desc);
    }
    ui.add_space(16.0);
    ui.label("Dependencies");
    for dep in pkg
        .dependencies
        .iter()
        .filter(|dep| dep.kind == DependencyKind::Normal)
    {
        ui.label(
            egui::RichText::new(&dep.name)
                .color(egui::Color32::WHITE)
                .strong(),
        );
    }
}
