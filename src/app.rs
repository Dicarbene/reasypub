use crate::components::chapter_editor::{ChapterEditorInput, ChapterEditorState};
use crate::conversion::{ConversionFacade, ConversionRequest};
use crate::{
    t, t1, t2, BookInfo, ConversionMethod, CssTemplate, FontAsset, ImageAsset, ImageFileReader,
    Key, Locale, PanelIndex, TextFileReader, TextStyle,
};
use bytes::Bytes;
use regex::Regex;
use rfd::FileDialog;
use std::path::{Path, PathBuf};

/// 我们派生 Deserialize/Serialize 以便在关闭时持久化应用状态
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // 如果添加新字段，在反序列化旧状态时给予默认值
pub struct MainApp {
    // 基础数据结构体
    input_txt_path: String, // 输入文本文件路径
    input_image_path: String, // 输入图片路径
    custom_regex_path: String, // 自定义正则配置文件路径
    #[serde(skip)]
    custom_regex_pattern: String, // 自定义正则表达式
    #[serde(skip)]
    custom_regex_file: Option<std::path::PathBuf>,
    #[serde(skip)]
    custom_regex_status: Option<(bool, String)>,
    // 转换配置
    available_methods: Vec<ConversionMethod>, // 可用的转换方法（使用枚举）
    selected_method: ConversionMethod, // 当前选中的转换方法
    available_panels: Vec<PanelIndex>, // 可用的面板索引
    panel_index: PanelIndex, // 当前面板索引
    book_info: BookInfo, // 书籍信息
    // 版式与字体配置
    text_style: TextStyle,
    // 界面主题
    theme_mode: ThemeMode,
    locale: Locale,
    // 插图配置
    #[serde(skip)]
    images: Vec<ImageFileReader>, // 插图列表
    #[serde(skip)]
    include_images_section: bool, // 是否生成插图章节
    #[serde(skip)]
    inline_toc: bool, // 是否插入目录页
    // 杂项配置
    output_path: String, // 输出路径
    filename_template: String, // 文件命名模板
    // 编辑器状态
    show_editor: bool, // 是否显示编辑器
    #[serde(skip)]
    chapter_editor: ChapterEditorState,
    #[serde(skip)]
    chapter_preview: Option<ChapterPreview>,
    #[serde(skip)]
    chapter_preview_error: Option<String>,
    #[serde(skip)]
    chapter_preview_signature: Option<u64>,
    // 转换状态
    #[serde(skip)]
    show_conversion_modal: bool, // 是否显示转换完成modal
    #[serde(skip)]
    conversion_result: Option<String>, // 转换结果（成功时的文件路径）
    #[serde(skip)]
    conversion_error: Option<String>, // 转换错误信息
    #[serde(skip)]
    input_file: TextFileReader, // 文本文件读取器
    #[serde(skip)]
    input_image: ImageFileReader, // 图片文件读取器
    #[serde(skip)]
    font_asset: Option<FontAsset>,
    #[serde(skip)]
    font_error: Option<String>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Copy, PartialEq)]
enum ThemeMode {
    Light,
    Dark,
}

impl ThemeMode {
    fn toggle(&mut self) {
        *self = match self {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        };
    }
}

impl Default for MainApp {
    // 基础数据初始化
    fn default() -> Self {
        Self {
            input_txt_path: String::new(),
            input_image_path: String::new(),
            custom_regex_path: String::new(),
            custom_regex_pattern: String::new(),
            custom_regex_file: None,
            custom_regex_status: None,
            available_methods: vec![
                ConversionMethod::SimpleRules, // 简单规则
                ConversionMethod::Regex, // 正则表达式
                ConversionMethod::CustomConfig, // 自定义配置
            ],
            selected_method: ConversionMethod::Regex, // 默认使用正则表达式方法
            available_panels: vec![
                PanelIndex::Chapter, // 章节面板
                PanelIndex::Format, // 版式面板
                PanelIndex::Font, // 字体面板
                PanelIndex::PublishInfo, // 出版信息面板
                PanelIndex::CSS, // CSS 面板
                PanelIndex::Images, // 插图面板
                PanelIndex::Misc, // 杂项面板
            ],
            panel_index: PanelIndex::Chapter, // 默认显示章节面板
            book_info: BookInfo::default(),
            text_style: TextStyle::default(),
            theme_mode: ThemeMode::Light,
            locale: Locale::Zh,
            images: Vec::new(),
            include_images_section: true,
            inline_toc: true,
            output_path: ".".to_owned(),
            filename_template: "{书名}_{作者}.epub".to_owned(),
            show_editor: false,
            chapter_editor: ChapterEditorState::default(),
            chapter_preview: None,
            chapter_preview_error: None,
            chapter_preview_signature: None,
            show_conversion_modal: false,
            conversion_result: None,
            conversion_error: None,
            input_file: TextFileReader::default(),
            input_image: ImageFileReader::default(),
            font_asset: None,
            font_error: None,
        }
    }
}

impl MainApp {
    /// 在第一帧被调用一次
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 这里也可以自定义 egui 的外观
        // 使用 `cc.egui_ctx.set_visuals` 和 `cc.egui_ctx.set_fonts`
        // 创建字体配置
        use std::sync::Arc;
        let mut fonts = egui::FontDefinitions::default();

        // 插入自定义字体数据
        fonts.font_data.insert(
            "stdg".to_owned(),
            Arc::new(egui::FontData::from_static(include_bytes!(
                "../assets/stdg-regular.ttf"
            ))), // 注意这里补全了括号
        );

        // 设置字体
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "stdg".to_owned());
        
        // 为等宽字体也设置中文字体支持
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "stdg".to_owned());
        
        cc.egui_ctx.set_fonts(fonts);

        // 加载之前的应用状态（如果有）
        // 注意：必须启用 `persistence` 功能才能使用
/*         if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
 */
        let app = Self::default();
        apply_theme(&cc.egui_ctx, app.theme_mode);
        app
    }

    fn run_conversion(&mut self) {
        self.conversion_error = None;
        self.conversion_result = None;

        if self.input_file.content.trim().is_empty() {
            self.conversion_error = Some(t(self.locale, Key::PreviewTextEmpty).to_string());
            self.show_conversion_modal = true;
            return;
        }

        if self.chapter_editor.use_for_conversion {
            let signature = self.preview_signature();
            self.chapter_editor.update_stale(signature);
            if self.chapter_editor.stale {
                self.conversion_error =
                    Some(t(self.locale, Key::ChapterWarningStale).to_string());
                self.show_conversion_modal = true;
                return;
            }
            if self.chapter_editor.chapters.is_empty() {
                self.conversion_error = Some(t(self.locale, Key::NoChapters).to_string());
                self.show_conversion_modal = true;
                return;
            }
        }

        let cover = cover_asset_from_reader(&self.input_image);
        let images = collect_image_assets(&self.images);
        let font = self.resolve_font_asset();

        let chapters_override = if self.chapter_editor.use_for_conversion {
            Some(self.chapter_editor.chapters.clone())
        } else {
            None
        };

        let request = ConversionRequest {
            text: self.input_file.content.clone(),
            method: self.selected_method,
            custom_regex: self.custom_regex_pattern.clone(),
            custom_config_path: self.custom_regex_file.clone(),
            book_info: self.book_info.clone(),
            output_dir: PathBuf::from(&self.output_path),
            filename_template: self.filename_template.clone(),
            style: self.text_style.clone(),
            cover,
            images,
            font,
            chapters_override,
            include_images_section: self.include_images_section,
            inline_toc: self.inline_toc,
        };

        match ConversionFacade::convert(request) {
            Ok(result) => {
                self.conversion_result = Some(result.output_path);
                self.conversion_error = None;
                self.show_conversion_modal = true;
            }
            Err(err) => {
                self.conversion_error = Some(err.to_string());
                self.conversion_result = None;
                self.show_conversion_modal = true;
            }
        }
    }

    fn resolve_font_asset(&mut self) -> Option<FontAsset> {
        if let Some(asset) = self.font_asset.clone() {
            return Some(asset);
        }
        if self.text_style.font_path.trim().is_empty() {
            return None;
        }

        match load_font_asset(Path::new(&self.text_style.font_path)) {
            Ok(asset) => {
                self.font_error = None;
                self.font_asset = Some(asset.clone());
                Some(asset)
            }
            Err(err) => {
                self.font_error = Some(t1(self.locale, Key::ReadFailed, err));
                None
            }
        }
    }

    fn refresh_chapter_preview(&mut self) {
        if self.input_file.content.trim().is_empty() {
            self.chapter_preview_error = Some(t(self.locale, Key::PreviewTextEmpty).to_string());
            self.chapter_preview = None;
            return;
        }

        let signature = self.preview_signature();
        match crate::conversion::StrategyFactory::create(
            self.selected_method,
            &self.custom_regex_pattern,
            self.custom_regex_file.as_ref(),
        ) {
            Ok(strategy) => match strategy.split(&self.input_file.content) {
                Ok(chapters) => {
                    let titles = chapters
                        .iter()
                        .take(2)
                        .map(|c| c.title.clone())
                        .collect::<Vec<_>>();
                    self.chapter_preview = Some(ChapterPreview {
                        count: chapters.len(),
                        titles,
                    });
                    self.chapter_preview_error = None;
                    self.chapter_preview_signature = Some(signature);
                }
                Err(err) => {
                    self.chapter_preview_error = Some(err.to_string());
                    self.chapter_preview = None;
                }
            },
            Err(err) => {
                self.chapter_preview_error = Some(err.to_string());
                self.chapter_preview = None;
            }
        }
    }

    fn preview_signature(&self) -> u64 {
        crate::chapter_signature(
            &self.input_file.content,
            self.selected_method,
            &self.custom_regex_pattern,
            self.custom_regex_file.as_deref(),
        )
    }

    fn validate_custom_config(&self, locale: Locale, path: &Path) -> (bool, String) {
        match std::fs::read_to_string(path) {
            Ok(content) => match Regex::new(content.trim()) {
                Ok(_) => (true, t(locale, Key::ConfigRegexOk).to_string()),
                Err(err) => (false, t1(locale, Key::RegexError, err)),
            },
            Err(err) => (false, t1(locale, Key::ReadFailed, err)),
        }
    }
}

#[derive(Clone, Debug, Default)]
struct ChapterPreview {
    count: usize,
    titles: Vec<String>,
}

impl eframe::App for MainApp {
    /// 由框架在关闭前调用以保存状态
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// 每帧更新都调用
    
    
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        apply_theme(ctx, self.theme_mode);
        let locale = self.locale;
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
                                    .selected_text(self.locale.label())
                                    .show_ui(ui, |ui| {
                                        for loc in Locale::ALL {
                                            ui.selectable_value(
                                                &mut self.locale,
                                                loc,
                                                loc.label(),
                                            );
                                        }
                                    });
                                ui.add_space(6.0);
                                if ui
                                    .button(match self.theme_mode {
                                        ThemeMode::Light => tr(Key::ThemeDark),
                                        ThemeMode::Dark => tr(Key::ThemeLight),
                                    })
                                    .clicked()
                                {
                                    self.theme_mode.toggle();
                                    apply_theme(ctx, self.theme_mode);
                                }
                            },
                        );
                    });
                });
        });

        egui::SidePanel::left("nav_panel")
            .resizable(false)
            .min_width(150.0)
            .default_width(170.0)
            .show(ctx, |ui| {
                ui.heading(tr(Key::Sections));
                ui.add_space(6.0);
                for panel in &self.available_panels {
                    let selected = self.panel_index == *panel;
                    if ui
                        .selectable_label(selected, panel.label(locale))
                        .clicked()
                    {
                        self.panel_index = *panel;
                    }
                }
                ui.add_space(10.0);
                ui.separator();
                ui.label(tr(Key::QuickActions));
                if ui.button(tr(Key::EditTxt)).clicked() {
                    self.show_editor = true;
                }
                if ui.button(tr(Key::ChapterEditor)).clicked() {
                    self.chapter_editor.open = true;
                }
                ui.checkbox(&mut self.chapter_editor.use_for_conversion, tr(Key::UseChapterEdits));
                ui.add_space(6.0);
                ui.checkbox(&mut self.include_images_section, tr(Key::IncludeGallery));
                ui.checkbox(&mut self.inline_toc, tr(Key::InsertToc));
            });

        egui::SidePanel::right("preview_panel")
            .resizable(false)
            .min_width(220.0)
            .default_width(240.0)
            .show(ctx, |ui| {
                card(ui, tr(Key::CoverPreview), |ui| {
                    show_image_ui(ui, locale, &mut self.input_image);
                });

                ui.add_space(10.0);

                card(ui, tr(Key::ExportSummary), |ui| {
                    ui.label(format!(
                        "{}: {}",
                        tr(Key::TitleLabel),
                        display_or_placeholder(&self.book_info.title, tr(Key::PlaceholderUntitled))
                    ));
                    ui.label(format!(
                        "{}: {}",
                        tr(Key::AuthorLabel),
                        display_or_placeholder(&self.book_info.author, tr(Key::PlaceholderUnknown))
                    ));
                    ui.label(t1(locale, Key::OutputLabel, &self.output_path));
                    ui.label(t1(locale, Key::TemplateLabel, &self.filename_template));
                    ui.label(t1(locale, Key::ImagesLabel, self.images.len()));
                });
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
            card(ui, tr(Key::Basics), |ui| {
                readtxt(
                    ui,
                    locale,
                    &mut self.input_file,
                    &mut self.input_txt_path,
                    &mut self.book_info,
                );

                ui.horizontal(|ui| {
                    if ui.button(tr(Key::ChangeCover)).clicked() {
                        if let Some(path) = FileDialog::new()
                            .add_filter(tr(Key::PanelImages), &["jpeg", "png", "webp", "jpg"])
                            .pick_file()
                        {
                            if let Ok(metadata) = std::fs::metadata(&path) {
                                if metadata.len() > 10 * 1024 * 1024 {
                                    self.input_image.error =
                                        Some(t(locale, Key::FileTooLarge).to_string());
                                    return;
                                }
                            }

                            match std::fs::read(&path) {
                                Ok(content) => {
                                    self.input_image.content = Bytes::from(content);
                                    self.input_image.error = None;
                                    self.input_image.path = Some(path.clone());
                                    self.input_image_path = path.to_string_lossy().to_string();
                                    self.input_image.caption = path
                                        .file_stem()
                                        .and_then(|s| s.to_str())
                                        .map(|s| s.to_string());
                                    self.input_image.texture = None;
                                }
                                Err(e) => {
                                    self.input_image.error = Some(t1(locale, Key::ReadFailed, e));
                                }
                            }
                        }
                    }
                    if ui.button(tr(Key::ClearCover)).clicked() {
                        self.input_image = ImageFileReader::default();
                        self.input_image_path.clear();
                    }

                    if self.input_image.error.is_none() {
                        if self.input_image.path.is_some() && !self.input_image_path.is_empty() {
                            ui.label(&self.input_image_path);
                        } else {
                            ui.label(tr(Key::InputImagePlaceholder));
                        }
                    }
                    if let Some(err) = &self.input_image.error {
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    }
                });

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(tr(Key::TitleLabel));
                    ui.text_edit_singleline(&mut self.book_info.title);
                    ui.label(tr(Key::AuthorLabel));
                    ui.text_edit_singleline(&mut self.book_info.author);
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    if ui.button(tr(Key::EditTxt)).clicked() {
                        self.show_editor = true;
                    }
                    if ui.button(tr(Key::ChapterEditor)).clicked() {
                        self.chapter_editor.open = true;
                    }
                    if primary_button(ui, tr(Key::Convert)).clicked() {
                        self.run_conversion();
                    }
                });
            });

            ui.add_space(12.0);

            let panel_title = self.panel_index.label(locale);

            card(ui, panel_title, |ui| match &self.panel_index {
                PanelIndex::Chapter => {
                    ui.label(tr(Key::SplitMethod));
                    let methods = self.available_methods.clone();
                    for method in methods {
                        match method {
                            ConversionMethod::CustomConfig => {
                                ui.vertical(|ui| {
                                    ui.radio_value(
                                        &mut self.selected_method,
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
                                                self.custom_regex_file = Some(path.clone());
                                                self.custom_regex_path =
                                                    path.to_string_lossy().to_string();
                                                self.custom_regex_status =
                                                    Some(self.validate_custom_config(locale, &path));
                                            }
                                        }
                                        if ui.button(tr(Key::ClearConfig)).clicked() {
                                            self.custom_regex_file = None;
                                            self.custom_regex_path.clear();
                                            self.custom_regex_status = None;
                                        }
                                        let config_label = if self.custom_regex_file.is_some()
                                            && !self.custom_regex_path.is_empty()
                                        {
                                            self.custom_regex_path.as_str()
                                        } else {
                                            tr(Key::NoConfigSelected)
                                        };
                                        ui.label(config_label);
                                    });
                                    ui.horizontal(|ui| {
                                        if ui.button(tr(Key::ValidateConfig)).clicked() {
                                            if let Some(path) = self.custom_regex_file.as_ref() {
                                                self.custom_regex_status =
                                                    Some(self.validate_custom_config(locale, path));
                                            } else {
                                                self.custom_regex_status = Some((
                                                    false,
                                                    t(locale, Key::NoConfigSelected).to_string(),
                                                ));
                                            }
                                        }
                                        if let Some((ok, message)) = &self.custom_regex_status {
                                            let color = if *ok {
                                                egui::Color32::GREEN
                                            } else {
                                                egui::Color32::RED
                                            };
                                            ui.label(egui::RichText::new(message).color(color));
                                        }
                                    });
                                });
                            }
                            ConversionMethod::Regex => {
                                ui.vertical(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.radio_value(
                                            &mut self.selected_method,
                                            method,
                                            method.label(locale),
                                        );
                                        ui.label(tr(Key::RegexPattern));
                                        ui.text_edit_singleline(&mut self.custom_regex_pattern);
                                    });
                                    if self.custom_regex_pattern.trim().is_empty() {
                                        ui.label(
                                            egui::RichText::new(
                                                tr(Key::BuiltinChinesePattern),
                                            )
                                            .color(egui::Color32::GRAY),
                                        );
                                    } else {
                                        match Regex::new(self.custom_regex_pattern.trim()) {
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
                                        &mut self.selected_method,
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
                    ui.checkbox(&mut self.chapter_editor.use_for_conversion, tr(Key::UseChapterEdits));
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button(tr(Key::PreviewChapters)).clicked() {
                            self.refresh_chapter_preview();
                        }
                        let stale = self
                            .chapter_preview_signature
                            .map(|sig| sig != self.preview_signature())
                            .unwrap_or(true);
                        if self.chapter_preview.is_some() && stale {
                            ui.label(
                                egui::RichText::new(tr(Key::PreviewStale))
                                    .color(egui::Color32::from_rgb(207, 95, 38)),
                            );
                        }
                    });
                    if let Some(err) = &self.chapter_preview_error {
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    } else if let Some(preview) = &self.chapter_preview {
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
                            egui::Slider::new(&mut self.text_style.line_height, 1.0..=3.0)
                                .step_by(0.1),
                        );
                        ui.label(format!("{:.1}", self.text_style.line_height));
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr(Key::ParagraphSpacing));
                        ui.add(
                            egui::Slider::new(
                                &mut self.text_style.paragraph_spacing,
                                0.0..=3.0,
                            )
                            .step_by(0.1),
                        );
                        ui.label(format!("{:.1}", self.text_style.paragraph_spacing));
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr(Key::IndentEm));
                        ui.add(
                            egui::Slider::new(&mut self.text_style.text_indent, 0.0..=4.0)
                                .step_by(0.5),
                        );
                        ui.label(format!("{:.1}", self.text_style.text_indent));
                    });
                }
                PanelIndex::CSS => {
                    ui.horizontal(|ui| {
                        ui.label(tr(Key::Template));
                        egui::ComboBox::from_id_salt("css_template")
                            .selected_text(self.text_style.css_template.to_string())
                            .show_ui(ui, |ui| {
                                for template in CssTemplate::ALL {
                                    ui.selectable_value(
                                        &mut self.text_style.css_template,
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
                            ui.text_edit_multiline(&mut self.text_style.custom_css);
                        });

                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        if ui.button(tr(Key::ImportCss)).clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("CSS", &["css"])
                                .pick_file()
                            {
                                if let Ok(content) = std::fs::read_to_string(&path) {
                                    self.text_style.custom_css = content;
                                }
                            }
                        }
                        if ui.button(tr(Key::ExportCss)).clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter("CSS", &["css"])
                                .save_file()
                            {
                                let _ = std::fs::write(&path, &self.text_style.custom_css);
                            }
                        }
                    });
                }
                PanelIndex::Font => {
                    ui.horizontal(|ui| {
                        ui.label(tr(Key::FontSize));
                        ui.add(
                            egui::Slider::new(&mut self.text_style.font_size, 10.0..=32.0)
                                .step_by(1.0),
                        );
                        ui.label(format!("{:.0}", self.text_style.font_size));
                    });
                    ui.horizontal(|ui| {
                        ui.label(tr(Key::FontColor));
                        ui.color_edit_button_srgba(&mut self.text_style.font_color);
                    });
                    ui.horizontal(|ui| {
                        if ui.button(tr(Key::ChooseFont)).clicked() {
                            if let Some(path) = FileDialog::new()
                                .add_filter(tr(Key::PanelFonts), &["ttf", "otf"])
                                .pick_file()
                            {
                                match load_font_asset(&path) {
                                    Ok(asset) => {
                                        self.font_asset = Some(asset);
                                        self.font_error = None;
                                        self.text_style.font_path =
                                            path.to_string_lossy().to_string();
                                    }
                                    Err(err) => {
                                        self.font_error = Some(t1(locale, Key::ReadFailed, err));
                                    }
                                }
                            }
                        }
                        if ui.button(tr(Key::ClearFont)).clicked() {
                            self.font_asset = None;
                            self.text_style.font_path.clear();
                        }
                    });
                    if let Some(err) = &self.font_error {
                        ui.label(egui::RichText::new(err).color(egui::Color32::RED));
                    } else if !self.text_style.font_path.is_empty() {
                        ui.label(t1(locale, Key::FontLabel, &self.text_style.font_path));
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
                                self.images.push(image_reader_from_path(locale, &path));
                            }
                        }
                        ui.label(t1(locale, Key::TotalImages, self.images.len()));
                    });

                    ui.add_space(8.0);
                    egui::ScrollArea::vertical()
                        .max_height(360.0)
                        .show(ui, |ui| {
                            if self.images.is_empty() {
                                ui.centered_and_justified(|ui| {
                                    ui.label(tr(Key::NoImages));
                                });
                            } else {
                                let mut indices_to_remove = Vec::new();
                                for (index, image) in self.images.iter_mut().enumerate() {
                                    ui.group(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(t1(locale, Key::ImageIndex, index + 1));
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    if ui.small_button(tr(Key::Delete)).clicked() {
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
                                    self.images.remove(index);
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
                                    self.images.push(image_reader_from_path(locale, &path));
                                }
                            }
                        }
                        if ui.button(tr(Key::ClearAll)).clicked() {
                            self.images.clear();
                        }
                    });
                }
                PanelIndex::PublishInfo => {
                    ui.label(tr(Key::LanguageField));
                    ui.text_edit_singleline(&mut self.book_info.language);
                    ui.label(tr(Key::Publisher));
                    ui.text_edit_singleline(&mut self.book_info.publisher);
                    ui.label(tr(Key::Isbn));
                    ui.text_edit_singleline(&mut self.book_info.isbn);
                    ui.label(tr(Key::Category));
                    ui.text_edit_singleline(&mut self.book_info.category);
                    ui.label(tr(Key::PublishDate));
                    ui.text_edit_singleline(&mut self.book_info.publish_date);
                    ui.add_space(6.0);
                    ui.label(tr(Key::Description));
                    ui.text_edit_multiline(&mut self.book_info.description);
                }
                PanelIndex::Misc => {
                    ui.label(tr(Key::OutputFolder));
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.output_path);
                        if ui.button(tr(Key::Browse)).clicked() {
                            if let Some(path) = FileDialog::new().pick_folder() {
                                self.output_path = path.to_string_lossy().to_string();
                            }
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(tr(Key::FilenameTemplate));
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(&mut self.filename_template);
                        ui.label(tr(Key::VarsHint));
                    });

                    ui.add_space(8.0);
                    ui.label(tr(Key::Current));
                    ui.label(t1(locale, Key::OutputLabel, &self.output_path));
                    ui.label(t1(locale, Key::TemplateLabel, &self.filename_template));
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

        if self.show_editor {
            egui::Window::new(tr(Key::TextEditor))
                .collapsible(false)
                .resizable(true)
                .default_width(800.0)
                .default_height(600.0)
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(tr(Key::Save)).clicked() {
                            if let Some(path) = &self.input_file.path {
                                if let Err(e) = std::fs::write(path, &self.input_file.content) {
                                    eprintln!("Save failed: {}", e);
                                }
                            }
                        }
                        if ui.button(tr(Key::Close)).clicked() {
                            self.show_editor = false;
                        }
                    });

                    ui.add_space(10.0);
                    ui.separator();
                    ui.add_space(10.0);

                    egui::ScrollArea::vertical()
                        .max_height(500.0)
                        .show(ui, |ui| {
                            ui.text_edit_multiline(&mut self.input_file.content);
                        });

                    ui.add_space(10.0);
                    ui.label(t1(locale, Key::Chars, self.input_file.content.chars().count()));
                });
        }

        if self.show_conversion_modal {
            egui::Window::new(tr(Key::ConversionResult))
                .collapsible(false)
                .resizable(false)
                .fixed_size(egui::vec2(500.0, 300.0))
                .anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.add_space(20.0);

                        if let Some(error) = &self.conversion_error {
                            ui.label(
                                egui::RichText::new(tr(Key::ConversionFailed))
                                    .size(24.0)
                                    .color(egui::Color32::RED),
                            );
                            ui.add_space(20.0);
                            ui.label(egui::RichText::new(error).size(16.0));
                        } else if let Some(output_path) = &self.conversion_result {
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
                                    if let Some(path) =
                                        std::path::PathBuf::from(output_path).parent()
                                    {
                                        let _ = open_in_file_manager(path);
                                    }
                                }
                                if ui.button(tr(Key::OpenFile)).clicked() {
                                    let _ = open_in_file_manager(std::path::Path::new(output_path));
                                }
                            });
                        }

                        ui.add_space(20.0);
                        ui.separator();
                        ui.add_space(10.0);

                        if ui.button(tr(Key::Close)).clicked() {
                            self.show_conversion_modal = false;
                            self.conversion_result = None;
                            self.conversion_error = None;
                        }
                    });
                });
        }

        if self.chapter_editor.open {
            let input = ChapterEditorInput {
                text: &self.input_file.content,
                method: self.selected_method,
                regex: &self.custom_regex_pattern,
                config_path: self.custom_regex_file.as_ref(),
            };
            self.chapter_editor.show(ctx, &input, self.locale);
        }
    }


}

fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
    let mut visuals = match mode {
        ThemeMode::Light => egui::Visuals::light(),
        ThemeMode::Dark => egui::Visuals::dark(),
    };

    let accent = match mode {
        ThemeMode::Light => egui::Color32::from_rgb(47, 125, 113),
        ThemeMode::Dark => egui::Color32::from_rgb(94, 194, 177),
    };

    visuals.selection.bg_fill = accent;
    visuals.hyperlink_color = accent;
    visuals.widgets.active.bg_fill = accent;
    visuals.widgets.hovered.bg_fill = accent.gamma_multiply(0.9);
    visuals.window_corner_radius = egui::CornerRadius::same(12);
    visuals.panel_fill = if mode == ThemeMode::Light {
        egui::Color32::from_rgb(248, 249, 250)
    } else {
        egui::Color32::from_rgb(28, 30, 33)
    };
    visuals.widgets.noninteractive.bg_fill = if mode == ThemeMode::Light {
        egui::Color32::from_rgb(243, 245, 247)
    } else {
        egui::Color32::from_rgb(36, 38, 42)
    };
    visuals.widgets.noninteractive.bg_stroke.color = if mode == ThemeMode::Light {
        egui::Color32::from_rgb(221, 225, 229)
    } else {
        egui::Color32::from_rgb(60, 64, 68)
    };

    ctx.set_visuals(visuals);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 10.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);
}

fn card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
    let fill = ui.visuals().widgets.noninteractive.bg_fill;
    let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
    egui::Frame::NONE
        .fill(fill)
        .stroke(stroke)
        .corner_radius(egui::CornerRadius::same(10))
        .inner_margin(egui::Margin::same(12))
        .show(ui, |ui| {
            ui.label(egui::RichText::new(title).size(16.0).strong());
            ui.add_space(6.0);
            add_contents(ui);
        });
}

fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let accent = ui.visuals().selection.bg_fill;
    ui.add(
        egui::Button::new(egui::RichText::new(text).color(egui::Color32::WHITE).strong())
            .fill(accent)
            .corner_radius(egui::CornerRadius::same(6)),
    )
}

fn display_or_placeholder(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

fn open_in_file_manager(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", path.to_string_lossy().as_ref()])
            .spawn()?;
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
        return Ok(());
    }
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
        return Ok(());
    }
    #[allow(unreachable_code)]
    Ok(())
}

fn cover_asset_from_reader(reader: &ImageFileReader) -> Option<ImageAsset> {
    if reader.content.is_empty() {
        return None;
    }
    let ext = reader
        .path
        .as_ref()
        .and_then(|p| p.extension())
        .and_then(|s| s.to_str())
        .unwrap_or("png")
        .to_lowercase();
    let mime = image_mime_from_extension(&ext).to_string();
    Some(ImageAsset {
        name: format!("cover.{}", ext),
        bytes: reader.content.clone(),
        mime,
        caption: None,
    })
}

fn collect_image_assets(images: &[ImageFileReader]) -> Vec<ImageAsset> {
    images
        .iter()
        .enumerate()
        .filter_map(|(idx, image)| image_asset_from_reader(image, idx))
        .collect()
}

fn image_asset_from_reader(reader: &ImageFileReader, index: usize) -> Option<ImageAsset> {
    if reader.content.is_empty() {
        return None;
    }
    let (name, mime) = if let Some(path) = &reader.path {
        let fallback_name = format!("image_{:04}.png", index + 1);
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(fallback_name.as_str());
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("png")
            .to_lowercase();
        (
            sanitize_resource_name(file_name),
            image_mime_from_extension(&ext).to_string(),
        )
    } else {
        (
            format!("image_{:04}.png", index + 1),
            "image/png".to_string(),
        )
    };

    Some(ImageAsset {
        name,
        bytes: reader.content.clone(),
        mime,
        caption: reader.caption.clone(),
    })
}

fn load_font_asset(path: &Path) -> Result<FontAsset, std::io::Error> {
    let bytes = Bytes::from(std::fs::read(path)?);
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("ttf")
        .to_lowercase();
    let mime = font_mime_from_extension(&ext).to_string();
    let family = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("CustomFont")
        .to_string();
    let file_name = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("custom-font.ttf");
    let name = sanitize_resource_name(file_name);

    Ok(FontAsset {
        name,
        family,
        bytes,
        mime,
    })
}

fn sanitize_resource_name(name: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut cleaned = name.to_string();
    for &c in &invalid_chars {
        cleaned = cleaned.replace(c, "");
    }
    cleaned.replace(' ', "_")
}

fn image_mime_from_extension(ext: &str) -> &'static str {
    match ext {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        _ => "image/png",
    }
}

fn font_mime_from_extension(ext: &str) -> &'static str {
    match ext {
        "otf" => "font/otf",
        _ => "font/ttf",
    }
}

/// 显示 egui 和 eframe 的版权信息
fn powered_by_egui_and_eframe(ui: &mut egui::Ui, locale: Locale) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label(t(locale, Key::PoweredBy));
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(t(locale, Key::PoweredByAnd));
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}

/// 从文件名中提取书名和作者
fn parse_filename_to_book_info(filename: &str) -> (String, String) {
    let stem = std::path::Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    
    let mut title = String::new();
    let mut author = String::new();
    
    // 尝试不同的分隔符
    let separators = ['_', '-', ' ', '—', '–', '·'];
    
    for sep in separators {
        if let Some((first, second)) = stem.split_once(sep) {
            title = first.trim().to_string();
            author = second.trim().to_string();
            break;
        }
    }
    
    // 如果没有找到分隔符，整个文件名作为书名
    if title.is_empty() {
        title = stem.to_string();
    }
    
    (title, author)
}

/// 读取文本文件
fn readtxt(
    ui: &mut egui::Ui,
    locale: Locale,
    input_txt: &mut TextFileReader,
    input_txt_path: &mut String,
    book_info: &mut BookInfo,
) {
    ui.horizontal(|ui| {
        if ui.button(t(locale, Key::OpenTextFile)).clicked() {
            if let Some(path) = FileDialog::new()
                .add_filter(t(locale, Key::TextFileFilter), &["txt"])
                .pick_file()
            {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        input_txt.content = content;
                        input_txt.error = None;
                        input_txt.path = Some(path.clone());
                        // txt路径显示
                        *input_txt_path = path.to_string_lossy().to_string();
                        
                        // 从文件名中自动提取书名和作者
                        if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                            let (title, author) = parse_filename_to_book_info(filename);
                            if book_info.title.trim().is_empty() {
                                book_info.title = title;
                            }
                            if book_info.author.trim().is_empty() {
                                book_info.author = author;
                            }
                        }
                    }
                    Err(e) => {
                        input_txt.error = Some(t1(locale, Key::ReadFailed, e));
                    }
                }
            }
        }

        // 显示错误信息
        if let Some(err) = &input_txt.error {
            ui.label(egui::RichText::new(err).color(egui::Color32::RED));
        } else if input_txt.path.is_some() && !input_txt_path.is_empty() {
            ui.label(input_txt_path.clone());
        } else {
            ui.label(t(locale, Key::InputTxtPlaceholder));
        }
    });
}

/// 从图片路径构建读取器
fn image_reader_from_path(locale: Locale, path: &Path) -> ImageFileReader {
    let caption = path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string());
    match std::fs::read(path) {
        Ok(content) => ImageFileReader {
            content: Bytes::from(content),
            error: None,
            path: Some(path.to_path_buf()),
            texture: None,
            caption,
        },
        Err(err) => ImageFileReader {
            content: Bytes::new(),
            error: Some(t1(locale, Key::ReadFailed, err)),
            path: Some(path.to_path_buf()),
            texture: None,
            caption,
        },
    }
}

/// 显示图片 UI
fn show_image_ui(ui: &mut egui::Ui, locale: Locale, reader: &mut ImageFileReader) {
    // 带边框的容器
    let frame = egui::Frame::new().inner_margin(4.0).corner_radius(4.0);
    frame.show(ui, |ui| {
        // 更新纹理
        reader.update_texture(ui.ctx());

        if let Some(texture) = &reader.texture {
            // 显示图片（自动缩放填充容器）
            ui.add(egui::Image::new(texture).max_width(200.0).corner_radius(10.0));
        } else if let Some(err) = &reader.error {
            // 显示错误信息
            ui.colored_label(egui::Color32::RED, err);
        } else {
            // 显示占位符
            ui.label(t(locale, Key::CoverEmpty));
        }
    });
}


#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parse_filename_splits_title_author() {
        let (title, author) = parse_filename_to_book_info("My Book - Alice.txt");
        assert_eq!(title, "My Book");
        assert_eq!(author, "Alice");
    }

    #[test]
    fn image_reader_handles_missing_file() {
        let path = Path::new("this-file-should-not-exist.png");
        let reader = image_reader_from_path(Locale::En, path);
        assert!(reader.error.is_some());
        assert!(reader.content.is_empty());
    }

    #[test]
    fn image_reader_reads_existing_file() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("reasypub-test-{suffix}.bin"));
        std::fs::write(&path, [1u8, 2, 3]).expect("write temp file");
        let reader = image_reader_from_path(Locale::En, &path);
        let _ = std::fs::remove_file(&path);
        assert!(reader.error.is_none());
        assert_eq!(reader.content.len(), 3);
        assert!(reader.path.is_some());
    }

    #[test]
    fn sanitize_resource_name_removes_invalid_chars() {
        let name = "bad:/name *.png";
        assert_eq!(sanitize_resource_name(name), "badname_.png");
    }

    #[test]
    fn image_mime_maps_extensions() {
        assert_eq!(image_mime_from_extension("jpg"), "image/jpeg");
        assert_eq!(image_mime_from_extension("jpeg"), "image/jpeg");
        assert_eq!(image_mime_from_extension("webp"), "image/webp");
        assert_eq!(image_mime_from_extension("gif"), "image/gif");
        assert_eq!(image_mime_from_extension("png"), "image/png");
    }

    #[test]
    fn font_mime_maps_extensions() {
        assert_eq!(font_mime_from_extension("otf"), "font/otf");
        assert_eq!(font_mime_from_extension("ttf"), "font/ttf");
        assert_eq!(font_mime_from_extension("unknown"), "font/ttf");
    }

    #[test]
    fn cover_asset_from_reader_uses_extension() {
        let reader = ImageFileReader {
            content: Bytes::from_static(b"img"),
            error: None,
            path: Some(PathBuf::from("cover.jpg")),
            texture: None,
            caption: None,
        };
        let asset = cover_asset_from_reader(&reader).expect("asset");
        assert_eq!(asset.name, "cover.jpg");
        assert_eq!(asset.mime, "image/jpeg");
        assert_eq!(asset.bytes.len(), 3);
    }

    #[test]
    fn image_asset_from_reader_uses_sanitized_name() {
        let reader = ImageFileReader {
            content: Bytes::from_static(b"img"),
            error: None,
            path: Some(PathBuf::from("hero image.JPG")),
            texture: None,
            caption: Some("Hero".to_string()),
        };
        let asset = image_asset_from_reader(&reader, 0).expect("asset");
        assert_eq!(asset.name, "hero_image.JPG");
        assert_eq!(asset.mime, "image/jpeg");
        assert_eq!(asset.caption.as_deref(), Some("Hero"));
    }

    #[test]
    fn display_or_placeholder_uses_fallback() {
        assert_eq!(
            display_or_placeholder("  ", "Fallback"),
            "Fallback".to_string()
        );
        assert_eq!(
            display_or_placeholder(" Title ", "Fallback"),
            "Title".to_string()
        );
    }
}
