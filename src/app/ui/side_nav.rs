use crate::{Key, t};

use super::super::MainApp;

pub(super) fn side_nav(app: &mut MainApp, ctx: &egui::Context) {
    let locale = app.locale;
    let tr = |key| t(locale, key);

    egui::SidePanel::left("nav_panel")
        .resizable(false)
        .min_width(150.0)
        .default_width(170.0)
        .show(ctx, |ui| {
            ui.heading(tr(Key::Sections));
            ui.add_space(6.0);
            for panel in &app.available_panels {
                let selected = app.panel_index == *panel;
                if ui.selectable_label(selected, panel.label(locale)).clicked() {
                    app.panel_index = *panel;
                }
            }
            ui.add_space(10.0);
            ui.separator();
            ui.label(tr(Key::QuickActions));
            if ui.button(tr(Key::EditTxt)).clicked() {
                app.show_editor = true;
            }
            if ui.button(tr(Key::ChapterEditor)).clicked() {
                app.chapter_editor.open = true;
            }
            ui.checkbox(
                &mut app.chapter_editor.use_for_conversion,
                tr(Key::UseChapterEdits),
            );
            ui.add_space(6.0);
            ui.checkbox(&mut app.include_images_section, tr(Key::IncludeGallery));
            ui.checkbox(&mut app.inline_toc, tr(Key::InsertToc));
        });
}
