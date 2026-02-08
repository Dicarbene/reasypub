use bytes::Bytes;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;
use std::path::{Path, PathBuf};

use crate::{BookInfo, FontAsset, ImageAsset, ImageFileReader, Key, Locale, TextFileReader, t, t1};

use super::ThemeMode;

pub(super) fn apply_theme(ctx: &egui::Context, mode: ThemeMode) {
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

pub(super) fn card(ui: &mut egui::Ui, title: &str, add_contents: impl FnOnce(&mut egui::Ui)) {
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

pub(super) fn primary_button(ui: &mut egui::Ui, text: &str) -> egui::Response {
    let accent = ui.visuals().selection.bg_fill;
    ui.add(
        egui::Button::new(
            egui::RichText::new(text)
                .color(egui::Color32::WHITE)
                .strong(),
        )
        .fill(accent)
        .corner_radius(egui::CornerRadius::same(6)),
    )
}

pub(super) fn display_or_placeholder(value: &str, fallback: &str) -> String {
    if value.trim().is_empty() {
        fallback.to_string()
    } else {
        value.trim().to_string()
    }
}

pub(super) fn open_in_file_manager(path: &Path) -> std::io::Result<()> {
    let _ = path;

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

#[cfg(not(target_arch = "wasm32"))]
fn pick_text_file(filter_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .add_filter(filter_name, &["txt"])
        .pick_file()
}

#[cfg(target_arch = "wasm32")]
fn pick_text_file(_filter_name: &str) -> Option<PathBuf> {
    None
}

pub(super) fn cover_asset_from_reader(reader: &ImageFileReader) -> Option<ImageAsset> {
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

pub(super) fn chapter_header_asset_from_reader(reader: &ImageFileReader) -> Option<ImageAsset> {
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
        name: format!("chapter-header.{}", ext),
        bytes: reader.content.clone(),
        mime,
        caption: None,
    })
}

pub(super) fn collect_image_assets(images: &[ImageFileReader]) -> Vec<ImageAsset> {
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

pub(super) fn load_font_asset(path: &Path) -> Result<FontAsset, std::io::Error> {
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
pub(super) fn powered_by_egui_and_eframe(ui: &mut egui::Ui, locale: Locale) {
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
pub(super) fn readtxt(
    ui: &mut egui::Ui,
    locale: Locale,
    input_txt: &mut TextFileReader,
    input_txt_path: &mut String,
    book_info: &mut BookInfo,
    runtime_notice: &mut Option<String>,
) {
    ui.horizontal(|ui| {
        if ui.button(t(locale, Key::OpenTextFile)).clicked() {
            if let Some(path) = pick_text_file(t(locale, Key::TextFileFilter)) {
                match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        input_txt.content = content;
                        input_txt.error = None;
                        input_txt.path = Some(path.clone());
                        *input_txt_path = path.to_string_lossy().to_string();
                        *runtime_notice = None;

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
            } else if cfg!(target_arch = "wasm32") {
                *runtime_notice = Some(t(locale, Key::DesktopOnlyAction).to_string());
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
pub(super) fn image_reader_from_path(locale: Locale, path: &Path) -> ImageFileReader {
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
pub(super) fn show_image_ui(ui: &mut egui::Ui, locale: Locale, reader: &mut ImageFileReader) {
    // 带边框的容器
    let frame = egui::Frame::new().inner_margin(4.0).corner_radius(4.0);
    frame.show(ui, |ui| {
        // 更新纹理
        reader.update_texture(ui.ctx());

        if let Some(texture) = &reader.texture {
            // 显示图片（自动缩放填充容器）
            ui.add(
                egui::Image::new(texture)
                    .max_width(200.0)
                    .corner_radius(10.0),
            );
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
    use std::path::PathBuf;
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
    fn chapter_header_asset_from_reader_uses_extension() {
        let reader = ImageFileReader {
            content: Bytes::from_static(b"img"),
            error: None,
            path: Some(PathBuf::from("header.webp")),
            texture: None,
            caption: None,
        };
        let asset = chapter_header_asset_from_reader(&reader).expect("asset");
        assert_eq!(asset.name, "chapter-header.webp");
        assert_eq!(asset.mime, "image/webp");
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
