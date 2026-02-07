#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod components;
pub mod conversion;
pub mod epubworker;
pub mod i18n;

pub use i18n::{t, t1, t2, Key, Locale};

pub use app::MainApp;
use bytes::Bytes;
use egui::{ColorImage, TextureHandle};
use image::ImageError;
use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy, Hash)]
pub enum ConversionMethod {
    Regex,
    CustomConfig,
    SimpleRules,
}

impl ConversionMethod {
    pub fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::SimpleRules => t(locale, Key::MethodSimple),
            Self::Regex => t(locale, Key::MethodRegex),
            Self::CustomConfig => t(locale, Key::MethodConfig),
        }
    }
}

// 转化方式显示名称
impl std::fmt::Display for ConversionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SimpleRules => write!(f, "Simple Rules"),
            Self::Regex => write!(f, "Regex"),
            Self::CustomConfig => write!(f, "From File"),
        }
    }
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum PanelIndex {
    Chapter,
    Format,
    Font,
    PublishInfo,
    CSS,
    Images,
    Misc,
}

impl PanelIndex {
    pub fn label(self, locale: Locale) -> &'static str {
        match self {
            Self::Chapter => t(locale, Key::PanelChapters),
            Self::Format => t(locale, Key::PanelLayout),
            Self::Font => t(locale, Key::PanelFonts),
            Self::PublishInfo => t(locale, Key::PanelPublishInfo),
            Self::CSS => t(locale, Key::PanelCss),
            Self::Images => t(locale, Key::PanelImages),
            Self::Misc => t(locale, Key::PanelMisc),
        }
    }
}

// 面板选择
impl std::fmt::Display for PanelIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chapter => write!(f, "Chapters"),
            Self::Format => write!(f, "Layout"),
            Self::Font => write!(f, "Fonts"),
            Self::PublishInfo => write!(f, "Publish Info"),
            Self::CSS => write!(f, "CSS & HTML"),
            Self::Images => write!(f, "Images"),
            Self::Misc => write!(f, "Misc"),
        }
    }
}

pub fn chapter_signature(
    text: &str,
    method: ConversionMethod,
    regex: &str,
    config_path: Option<&std::path::Path>,
) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    method.hash(&mut hasher);
    regex.hash(&mut hasher);
    if let Some(path) = config_path {
        match std::fs::read(path) {
            Ok(bytes) => bytes.hash(&mut hasher),
            Err(_) => path.to_string_lossy().hash(&mut hasher),
        };
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn chapter_signature_changes_on_inputs() {
        let base = chapter_signature("text", ConversionMethod::Regex, "", None);
        let with_method = chapter_signature("text", ConversionMethod::SimpleRules, "", None);
        let with_regex = chapter_signature("text", ConversionMethod::Regex, "abc", None);
        let with_path = chapter_signature(
            "text",
            ConversionMethod::Regex,
            "",
            Some(Path::new("config.txt")),
        );
        assert_ne!(base, with_method);
        assert_ne!(base, with_regex);
        assert_ne!(base, with_path);
    }

    #[test]
    fn i18n_formatting_replaces_placeholders() {
        assert_eq!(t1(Locale::En, Key::OutputLabel, "out"), "Output: out");
        assert_eq!(t2(Locale::En, Key::ChapterIndex, 1, "Title"), "#1 Title");
    }

    #[test]
    fn text_processor_simple_rules_detects_chapters() {
        let text = "第1章 开始\n内容\n\n第2章 继续\n更多";
        let processor = TextProcessor::new(Pattern::SimpleRules, text.to_string());
        let drafts = processor.split_to_drafts();
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].title, "第1章 开始");
        assert_eq!(drafts[1].title, "第2章 继续");
    }

    #[test]
    fn text_processor_regex_keeps_preface() {
        let text = "Preface\nCHAPTER 1\nHello\nCHAPTER 2\nWorld";
        let pattern = Pattern::Custom(Regex::new(r"(?m)^CHAPTER\s+\d+").unwrap());
        let processor = TextProcessor::new(pattern, text.to_string());
        let drafts = processor.split_to_drafts();
        assert_eq!(drafts.len(), 3);
        assert_eq!(drafts[0].title, "Preface");
        assert_eq!(drafts[1].title, "CHAPTER 1");
    }

    #[test]
    fn text_processor_chinese_regex_ignores_inline_mentions() {
        let text = "第一章 开始\n内容里提到第十章但不应切分\n第二章 继续\n内容";
        let processor = TextProcessor::new(Pattern::ChineseChapter, text.to_string());
        let drafts = processor.split_to_drafts();
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].title, "第一章 开始");
        assert_eq!(drafts[1].title, "第二章 继续");
    }

    #[test]
    fn text_processor_simple_rules_accepts_preface_markers() {
        let text = "序章\n内容\n第一章 开始\n内容";
        let processor = TextProcessor::new(Pattern::SimpleRules, text.to_string());
        let drafts = processor.split_to_drafts();
        assert_eq!(drafts.len(), 2);
        assert_eq!(drafts[0].title, "序章");
        assert_eq!(drafts[1].title, "第一章 开始");
    }

    #[test]
    fn chapter_draft_from_raw_splits_title_and_body() {
        let raw = "Title line\nSecond line\nThird line";
        let draft = ChapterDraft::from_raw(raw);
        assert_eq!(draft.title, "Title line");
        assert_eq!(draft.content, "Second line\nThird line");
    }
}

#[derive(Default)]
pub struct TextFileReader {
    content: String,
    error: Option<String>,
    path: Option<std::path::PathBuf>,
}

#[derive(Default, Clone)]
pub struct ImageFileReader {
    pub content: Bytes,
    pub error: Option<String>,
    pub path: Option<std::path::PathBuf>,
    pub texture: Option<TextureHandle>, // 存储纹理句柄
    pub caption: Option<String>,
}

impl std::fmt::Debug for ImageFileReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ImageFileReader")
            .field("path", &self.path)
            .field("error", &self.error)
            .field("content_len", &self.content.len())
            .field("caption", &self.caption)
            .finish()
    }
}
impl ImageFileReader {
    /// 更新纹理（在 UI 线程调用）
    fn update_texture(&mut self, ctx: &egui::Context) {
        // 仅当内容变化时重新加载
        if !self.content.is_empty() && self.texture.is_none() {
            match self.load_texture(ctx) {
                Ok(texture) => self.texture = Some(texture),
                Err(e) => self.error = Some(format!("Image load failed: {}", e)),
            }
        }
    }

    /// 解码图片并生成纹理
    fn load_texture(&self, ctx: &egui::Context) -> Result<TextureHandle, ImageError> {
        // 解码图片
        let img = image::load_from_memory(&self.content)?;
        let rgba = img.to_rgba8();

        // 转换为 egui 需要的格式
        let size = [rgba.width() as _, rgba.height() as _];
        let pixels = rgba.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

        // 上传纹理到 GPU
        Ok(ctx.load_texture(
            self.texture_key(),
            color_image,
            egui::TextureOptions::default(),
        ))
    }

    fn texture_key(&self) -> String {
        if let Some(path) = &self.path {
            return format!("image:{}", path.to_string_lossy());
        }
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.content.hash(&mut hasher);
        format!("image:{:x}", hasher.finish())
    }
}

#[derive(Clone, Debug)]
pub struct ImageAsset {
    pub name: String,
    pub bytes: Bytes,
    pub mime: String,
    pub caption: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FontAsset {
    pub name: String,
    pub family: String,
    pub bytes: Bytes,
    pub mime: String,
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct BookInfo {
    pub author: String,
    pub title: String,
    pub language: String,
    pub publisher: String,
    pub isbn: String,
    pub category: String,
    pub publish_date: String,
    pub description: String,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
pub enum CssTemplate {
    Classic,
    Modern,
    Clean,
    Elegant,
    Folio,
    Fantasy,
    Minimal,
}

impl CssTemplate {
    pub const ALL: [CssTemplate; 7] = [
        CssTemplate::Classic,
        CssTemplate::Modern,
        CssTemplate::Clean,
        CssTemplate::Elegant,
        CssTemplate::Folio,
        CssTemplate::Fantasy,
        CssTemplate::Minimal,
    ];

    pub fn css(&self) -> &'static str {
        match self {
            CssTemplate::Classic => r#"
body {
  font-family: "Latin Modern Roman", "CMU Serif", "STIX Two Text", "Source Serif 4", "Garamond",
    "Georgia", "Times New Roman", serif;
  font-kerning: normal;
  font-variant-ligatures: common-ligatures;
  font-variant-numeric: oldstyle-nums;
  text-rendering: optimizeLegibility;
}
h1, h2, h3, h4 {
  font-family: "Latin Modern Roman", "CMU Serif", "STIX Two Text", "Source Serif 4", "Garamond",
    "Georgia", serif;
  letter-spacing: 0.08em;
}
h2 { font-weight: 600; text-align: center; margin-top: 2em; margin-bottom: 1.2em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
  letter-spacing: 0.01em;
  word-spacing: 0.02em;
}
.chapter-label {
  font-size: 0.75em;
  font-variant: small-caps;
  text-transform: none;
  letter-spacing: 0.4em;
  text-align: center;
  color: #444444;
  margin-top: 0.9em;
  margin-bottom: 0.3em;
}
"#,
            CssTemplate::Modern => r#"
body {
  font-family: "Source Serif 4", "Noto Serif", "Georgia", "Times New Roman", serif;
  font-kerning: normal;
}
h1, h2, h3, h4 {
  font-family: "Source Sans 3", "Helvetica Neue", "Helvetica", "Arial", sans-serif;
  letter-spacing: 0.08em;
}
h2 { font-weight: 500; text-align: center; margin-top: 1.5em; margin-bottom: 0.95em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
}
.chapter-label {
  font-size: 0.78em;
  color: #2f5d50;
  letter-spacing: 0.25em;
  text-align: center;
  margin-top: 0.6em;
  margin-bottom: 0.2em;
}
"#,
            CssTemplate::Clean => r#"
body { font-family: "Georgia", "Times New Roman", serif; }
h2 { letter-spacing: 0.04em; font-weight: 600; text-align: center; margin-top: 1.6em; margin-bottom: 1em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
}
.chapter-label { font-size: 0.8em; color: #3b3b3b; letter-spacing: 0.2em; text-align: center; }
"#,
            CssTemplate::Elegant => r#"
body {
  font-family: "Garamond", "Palatino", "Times New Roman", serif;
  font-variant-ligatures: common-ligatures;
  font-variant-numeric: oldstyle-nums;
  text-rendering: optimizeLegibility;
}
h1, h2, h3, h4 {
  font-family: "Garamond", "Palatino", "Times New Roman", serif;
  letter-spacing: 0.06em;
}
h2 { font-weight: 600; text-align: center; margin-top: 2.2em; margin-bottom: 1.2em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
  letter-spacing: 0.02em;
  word-spacing: 0.03em;
}
.chapter-label {
  font-size: 0.75em;
  color: #5a4a3b;
  font-variant: small-caps;
  text-transform: none;
  letter-spacing: 0.32em;
  text-align: center;
  margin-top: 1em;
  margin-bottom: 0.35em;
}
"#,
            CssTemplate::Folio => r#"
body {
  font-family: "Baskerville", "Garamond", "Palatino", "Times New Roman", serif;
  font-variant-ligatures: common-ligatures;
  font-variant-numeric: oldstyle-nums;
  text-rendering: optimizeLegibility;
}
h1, h2, h3, h4 {
  font-family: "Baskerville", "Garamond", "Palatino", "Times New Roman", serif;
  letter-spacing: 0.1em;
}
h2 { font-weight: 600; text-align: center; margin-top: 2.2em; margin-bottom: 1.2em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
  letter-spacing: 0.015em;
  word-spacing: 0.04em;
}
.chapter-label {
  font-size: 0.76em;
  color: #5a4a3b;
  font-variant: small-caps;
  letter-spacing: 0.32em;
  text-align: center;
  margin-top: 0.8em;
  margin-bottom: 0.3em;
}
"#,
            CssTemplate::Fantasy => r#"
body {
  font-family: "kt", "KaiTi", "STKaiti", "Kaiti SC", "Baskerville", "Garamond", serif;
  font-variant-ligatures: common-ligatures;
  text-rendering: optimizeLegibility;
  color: #2a1e14;
  letter-spacing: 0.02em;
}
h1, h2, h3, h4 {
  font-family: "rbs", "dbs", "KaiTi", "STKaiti", "Kaiti SC", "Garamond", serif;
  letter-spacing: 0.16em;
}
h2 { font-weight: 600; text-align: center; margin-top: 2.4em; margin-bottom: 1.4em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
  letter-spacing: 0.02em;
  word-spacing: 0.04em;
}
.chapter-label {
  font-size: 0.78em;
  color: #a66c44;
  font-variant: small-caps;
  letter-spacing: 0.42em;
  text-align: center;
  margin-top: 0.9em;
  margin-bottom: 0.35em;
}
"#,
            CssTemplate::Minimal => r#"
body { font-family: "Times New Roman", "Georgia", serif; }
h1, h2, h3, h4 { font-family: "Times New Roman", "Georgia", serif; letter-spacing: 0.04em; }
h2 { font-weight: normal; text-align: center; margin-top: 1.4em; margin-bottom: 0.9em; }
p {
  text-align: justify;
  text-justify: inter-ideograph;
  line-break: strict;
  word-break: break-word;
  hyphens: auto;
  -webkit-hyphens: auto;
  -moz-hyphens: auto;
}
.chapter-label { font-size: 0.76em; color: #2f2f2f; letter-spacing: 0.22em; text-align: center; }
"#,
        }
    }
}

impl std::fmt::Display for CssTemplate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CssTemplate::Classic => write!(f, "Classic"),
            CssTemplate::Modern => write!(f, "Modern"),
            CssTemplate::Clean => write!(f, "Clean"),
            CssTemplate::Elegant => write!(f, "Elegant"),
            CssTemplate::Folio => write!(f, "Folio"),
            CssTemplate::Fantasy => write!(f, "Fantasy"),
            CssTemplate::Minimal => write!(f, "Minimal"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct TextStyle {
    pub line_height: f32,
    pub paragraph_spacing: f32,
    pub text_indent: f32,
    pub font_size: f32,
    pub font_color: egui::Color32,
    pub font_path: String,
    pub css_template: CssTemplate,
    pub custom_css: String,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            line_height: 1.5,
            paragraph_spacing: 1.0,
            text_indent: 2.0,
            font_size: 16.0,
            font_color: egui::Color32::BLACK,
            font_path: String::new(),
            css_template: CssTemplate::Classic,
            custom_css: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ChapterDraft {
    pub title: String,
    pub content: String,
}

impl ChapterDraft {
    pub fn from_raw(raw: &str) -> Self {
        let mut lines = raw.lines();
        let title = lines.next().unwrap_or("Untitled Chapter").trim().to_string();
        let content = lines.collect::<Vec<_>>().join("\n");
        Self { title, content }
    }
}

#[derive(Debug, Clone)]
pub enum Pattern {
    ChineseChapter,
    EnglishChapter,
    SimpleRules,
    Custom(Regex),
}

impl Pattern {
    pub fn to_regex(&self) -> &Regex {
        match self {
            Pattern::ChineseChapter => {
                static RE: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(
                        r"(?m)^\s*(?:第[0-9０-９一二三四五六七八九十零〇○百千万两]+[章节回部节集卷][^\n]*|卷[0-9０-９一二三四五六七八九十零〇○百千万两]+[^\n]*|(?:序章|序言|序|楔子|引子|前言|后记|尾声|终章|番外|外传|附录)[^\n]*)",
                    )
                    .unwrap()
                });
                &RE
            }
            Pattern::EnglishChapter => {
                static RE: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"(?m)^\s*Chapter\s*[0-9]+[^\n]*").unwrap());
                &RE
            }
            Pattern::SimpleRules => {
                // SimpleRules 不使用正则，返回一个匹配所有内容的正则作为后备
                static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r".*").unwrap());
                &RE
            }
            Pattern::Custom(re) => re,
        }
    }
}

#[derive(Debug)]
pub(crate) struct TextProcessor {
    pattern: Pattern,
    text: String,
}

impl TextProcessor {
    pub(crate) fn new(pattern: Pattern, text: String) -> Self {
        Self { pattern, text }
    }

    pub(crate) fn split_to_drafts(&self) -> Vec<ChapterDraft> {
        self.split_by_pattern()
            .into_iter()
            .map(|raw| ChapterDraft::from_raw(&raw))
            .collect()
    }

    fn split_by_pattern(&self) -> Vec<String> {
        match &self.pattern {
            Pattern::SimpleRules => self.split_by_simple_rules(),
            _ => self.split_by_regex(),
        }
    }

    fn split_by_regex(&self) -> Vec<String> {
        let re = self.pattern.to_regex();
        let clean = Regex::new(r"[\r\u{3000}]+").unwrap();
        let t = clean.replace_all(&self.text, "").trim().to_string();

        let mut result = Vec::new();
        let mut last_end = 0;

        // 遍历所有匹配的章节标题
        for mat in re.find_iter(&t) {
            let start = mat.start();
            let end = mat.end();

            // 1. 如果是第一个章节，先检查前面是否有非章节内容（比如前言）
            if result.is_empty() && start > 0 {
                let preface = t[..start].trim();
                if !preface.is_empty() {
                    result.push(preface.to_string());
                }
            }

            // 2. 找到下一个章节标题的位置（或文本末尾）
            let next_match = re
                .find(&t[end..])
                .map(|m| end + m.start())
                .unwrap_or(t.len());

            // 3. 提取当前章节（标题 + 内容）
            let chapter = t[start..next_match].trim();
            if !chapter.is_empty() {
                result.push(chapter.to_string());
            }

            last_end = next_match;
        }

        // 4. 如果最后还有剩余内容（比如没有章节的结尾部分）
        if last_end < t.len() {
            let remaining = t[last_end..].trim();
            if !remaining.is_empty() {
                result.push(remaining.to_string());
            }
        }

        result
    }

    fn split_by_simple_rules(&self) -> Vec<String> {
        let clean = Regex::new(r"[\r\u{3000}]+").unwrap();
        let t = clean.replace_all(&self.text, "").trim().to_string();

        let mut result = Vec::new();
        let lines: Vec<&str> = t.lines().collect();
        let mut current_chapter = String::new();

        for line in lines {
            let trimmed = line.trim();
            if Self::is_chapter_title_line(trimmed) {
                // 如果当前章节不为空，先保存
                if !current_chapter.trim().is_empty() {
                    result.push(current_chapter.trim().to_string());
                }
                // 开始新章节
                current_chapter = format!("{}\n", trimmed);
            } else {
                // 添加到当前章节
                current_chapter.push_str(&format!("{}\n", trimmed));
            }
        }

        // 添加最后一个章节
        if !current_chapter.trim().is_empty() {
            result.push(current_chapter.trim().to_string());
        }

        result
    }

    fn is_chapter_title_line(line: &str) -> bool {
        if line.is_empty() {
            return false;
        }
        let len = line.chars().count();
        if len > 60 {
            return false;
        }

        let specials = [
            "序章", "序言", "序", "楔子", "引子", "前言", "后记", "尾声", "终章", "番外", "外传", "附录",
        ];
        if specials.iter().any(|&s| line == s || line.starts_with(s)) {
            return true;
        }

        if line.starts_with('第') {
            let markers = ['章', '回', '节', '集', '卷', '部', '篇'];
            if markers.iter().any(|&m| line.contains(m)) {
                return true;
            }
        }

        if line.starts_with('卷') {
            let numerals = "一二三四五六七八九十零〇○百千万两";
            let has_num = line.chars().skip(1).any(|ch| {
                ch.is_ascii_digit()
                    || ('０'..='９').contains(&ch)
                    || numerals.contains(ch)
            });
            return has_num;
        }

        false
    }
}
