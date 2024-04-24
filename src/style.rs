use eframe::egui::{self, Color32, FontData, FontDefinitions};

type StyleFn = fn() -> Style;

pub const STYLE_LIST: &[(&str, StyleFn); 3] = &[
    ("crates.io", crates_io),
    ("egui dark", egui_dark),
    ("egui light", egui_light),
];

#[derive(Clone)]
pub struct Font {
    pub name: String,
    pub data: FontData,
}

#[derive(Clone)]
pub struct Style {
    pub name: &'static str,
    pub font: Option<Font>,
    pub colors: Colors,
    pub icons: Icons,
}

#[derive(Clone, Copy)]
pub struct Colors {
    pub panel_fill: Color32,
    pub hyperlink_color: Color32,
    pub selected_bg_fill: Color32,
    pub hover_weak_bg_fill: Color32,
    pub active_weak_bg_fill: Color32,
    pub noninteractive_fg: Color32,
    pub highlighted_text: Color32,
    pub window_fill: Color32,
    pub inactive_weak_bg_fill: Color32,
    pub inactive_fg_stroke: Color32,
    pub open_weak_bg_fill: Color32,
    pub hover_fg_stroke: Color32,
    pub selection_stroke: Color32,
}

#[derive(Clone, Copy)]
pub struct Icons {
    pub settings: &'static str,
}

#[allow(dead_code)]
pub const DEBUG_COLOR: Color32 = Color32::from_rgb(255, 0, 255);

pub fn crates_io() -> Style {
    let font_data = egui::FontData::from_static(include_bytes!("../assets/FiraSans-Regular.ttf"));
    let font = Font {
        name: "firasans".into(),
        data: font_data,
    };
    Style {
        name: STYLE_LIST[0].0,
        font: Some(font),
        colors: Colors {
            panel_fill: Color32::from_rgb(249, 247, 236),
            hyperlink_color: Color32::from_rgb(3, 123, 66),
            selected_bg_fill: Color32::from_rgb(206, 247, 197),
            hover_weak_bg_fill: Color32::from_rgb(206, 247, 197),
            active_weak_bg_fill: Color32::from_rgb(47, 155, 23),
            noninteractive_fg: Color32::from_rgb(101, 114, 120),
            highlighted_text: Color32::BLACK,
            window_fill: Color32::from_rgb(206, 197, 139),
            inactive_weak_bg_fill: Color32::from_rgb(237, 158, 9),
            inactive_fg_stroke: Color32::WHITE,
            open_weak_bg_fill: Color32::from_rgb(38, 67, 35),
            hover_fg_stroke: Color32::from_rgb(32, 162, 67),
            selection_stroke: Color32::from_rgb(213, 167, 42),
        },
        icons: Icons { settings: "⚙" },
    }
}

pub fn egui_dark() -> Style {
    vanilla_egui(egui::Visuals::dark(), STYLE_LIST[1].0, Color32::WHITE)
}

pub fn egui_light() -> Style {
    vanilla_egui(egui::Visuals::light(), STYLE_LIST[2].0, Color32::BROWN)
}

pub fn vanilla_egui(
    visuals: egui::Visuals,
    name: &'static str,
    highlighted_text: Color32,
) -> Style {
    Style {
        name,
        font: None,
        colors: Colors {
            panel_fill: visuals.panel_fill,
            hyperlink_color: visuals.hyperlink_color,
            selected_bg_fill: visuals.selection.bg_fill,
            hover_weak_bg_fill: visuals.widgets.hovered.weak_bg_fill,
            active_weak_bg_fill: visuals.widgets.active.weak_bg_fill,
            noninteractive_fg: visuals.widgets.noninteractive.fg_stroke.color,
            highlighted_text,
            window_fill: visuals.window_fill,
            inactive_weak_bg_fill: visuals.widgets.inactive.weak_bg_fill,
            inactive_fg_stroke: visuals.widgets.inactive.fg_stroke.color,
            open_weak_bg_fill: visuals.widgets.open.weak_bg_fill,
            hover_fg_stroke: visuals.widgets.hovered.fg_stroke.color,
            selection_stroke: visuals.selection.stroke.color,
        },
        icons: Icons { settings: "⚙" },
    }
}

pub fn apply_style(egui_ctx: &egui::Context, style: Style) {
    let mut font_defs = FontDefinitions::default();
    if let Some(font) = style.font {
        font_defs.font_data.insert(font.name.clone(), font.data);
        if let Some(fam) = font_defs.families.get_mut(&egui::FontFamily::Proportional) {
            fam.insert(0, font.name);
        }
    }
    egui_ctx.set_fonts(font_defs);
    egui_ctx.style_mut(|egui_style| {
        egui_style
            .text_styles
            .get_mut(&egui::TextStyle::Heading)
            .unwrap()
            .size = 24.0;
        egui_style
            .text_styles
            .get_mut(&egui::TextStyle::Body)
            .unwrap()
            .size = 16.0;
        egui_style
            .text_styles
            .get_mut(&egui::TextStyle::Button)
            .unwrap()
            .size = 16.0;
        egui_style.visuals.panel_fill = style.colors.panel_fill;
        egui_style.visuals.widgets.noninteractive.fg_stroke.color = style.colors.noninteractive_fg;
        egui_style.visuals.selection.bg_fill = style.colors.selected_bg_fill;
        egui_style.visuals.hyperlink_color = style.colors.hyperlink_color;
        egui_style.visuals.widgets.hovered.weak_bg_fill = style.colors.hover_weak_bg_fill;
        egui_style.visuals.widgets.active.weak_bg_fill = style.colors.active_weak_bg_fill;
        egui_style.visuals.window_fill = style.colors.window_fill;
        egui_style.visuals.widgets.inactive.weak_bg_fill = style.colors.inactive_weak_bg_fill;
        egui_style.visuals.widgets.inactive.fg_stroke.color = style.colors.inactive_fg_stroke;
        egui_style.visuals.widgets.open.weak_bg_fill = style.colors.open_weak_bg_fill;
        egui_style.visuals.widgets.hovered.fg_stroke.color = style.colors.hover_fg_stroke;
        egui_style.visuals.selection.stroke.color = style.colors.selection_stroke;
    });
}
