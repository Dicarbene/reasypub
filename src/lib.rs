#![warn(clippy::all, rust_2018_idioms)]

mod app;
pub mod epubworker;

pub use app::MainApp;
use bytes::Bytes;
use egui::{ColorImage, TextureHandle};
use image::ImageError;
use once_cell::sync::Lazy;
use regex::Regex;

#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub enum ConversionMethod {
    Regex,
    CustomConfig,
    SimpleRules,
}

// 转化方式显示名称
impl std::fmt::Display for ConversionMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::SimpleRules => write!(f, "简易规则"),
            Self::Regex => write!(f, "正则表达式"),
            Self::CustomConfig => write!(f, "从文件加载"),
        }
    }
}
#[derive(serde::Deserialize, serde::Serialize, Debug, PartialEq, Clone)]
pub enum PanelIndex {
    Chapter,
    Format,
    Font,
    PublishInfo,
    CSS,
    Images,
    Misc,
}

// 面板选择
impl std::fmt::Display for PanelIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Chapter => write!(f, "章节"),
            Self::Format => write!(f, "版式"),
            Self::Font => write!(f, "字体"),
            Self::PublishInfo => write!(f, "出版信息"),
            Self::CSS => write!(f, "CSS与HTML"),
            Self::Images => write!(f, "插图"),
            Self::Misc => write!(f, "杂项"),
        }
    }
}

pub struct TextFileReader {
    content: String,
    error: Option<String>,
    path: Option<std::path::PathBuf>,
}

impl Default for TextFileReader {
    fn default() -> Self {
        Self {
            content: String::new(),
            path: None,
            error: None,
        }
    }
}

pub struct ImageFileReader {
    content: Bytes,
    error: Option<String>,
    path: Option<std::path::PathBuf>,
    texture: Option<TextureHandle>, // 存储纹理句柄
}

impl Default for ImageFileReader {
    fn default() -> Self {
        Self {
            content: Bytes::new(),
            path: None,
            error: None,
            texture: None,
        }
    }
}
impl ImageFileReader {
    /// 更新纹理（在 UI 线程调用）
    fn update_texture(&mut self, ctx: &egui::Context) {
        // 仅当内容变化时重新加载
        if !self.content.is_empty() && self.texture.is_none() {
            match self.load_texture(ctx) {
                Ok(texture) => self.texture = Some(texture),
                Err(e) => self.error = Some(format!("❌ 图片加载失败: {}", e)),
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
        Ok(ctx.load_texture("image", color_image, egui::TextureOptions::default()))
    }
}

pub struct BookInfo {
    language: String,
    publisher: String,
    isbn: String,
    category: String,
    publish_date: String,
    description: String,
}

#[derive(Debug)]
enum Pattern {
    ChineseChapter,
    EnglishChapter,
    Custom(Regex),
}

impl Pattern {
    fn to_regex(&self) -> &Regex {
        match self {
            Pattern::ChineseChapter => {
                static RE: Lazy<Regex> = Lazy::new(|| {
                    Regex::new(
                        r"\s*[第卷][0123456789一二三四五六七八九十零〇百千两]*[章回部节集卷].*",
                    )
                    .unwrap()
                });
                &RE
            }
            Pattern::EnglishChapter => {
                static RE: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"^\s*Chapter\s*[0123456789]*").unwrap());
                &RE
            }
            Pattern::Custom(re) => re,
        }
    }
}

#[derive(Debug)]
struct TextProcessor {
    pattern: Pattern,
    text: String,
}

impl TextProcessor {
    fn new(pattern: Pattern, text: String) -> Self {
        Self { pattern, text }
    }

    fn split_by_pattern(&self) -> Vec<String> {
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
}
