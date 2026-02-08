use crate::{Key, t, t1};

use super::super::MainApp;
use super::super::app_helpers::{card, display_or_placeholder, show_image_ui};

pub(super) fn preview_panel(app: &mut MainApp, ctx: &egui::Context) {
    let locale = app.locale;
    let tr = |key| t(locale, key);

    egui::SidePanel::right("preview_panel")
        .resizable(false)
        .min_width(220.0)
        .default_width(240.0)
        .show(ctx, |ui| {
            card(ui, tr(Key::CoverPreview), |ui| {
                show_image_ui(ui, locale, &mut app.input_image);
            });

            ui.add_space(10.0);

            card(ui, tr(Key::ExportSummary), |ui| {
                ui.label(format!(
                    "{}: {}",
                    tr(Key::TitleLabel),
                    display_or_placeholder(&app.book_info.title, tr(Key::PlaceholderUntitled))
                ));
                ui.label(format!(
                    "{}: {}",
                    tr(Key::AuthorLabel),
                    display_or_placeholder(&app.book_info.author, tr(Key::PlaceholderUnknown))
                ));
                ui.label(t1(locale, Key::OutputLabel, &app.output_path));
                ui.label(t1(locale, Key::TemplateLabel, &app.filename_template));
                ui.label(t1(locale, Key::ImagesLabel, app.images.len()));
            });
        });
}
