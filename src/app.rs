use crate::components::chapter_editor::ChapterEditorState;
use crate::conversion::{ConversionFacade, ConversionRequest};
use crate::{
    t, t1, BookInfo, ConversionMethod, FontAsset, ImageFileReader, Key, Locale, PanelIndex,
    TextFileReader, TextStyle,
};
use regex::Regex;
use std::path::{Path, PathBuf};

mod app_helpers;
mod ui;
use app_helpers::{
    apply_theme, chapter_header_asset_from_reader, collect_image_assets, cover_asset_from_reader,
    load_font_asset,
};

/// 我们派生 Deserialize/Serialize 以便在关闭时持久化应用状态
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // 如果添加新字段，在反序列化旧状态时给予默认值
pub struct MainApp {
    // 基础数据结构体
    input_txt_path: String, // 输入文本文件路径
    input_image_path: String, // 输入图片路径
    chapter_header_image_path: String, // 章头图路径
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
    chapter_header_image: ImageFileReader, // 章头图读取器
    #[serde(skip)]
    font_asset: Option<FontAsset>,
    #[serde(skip)]
    font_error: Option<String>,
    chapter_header_fullbleed: bool, // 章头图全宽/全屏
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
            chapter_header_image_path: String::new(),
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
            chapter_header_image: ImageFileReader::default(),
            font_asset: None,
            font_error: None,
            chapter_header_fullbleed: false,
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
        let chapter_header_image = chapter_header_asset_from_reader(&self.chapter_header_image);
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
            chapter_header_image,
            chapter_header_fullbleed: self.chapter_header_fullbleed,
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
        ui::top_panel(self, ctx);
        ui::side_nav(self, ctx);
        ui::preview_panel(self, ctx);
        ui::central_panel(self, ctx);
        ui::dialogs(self, ctx);
    }


}
