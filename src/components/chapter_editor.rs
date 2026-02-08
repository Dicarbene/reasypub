use std::path::PathBuf;

use egui::{Context, Id, Modal, ScrollArea, Ui};

use crate::conversion::StrategyFactory;
use crate::{ChapterDraft, ConversionMethod, Key, Locale, t, t1, t2};

pub struct ChapterEditorInput<'a> {
    pub text: &'a str,
    pub method: ConversionMethod,
    pub regex: &'a str,
    pub config_path: Option<&'a PathBuf>,
}

#[derive(Default)]
pub struct ChapterEditorState {
    pub open: bool,
    pub use_for_conversion: bool,
    pub chapters: Vec<ChapterDraft>,
    pub stale: bool,
    pub error: Option<String>,
    last_refresh_signature: Option<u64>,
    was_open: bool,
    modal_size: Option<egui::Vec2>,
}

impl ChapterEditorState {
    pub fn show(&mut self, ctx: &Context, input: &ChapterEditorInput<'_>, locale: Locale) {
        if !self.open {
            self.was_open = false;
            return;
        }

        let current_signature = crate::chapter_signature(
            input.text,
            input.method,
            input.regex,
            input.config_path.map(|path| path.as_path()),
        );
        self.update_stale(current_signature);

        let ignore_input = !self.was_open;
        self.was_open = true;

        let fixed_size = *self.modal_size.get_or_insert_with(|| {
            let screen = ctx.content_rect();
            let width = 860.0_f32.min(screen.width() * 0.95).max(420.0);
            let height = 640.0_f32.min(screen.height() * 0.92).max(320.0);
            egui::vec2(width, height)
        });
        let mut request_close = false;

        let modal = Modal::new(Id::new("chapter_editor_modal")).show(ctx, |ui| {
            ui.set_min_size(fixed_size);
            ui.set_max_size(fixed_size);

            let mut render_body = |ui: &mut Ui| {
                ui.horizontal(|ui| {
                    ui.heading(t(locale, Key::ChapterEditorTitle));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(t(locale, Key::Close)).clicked() {
                            request_close = true;
                            ui.close();
                        }
                    });
                });

                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                self.header_ui(ui, input, current_signature, locale);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                let scroll_height = ui.available_height().max(120.0);
                ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .max_height(scroll_height)
                    .show(ui, |ui| {
                        if self.chapters.is_empty() {
                            ui.centered_and_justified(|ui| {
                                ui.label(t(locale, Key::NoChapters));
                            });
                        } else {
                            self.chapters_ui(ui, locale);
                        }
                    });
            };

            if ignore_input {
                ui.add_enabled_ui(false, |ui| render_body(ui));
            } else {
                render_body(ui);
            }
        });

        if request_close || modal.should_close() {
            self.open = false;
            self.was_open = false;
            self.modal_size = None;
        }
    }

    fn header_ui(
        &mut self,
        ui: &mut Ui,
        input: &ChapterEditorInput<'_>,
        signature: u64,
        locale: Locale,
    ) {
        ui.horizontal(|ui| {
            if ui.button(t(locale, Key::Refresh)).clicked() {
                self.refresh(input, signature);
            }
            if ui.button(t(locale, Key::AddChapter)).clicked() {
                self.chapters.push(ChapterDraft {
                    title: t(locale, Key::NewChapter).to_string(),
                    content: String::new(),
                });
                self.stale = false;
                self.error = None;
            }
            if ui.button(t(locale, Key::Clear)).clicked() {
                self.chapters.clear();
                self.stale = false;
                self.error = None;
            }
        });

        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.use_for_conversion,
                t(locale, Key::UseChapterEdits),
            );
            if self.stale {
                ui.label(
                    egui::RichText::new(t(locale, Key::ChapterWarningStale))
                        .color(egui::Color32::from_rgb(207, 95, 38)),
                );
            }
        });

        if let Some(err) = &self.error {
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        } else {
            ui.label(t1(locale, Key::ChaptersCount, self.chapters.len()));
        }
    }

    fn chapters_ui(&mut self, ui: &mut Ui, locale: Locale) {
        let mut move_actions: Vec<(usize, isize)> = Vec::new();
        let mut remove_indices: Vec<usize> = Vec::new();

        let total = self.chapters.len();
        for (index, chapter) in self.chapters.iter_mut().enumerate() {
            let header = t2(locale, Key::ChapterIndex, index + 1, &chapter.title);
            let header_id = ui.make_persistent_id(("chapter_editor_chapter", index));
            let header_response = egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                header_id,
                index < 2,
            )
            .show_header(ui, |ui| {
                ui.add(egui::Label::new(header).wrap());
            });

            header_response.body(|ui| {
                ui.horizontal(|ui| {
                    if ui.small_button(t(locale, Key::Up)).clicked() && index > 0 {
                        move_actions.push((index, -1));
                    }
                    if ui.small_button(t(locale, Key::Down)).clicked() && index + 1 < total {
                        move_actions.push((index, 1));
                    }
                    if ui.small_button(t(locale, Key::Delete)).clicked() {
                        remove_indices.push(index);
                    }
                });

                ui.label(t(locale, Key::ChapterTitle));
                let title_width = ui.available_width().max(120.0);
                ui.add_sized(
                    [title_width, ui.spacing().interact_size.y],
                    egui::TextEdit::singleline(&mut chapter.title),
                );
                ui.add_space(6.0);
                ui.label(t(locale, Key::ChapterContent));
                let content_height = 160.0;
                let content_width = ui.available_width().max(120.0);
                ui.add_sized(
                    [content_width, content_height],
                    egui::TextEdit::multiline(&mut chapter.content)
                        .desired_rows(8)
                        .lock_focus(true),
                );
            });
            ui.add_space(6.0);
        }

        for (index, direction) in move_actions {
            let new_index = (index as isize + direction) as usize;
            if index < self.chapters.len() && new_index < self.chapters.len() {
                self.chapters.swap(index, new_index);
            }
        }

        for index in remove_indices.into_iter().rev() {
            if index < self.chapters.len() {
                self.chapters.remove(index);
            }
        }
    }

    fn refresh(&mut self, input: &ChapterEditorInput<'_>, signature: u64) {
        match StrategyFactory::create(input.method, input.regex, input.config_path) {
            Ok(strategy) => match strategy.split(input.text) {
                Ok(chapters) => {
                    self.chapters = chapters;
                    self.error = None;
                    self.stale = false;
                    self.last_refresh_signature = Some(signature);
                }
                Err(err) => {
                    self.error = Some(err.to_string());
                }
            },
            Err(err) => {
                self.error = Some(err.to_string());
            }
        }
    }

    pub fn update_stale(&mut self, signature: u64) -> bool {
        let stale = self
            .last_refresh_signature
            .map(|last| last != signature)
            .unwrap_or(true);
        self.stale = stale;
        stale
    }
}
