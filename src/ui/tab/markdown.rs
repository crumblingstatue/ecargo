use {
    crate::{
        project::Project,
        ui::{central_top_bar, Gui},
    },
    eframe::egui,
    egui_commonmark::CommonMarkViewer,
};

pub fn markdown_ui(ui: &mut egui::Ui, gui: &mut Gui, project: &Project) {
    central_top_bar(ui, gui, project);
    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.set_max_width(gui.right_panel_left - 14.0);
        if gui.style.name == "crates.io" {
            // Hack to make things more legible
            ui.style_mut().visuals = egui::Visuals::light();
        }
        CommonMarkViewer::new("md_view").show(ui, &mut gui.cm_cache, &gui.md.md);
    });
}
