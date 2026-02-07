use std::path::{Path, PathBuf};

use crate::{t, t1, Key};
use crate::components::chapter_editor::ChapterEditorInput;

use super::super::app_helpers::open_in_file_manager;
use super::super::MainApp;

pub(super) fn dialogs(app: &mut MainApp, ctx: &egui::Context) {
    let locale = app.locale;
    let tr = |key| t(locale, key);

    if app.show_editor {
        egui::Window::new(tr(Key::TextEditor))
            .collapsible(false)
            .resizable(true)
            .default_width(800.0)
            .default_height(600.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if ui.button(tr(Key::Save)).clicked() {
                        if let Some(path) = &app.input_file.path {
                            if let Err(e) = std::fs::write(path, &app.input_file.content) {
                                eprintln!("Save failed: {}", e);
                            }
                        }
                    }
                    if ui.button(tr(Key::Close)).clicked() {
                        app.show_editor = false;
                    }
                });

                ui.add_space(10.0);
                ui.separator();
                ui.add_space(10.0);

                egui::ScrollArea::vertical()
                    .max_height(500.0)
                    .show(ui, |ui| {
                        ui.text_edit_multiline(&mut app.input_file.content);
                    });

                ui.add_space(10.0);
                ui.label(t1(locale, Key::Chars, app.input_file.content.chars().count()));
            });
    }

    if app.show_conversion_modal {
        egui::Window::new(tr(Key::ConversionResult))
            .collapsible(false)
            .resizable(false)
            .fixed_size(egui::vec2(500.0, 300.0))
            .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);

                    if let Some(error) = &app.conversion_error {
                        ui.label(
                            egui::RichText::new(tr(Key::ConversionFailed))
                                .size(24.0)
                                .color(egui::Color32::RED),
                        );
                        ui.add_space(20.0);
                        ui.label(egui::RichText::new(error).size(16.0));
                    } else if let Some(output_path) = &app.conversion_result {
                        ui.label(
                            egui::RichText::new(tr(Key::ConversionSuccess))
                                .size(24.0)
                                .color(egui::Color32::GREEN),
                        );
                        ui.add_space(20.0);

                        ui.label(tr(Key::OutputFile));
                        ui.add_space(5.0);
                        ui.label(
                            egui::RichText::new(output_path)
                                .size(14.0)
                                .monospace(),
                        );
                        ui.add_space(20.0);

                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::OpenFolder)).clicked() {
                                if let Some(path) = PathBuf::from(output_path).parent() {
                                    let _ = open_in_file_manager(path);
                                }
                            }
                            if ui.button(tr(Key::OpenFile)).clicked() {
                                let _ = open_in_file_manager(Path::new(output_path));
                            }
                        });
                    }

                    ui.add_space(20.0);
                    ui.separator();
                    ui.add_space(10.0);

                    if ui.button(tr(Key::Close)).clicked() {
                        app.show_conversion_modal = false;
                        app.conversion_result = None;
                        app.conversion_error = None;
                    }
                });
            });
    }

    if app.chapter_editor.open {
        let input = ChapterEditorInput {
            text: &app.input_file.content,
            method: app.selected_method,
            regex: &app.custom_regex_pattern,
            config_path: app.custom_regex_file.as_ref(),
        };
        app.chapter_editor.show(ctx, &input, app.locale);
    }
}
