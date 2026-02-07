use std::path::PathBuf;

use egui::{CollapsingHeader, Context, ScrollArea, Ui, Window};

use crate::conversion::StrategyFactory;
use crate::{t, t1, t2, ChapterDraft, ConversionMethod, Key, Locale};

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
}

impl ChapterEditorState {
    pub fn show(&mut self, ctx: &Context, input: &ChapterEditorInput<'_>, locale: Locale) {
        if !self.open {
            return;
        }

        let current_signature = crate::chapter_signature(
            input.text,
            input.method,
            input.regex,
            input.config_path.map(|path| path.as_path()),
        );
        self.update_stale(current_signature);

        let mut open = self.open;
        Window::new(t(locale, Key::ChapterEditorTitle))
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(860.0)
            .default_height(640.0)
            .show(ctx, |ui| {
                self.header_ui(ui, input, current_signature, locale);
                ui.add_space(8.0);
                ui.separator();
                ui.add_space(8.0);

                ScrollArea::vertical().show(ui, |ui| {
                    if self.chapters.is_empty() {
                        ui.centered_and_justified(|ui| {
                            ui.label(t(locale, Key::NoChapters));
                        });
                    } else {
                        self.chapters_ui(ui, locale);
                    }
                });
            });
        self.open = open;
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
            ui.checkbox(&mut self.use_for_conversion, t(locale, Key::UseChapterEdits));
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
            CollapsingHeader::new(header)
                .default_open(index < 2)
                .show(ui, |ui| {
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
                    ui.text_edit_singleline(&mut chapter.title);
                    ui.add_space(6.0);
                    ui.label(t(locale, Key::ChapterContent));
                    ui.text_edit_multiline(&mut chapter.content);
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

