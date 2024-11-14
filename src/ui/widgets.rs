use {
    crate::style::Style,
    cargo_metadata::{semver::Version, DependencyKind},
    eframe::egui,
};

pub struct DepkindBadge<'s> {
    kind: DependencyKind,
    style: &'s Style,
}

impl egui::Widget for DepkindBadge<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (text, bg_color, text_color) = match self.kind {
            DependencyKind::Normal => (
                "normal",
                self.style.colors.inactive_weak_bg_fill,
                self.style.colors.inactive_fg_stroke,
            ),
            DependencyKind::Development => (
                "dev",
                egui::Color32::from_rgb(32, 60, 18),
                egui::Color32::YELLOW,
            ),
            DependencyKind::Build => (
                "build",
                egui::Color32::from_rgb(78, 40, 25),
                egui::Color32::YELLOW,
            ),
            DependencyKind::Unknown => (
                "unknown",
                egui::Color32::from_rgb(115, 115, 115),
                egui::Color32::YELLOW,
            ),
        };
        badge(ui, text, bg_color, text_color)
    }
}

impl<'a> DepkindBadge<'a> {
    pub fn new(kind: DependencyKind, style: &'a crate::style::Style) -> Self {
        Self { kind, style }
    }
}

pub fn badge(
    ui: &mut egui::Ui,
    text: &str,
    bg_color: egui::Color32,
    text_color: egui::Color32,
) -> egui::Response {
    let label = egui::Label::new(egui::RichText::new(text).size(13.0));
    let (pos, galley, re) = label.layout_in_ui(ui);
    let painter = ui.painter();
    let rect = re.rect.expand(2.0);
    painter.rect_filled(rect, 2.0, bg_color);
    painter.galley(pos, galley, text_color);
    re.with_new_rect(rect)
}

pub struct VersionBadge<'a> {
    ver: &'a Version,
    bg_color: egui::Color32,
    text_color: egui::Color32,
}

impl<'a> VersionBadge<'a> {
    pub fn new(ver: &'a Version, style: &crate::style::Style) -> Self {
        Self {
            ver,
            bg_color: style.colors.inactive_weak_bg_fill,
            text_color: style.colors.inactive_fg_stroke,
        }
    }
}

impl egui::Widget for VersionBadge<'_> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        badge(ui, &self.ver.to_string(), self.bg_color, self.text_color)
    }
}
