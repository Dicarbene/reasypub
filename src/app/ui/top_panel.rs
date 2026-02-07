use crate::{t, Key, Locale};

use super::super::app_helpers::apply_theme;
use super::super::{MainApp, ThemeMode};

pub(super) fn top_panel(app: &mut MainApp, ctx: &egui::Context) {
    let locale = app.locale;
    let tr = |key| t(locale, key);

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        let accent = ui.visuals().selection.bg_fill;
        egui::Frame::NONE
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Reasypub").size(24.0).strong());
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new(tr(Key::Subtitle)).size(12.0).color(accent));
                    ui.with_layout(
                        egui::Layout::right_to_left(egui::Align::Center),
                        |ui| {
                            egui::ComboBox::from_id_salt("locale")
                                .selected_text(app.locale.label())
                                .show_ui(ui, |ui| {
                                    for loc in Locale::ALL {
                                        ui.selectable_value(&mut app.locale, loc, loc.label());
                                    }
                                });
                            ui.add_space(6.0);
                            if ui
                                .button(match app.theme_mode {
                                    ThemeMode::Light => tr(Key::ThemeDark),
                                    ThemeMode::Dark => tr(Key::ThemeLight),
                                })
                                .clicked()
                            {
                                app.theme_mode.toggle();
                                apply_theme(ctx, app.theme_mode);
                            }
                        },
                    );
                });
            });
    });
}
