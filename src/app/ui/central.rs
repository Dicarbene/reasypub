use bytes::Bytes;
use regex::Regex;
use rfd::FileDialog;

use crate::{
    t, t1, t2, ConversionMethod, CssTemplate, ImageFileReader, Key, PanelIndex,
};

use super::super::app_helpers::{
    card, image_reader_from_path, load_font_asset, powered_by_egui_and_eframe, primary_button,
    readtxt,
};
use super::super::MainApp;

pub(super) fn central_panel(app: &mut MainApp, ctx: &egui::Context) {
    let locale = app.locale;
    let tr = |key| t(locale, key);

    egui::CentralPanel::default().show(ctx, |ui| {
        egui::ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                card(ui, tr(Key::Basics), |ui| {
                    readtxt(
                        ui,
                        locale,
                        &mut app.input_file,
                        &mut app.input_txt_path,
                        &mut app.book_info,
                    );

                    ui.horizontal(|ui| {
                        if ui.button(tr(Key::ChangeCover)).clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter(tr(Key::PanelImages), &["jpeg", "png", "webp", "jpg"])
                                .pick_file()
                            {
                                if let Ok(metadata) = std::fs::metadata(&path) {
                                    if metadata.len() > 10 * 1024 * 1024 {
                                        app.input_image.error =
                                            Some(t(locale, Key::FileTooLarge).to_string());
                                        return;
                                    }
                                }

                                match std::fs::read(&path) {
                                    Ok(content) => {
                                        app.input_image.content = Bytes::from(content);
                                        app.input_image.error = None;
                                        app.input_image.path = Some(path.clone());
                                        app.input_image_path = path.to_string_lossy().to_string();
                                        app.input_image.caption = path
                                            .file_stem()
                                            .and_then(|s| s.to_str())
                                            .map(|s| s.to_string());
                                        app.input_image.texture = None;
                                    }
                                    Err(e) => {
                                        app.input_image.error = Some(t1(locale, Key::ReadFailed, e));
                                    }
                                }
                            }
                        }
                        if ui.button(tr(Key::ClearCover)).clicked() {
                            app.input_image = ImageFileReader::default();
                            app.input_image_path.clear();
                        }

                        if app.input_image.error.is_none() {
                            if app.input_image.path.is_some() && !app.input_image_path.is_empty() {
                                ui.label(&app.input_image_path);
                            } else {
                                ui.label(tr(Key::InputImagePlaceholder));
                            }
                        }
                        if let Some(err) = &app.input_image.error {
                            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                        }
                    });

                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(tr(Key::TitleLabel));
                        ui.text_edit_singleline(&mut app.book_info.title);
                        ui.label(tr(Key::AuthorLabel));
                        ui.text_edit_singleline(&mut app.book_info.author);
                    });

                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button(tr(Key::EditTxt)).clicked() {
                            app.show_editor = true;
                        }
                        if ui.button(tr(Key::ChapterEditor)).clicked() {
                            app.chapter_editor.open = true;
                        }
                        if primary_button(ui, tr(Key::Convert)).clicked() {
                            app.run_conversion();
                        }
                    });
                });

                ui.add_space(12.0);

                let panel_title = app.panel_index.label(locale);

                card(ui, panel_title, |ui| match &app.panel_index {
                    PanelIndex::Chapter => {
                        ui.label(tr(Key::SplitMethod));
                        let methods = app.available_methods.clone();
                        for method in methods {
                            match method {
                                ConversionMethod::CustomConfig => {
                                    ui.vertical(|ui| {
                                        ui.radio_value(
                                            &mut app.selected_method,
                                            method,
                                            method.label(locale),
                                        );
                                        ui.horizontal(|ui| {
                                            if ui.button(tr(Key::ChooseConfigFile)).clicked() {
                                                if let Some(path) = FileDialog::new()
                                                    .add_filter(
                                                        tr(Key::TextFileFilter),
                                                        &["txt", "conf", "regex"],
                                                    )
                                                    .pick_file()
                                                {
                                                    app.custom_regex_file = Some(path.clone());
                                                    app.custom_regex_path =
                                                        path.to_string_lossy().to_string();
                                                    app.custom_regex_status = Some(
                                                        app.validate_custom_config(locale, &path),
                                                    );
                                                }
                                            }
                                            if ui.button(tr(Key::ClearConfig)).clicked() {
                                                app.custom_regex_file = None;
                                                app.custom_regex_path.clear();
                                                app.custom_regex_status = None;
                                            }
                                            let config_label = if app.custom_regex_file.is_some()
                                                && !app.custom_regex_path.is_empty()
                                            {
                                                app.custom_regex_path.as_str()
                                            } else {
                                                tr(Key::NoConfigSelected)
                                            };
                                            ui.label(config_label);
                                        });
                                        ui.horizontal(|ui| {
                                            if ui.button(tr(Key::ValidateConfig)).clicked() {
                                                if let Some(path) =
                                                    app.custom_regex_file.as_ref()
                                                {
                                                    app.custom_regex_status = Some(
                                                        app.validate_custom_config(locale, path),
                                                    );
                                                } else {
                                                    app.custom_regex_status = Some((
                                                        false,
                                                        t(locale, Key::NoConfigSelected)
                                                            .to_string(),
                                                    ));
                                                }
                                            }
                                            if let Some((ok, message)) =
                                                &app.custom_regex_status
                                            {
                                                let color = if *ok {
                                                    egui::Color32::GREEN
                                                } else {
                                                    egui::Color32::RED
                                                };
                                                ui.label(
                                                    egui::RichText::new(message).color(color),
                                                );
                                            }
                                        });
                                    });
                                }
                                ConversionMethod::Regex => {
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.radio_value(
                                                &mut app.selected_method,
                                                method,
                                                method.label(locale),
                                            );
                                            ui.label(tr(Key::RegexPattern));
                                            ui.text_edit_singleline(&mut app.custom_regex_pattern);
                                        });
                                        if app.custom_regex_pattern.trim().is_empty() {
                                            ui.label(
                                                egui::RichText::new(tr(Key::BuiltinChinesePattern))
                                                    .color(egui::Color32::GRAY),
                                            );
                                        } else {
                                            match Regex::new(app.custom_regex_pattern.trim()) {
                                                Ok(_) => {
                                                    ui.label(
                                                        egui::RichText::new(tr(Key::RegexOk))
                                                            .color(egui::Color32::GREEN),
                                                    );
                                                }
                                                Err(err) => {
                                                    ui.label(
                                                        egui::RichText::new(t1(
                                                            locale,
                                                            Key::RegexError,
                                                            err,
                                                        ))
                                                        .color(egui::Color32::RED),
                                                    );
                                                }
                                            }
                                        }
                                    });
                                }
                                ConversionMethod::SimpleRules => {
                                    ui.horizontal(|ui| {
                                        ui.radio_value(
                                            &mut app.selected_method,
                                            method,
                                            method.label(locale),
                                        );
                                        ui.label(tr(Key::SimpleRule));
                                        ui.label(
                                            egui::RichText::new(
                                                "\\s*[第卷][0123456789一二三四五六七八九十零〇百千两]*[章回部节集卷].*",
                                            )
                                            .monospace()
                                            .size(12.0),
                                        );
                                    });
                                }
                            }
                        }
                        ui.add_space(6.0);
                        ui.checkbox(
                            &mut app.chapter_editor.use_for_conversion,
                            tr(Key::UseChapterEdits),
                        );
                        ui.separator();
                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::PreviewChapters)).clicked() {
                                app.refresh_chapter_preview();
                            }
                            let stale = app
                                .chapter_preview_signature
                                .map(|sig| sig != app.preview_signature())
                                .unwrap_or(true);
                            if app.chapter_preview.is_some() && stale {
                                ui.label(
                                    egui::RichText::new(tr(Key::PreviewStale))
                                        .color(egui::Color32::from_rgb(207, 95, 38)),
                                );
                            }
                        });
                        if let Some(err) = &app.chapter_preview_error {
                            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                        } else if let Some(preview) = &app.chapter_preview {
                            ui.label(t1(locale, Key::ChaptersCount, preview.count));
                            for (idx, title) in preview.titles.iter().enumerate() {
                                ui.label(t2(locale, Key::ChapterIndex, idx + 1, title));
                            }
                        } else {
                            ui.label(tr(Key::NoPreview));
                        }
                    }
                    PanelIndex::Format => {
                        ui.horizontal(|ui| {
                            ui.label(tr(Key::LineHeight));
                            ui.add(
                                egui::Slider::new(&mut app.text_style.line_height, 1.0..=3.0)
                                    .step_by(0.1),
                            );
                            ui.label(format!("{:.1}", app.text_style.line_height));
                        });
                        ui.horizontal(|ui| {
                            ui.label(tr(Key::ParagraphSpacing));
                            ui.add(
                                egui::Slider::new(
                                    &mut app.text_style.paragraph_spacing,
                                    0.0..=3.0,
                                )
                                .step_by(0.1),
                            );
                            ui.label(format!("{:.1}", app.text_style.paragraph_spacing));
                        });
                        ui.horizontal(|ui| {
                            ui.label(tr(Key::IndentEm));
                            ui.add(
                                egui::Slider::new(&mut app.text_style.text_indent, 0.0..=4.0)
                                    .step_by(0.5),
                            );
                            ui.label(format!("{:.1}", app.text_style.text_indent));
                        });
                    }
                    PanelIndex::CSS => {
                        ui.horizontal(|ui| {
                            ui.label(tr(Key::Template));
                            egui::ComboBox::from_id_salt("css_template")
                                .selected_text(app.text_style.css_template.to_string())
                                .show_ui(ui, |ui| {
                                    for template in CssTemplate::ALL {
                                        ui.selectable_value(
                                            &mut app.text_style.css_template,
                                            template,
                                            template.to_string(),
                                        );
                                    }
                                });
                        });

                        ui.add_space(8.0);
                        ui.label(tr(Key::CustomCss));
                        ui.add_space(4.0);
                        egui::ScrollArea::vertical()
                            .max_height(260.0)
                            .show(ui, |ui| {
                                ui.text_edit_multiline(&mut app.text_style.custom_css);
                            });

                        ui.add_space(10.0);
                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::ImportCss)).clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("CSS", &["css"])
                                    .pick_file()
                                {
                                    if let Ok(content) = std::fs::read_to_string(&path) {
                                        app.text_style.custom_css = content;
                                    }
                                }
                            }
                            if ui.button(tr(Key::ExportCss)).clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter("CSS", &["css"])
                                    .save_file()
                                {
                                    let _ = std::fs::write(&path, &app.text_style.custom_css);
                                }
                            }
                        });

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(tr(Key::ChapterHeaderImage));
                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::ChooseChapterHeader)).clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter(
                                        tr(Key::PanelImages),
                                        &["jpeg", "png", "webp", "jpg"],
                                    )
                                    .pick_file()
                                {
                                    app.chapter_header_image =
                                        image_reader_from_path(locale, &path);
                                    app.chapter_header_image_path =
                                        path.to_string_lossy().to_string();
                                }
                            }
                            if ui.button(tr(Key::ClearChapterHeader)).clicked() {
                                app.chapter_header_image = ImageFileReader::default();
                                app.chapter_header_image_path.clear();
                            }

                            if app.chapter_header_image.error.is_none() {
                                if app.chapter_header_image.path.is_some()
                                    && !app.chapter_header_image_path.is_empty()
                                {
                                    ui.label(&app.chapter_header_image_path);
                                } else {
                                    ui.label(tr(Key::ChapterHeaderPlaceholder));
                                }
                            }
                            if let Some(err) = &app.chapter_header_image.error {
                                ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                            }
                        });
                        ui.checkbox(
                            &mut app.chapter_header_fullbleed,
                            tr(Key::ChapterHeaderFullBleed),
                        );

                        ui.add_space(12.0);
                        ui.separator();
                        ui.add_space(8.0);
                        ui.label(tr(Key::ExtraBodyClass));
                        ui.text_edit_singleline(&mut app.text_style.extra_body_class);
                        ui.label(tr(Key::ExtraChapterClass));
                        ui.text_edit_singleline(&mut app.text_style.extra_chapter_class);
                        ui.label(tr(Key::ExtraTitleClass));
                        ui.text_edit_singleline(&mut app.text_style.extra_title_class);
                        ui.label(tr(Key::ExtraParagraphClass));
                        ui.text_edit_singleline(&mut app.text_style.extra_paragraph_class);
                        ui.add_space(4.0);
                        ui.label(tr(Key::ClassMarkerHint));
                    }
                    PanelIndex::Font => {
                        ui.horizontal(|ui| {
                            ui.label(tr(Key::FontSize));
                            ui.add(
                                egui::Slider::new(&mut app.text_style.font_size, 10.0..=32.0)
                                    .step_by(1.0),
                            );
                            ui.label(format!("{:.0}", app.text_style.font_size));
                        });
                        ui.horizontal(|ui| {
                            ui.label(tr(Key::FontColor));
                            ui.color_edit_button_srgba(&mut app.text_style.font_color);
                        });
                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::ChooseFont)).clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter(tr(Key::PanelFonts), &["ttf", "otf"])
                                    .pick_file()
                                {
                                    match load_font_asset(&path) {
                                        Ok(asset) => {
                                            app.font_asset = Some(asset);
                                            app.font_error = None;
                                            app.text_style.font_path =
                                                path.to_string_lossy().to_string();
                                        }
                                        Err(err) => {
                                            app.font_error =
                                                Some(t1(locale, Key::ReadFailed, err));
                                        }
                                    }
                                }
                            }
                            if ui.button(tr(Key::ClearFont)).clicked() {
                                app.font_asset = None;
                                app.text_style.font_path.clear();
                            }
                        });
                        if let Some(err) = &app.font_error {
                            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                        } else if !app.text_style.font_path.is_empty() {
                            ui.label(t1(locale, Key::FontLabel, &app.text_style.font_path));
                        }
                    }
                    PanelIndex::Images => {
                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::AddImage)).clicked() {
                                if let Some(path) = FileDialog::new()
                                    .add_filter(
                                        tr(Key::PanelImages),
                                        &["jpeg", "png", "webp", "jpg", "gif"],
                                    )
                                    .pick_file()
                                {
                                    app.images.push(image_reader_from_path(locale, &path));
                                }
                            }
                            ui.label(t1(locale, Key::TotalImages, app.images.len()));
                        });

                        ui.add_space(8.0);
                        egui::ScrollArea::vertical()
                            .max_height(360.0)
                            .show(ui, |ui| {
                                if app.images.is_empty() {
                                    ui.centered_and_justified(|ui| {
                                        ui.label(tr(Key::NoImages));
                                    });
                                } else {
                                    let mut indices_to_remove = Vec::new();
                                    for (index, image) in app.images.iter_mut().enumerate() {
                                        ui.group(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(t1(locale, Key::ImageIndex, index + 1));
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(egui::Align::Center),
                                                    |ui| {
                                                        if ui.small_button(tr(Key::Delete)).clicked()
                                                        {
                                                            indices_to_remove.push(index);
                                                        }
                                                    },
                                                );
                                            });

                                            if let Some(path) = &image.path {
                                                ui.label(path.to_string_lossy().to_string());
                                            }

                                            ui.label(tr(Key::Caption));
                                            let caption =
                                                image.caption.get_or_insert_with(String::new);
                                            ui.text_edit_singleline(caption);

                                            image.update_texture(ui.ctx());
                                            if let Some(texture) = &image.texture {
                                                ui.add(
                                                    egui::Image::from_texture(texture)
                                                        .max_width(150.0),
                                                );
                                            } else if let Some(err) = &image.error {
                                                ui.label(
                                                    egui::RichText::new(err)
                                                        .color(egui::Color32::RED),
                                                );
                                            } else {
                                                ui.label(tr(Key::Loading));
                                            }
                                        });
                                    }
                                    for index in indices_to_remove.into_iter().rev() {
                                        app.images.remove(index);
                                    }
                                }
                            });

                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button(tr(Key::BatchImport)).clicked() {
                                if let Some(paths) = FileDialog::new()
                                    .add_filter(
                                        tr(Key::PanelImages),
                                        &["jpeg", "png", "webp", "jpg", "gif"],
                                    )
                                    .pick_files()
                                {
                                    for path in paths {
                                        app.images.push(image_reader_from_path(locale, &path));
                                    }
                                }
                            }
                            if ui.button(tr(Key::ClearAll)).clicked() {
                                app.images.clear();
                            }
                        });
                    }
                    PanelIndex::PublishInfo => {
                        ui.label(tr(Key::LanguageField));
                        ui.text_edit_singleline(&mut app.book_info.language);
                        ui.label(tr(Key::Publisher));
                        ui.text_edit_singleline(&mut app.book_info.publisher);
                        ui.label(tr(Key::Isbn));
                        ui.text_edit_singleline(&mut app.book_info.isbn);
                        ui.label(tr(Key::Category));
                        ui.text_edit_singleline(&mut app.book_info.category);
                        ui.label(tr(Key::PublishDate));
                        ui.text_edit_singleline(&mut app.book_info.publish_date);
                        ui.add_space(6.0);
                        ui.label(tr(Key::Description));
                        ui.text_edit_multiline(&mut app.book_info.description);
                    }
                    PanelIndex::Misc => {
                        ui.label(tr(Key::OutputFolder));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut app.output_path);
                            if ui.button(tr(Key::Browse)).clicked() {
                                if let Some(path) = FileDialog::new().pick_folder() {
                                    app.output_path = path.to_string_lossy().to_string();
                                }
                            }
                        });

                        ui.add_space(8.0);
                        ui.label(tr(Key::FilenameTemplate));
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut app.filename_template);
                            ui.label(tr(Key::VarsHint));
                        });

                        ui.add_space(8.0);
                        ui.label(tr(Key::Current));
                        ui.label(t1(locale, Key::OutputLabel, &app.output_path));
                        ui.label(t1(locale, Key::TemplateLabel, &app.filename_template));
                    }
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(8.0);
                powered_by_egui_and_eframe(ui, locale);
                ui.add(egui::github_link_file!(
                    "https://github.com/Dicarbene/reasypub/",
                    tr(Key::SourceLabel)
                ));
                egui::warn_if_debug_build(ui);
            });
    });
}
