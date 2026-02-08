use super::*;
use crate::TocOptions;
use bytes::Bytes;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{suffix}"))
}

fn zip_entries(path: &Path) -> Vec<String> {
    let file = File::open(path).expect("open epub");
    let mut archive = ZipArchive::new(file).expect("zip");
    (0..archive.len())
        .map(|idx| archive.by_index(idx).expect("entry").name().to_string())
        .collect()
}

fn zip_read_to_string(path: &Path, suffix: &str) -> String {
    let file = File::open(path).expect("open epub");
    let mut archive = ZipArchive::new(file).expect("zip");
    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).expect("entry");
        if entry.name().ends_with(suffix) {
            let mut content = String::new();
            use std::io::Read;
            entry.read_to_string(&mut content).expect("read entry");
            return content;
        }
    }
    panic!("missing entry with suffix {suffix}");
}

fn zip_read_to_string_optional(path: &Path, suffix: &str) -> Option<String> {
    let file = File::open(path).expect("open epub");
    let mut archive = ZipArchive::new(file).expect("zip");
    for idx in 0..archive.len() {
        let mut entry = archive.by_index(idx).expect("entry");
        if entry.name().ends_with(suffix) {
            let mut content = String::new();
            use std::io::Read;
            entry.read_to_string(&mut content).expect("read entry");
            return Some(content);
        }
    }
    None
}

#[test]
fn escape_html_replaces_special_chars() {
    let input = r#"&<>"'"#;
    let out = escape_html(input);
    assert_eq!(out, "&amp;&lt;&gt;&quot;&#39;");
}

#[test]
fn split_title_line_handles_title_and_label() {
    let (label, title) = split_title_line("Chapter 1 The Start");
    assert_eq!(label, "Chapter");
    assert_eq!(title.as_deref(), Some("1 The Start"));

    let (label, title) = split_title_line("Chapter1");
    assert_eq!(label, "Chapter1");
    assert!(title.is_none());
}

#[test]
fn split_paragraphs_groups_lines() {
    let paras = split_paragraphs("a\nb\n\nc\n\n\n");
    assert_eq!(paras.len(), 2);
    assert_eq!(paras[0], vec!["a".to_string(), "b".to_string()]);
    assert_eq!(paras[1], vec!["c".to_string()]);
}

#[test]
fn render_chapter_includes_label_and_paragraphs() {
    let chapter = ChapterDraft {
        title: "Chapter 1 The Start".to_string(),
        content: "Line one\n\nLine two".to_string(),
    };
    let style = TextStyle::default();
    let html = render_chapter(
        &chapter,
        "en",
        &style,
        crate::CssTemplate::Classic,
        1,
        None,
        false,
    );
    assert!(html.contains("class=\"chapter-label\">Chapter I</div>"));
    assert!(html.contains("<h2>The Start</h2>"));
    assert!(html.contains("chapter-paragraph-first"));
    assert!(html.contains("text-indent: 0.00em;"));
    assert!(html.contains("Line one</p>"));
    assert!(html.contains("chapter-paragraph\" style=\"text-indent: 2.00em;"));
    assert!(html.contains("Line two</p>"));
}

#[test]
fn render_chapter_fantasy_header_structure() {
    let chapter = ChapterDraft {
        title: "第十二章 星落".to_string(),
        content: "内容".to_string(),
    };
    let style = TextStyle::default();
    let html = render_chapter(
        &chapter,
        "zh-CN",
        &style,
        crate::CssTemplate::Fantasy,
        12,
        None,
        false,
    );
    assert!(html.contains("Header-image-dk"));
    assert!(html.contains("images/头图.webp"));
    assert!(html.contains("class=\"nt\""));
    assert!(html.contains("CHAPTER12"));
    assert!(html.contains("class=\"ct\""));
    assert!(html.contains("images/4star.webp"));
    assert!(html.contains("images/ttl.webp"));
    assert!(html.contains("images/ttr.webp"));
}

#[test]
fn render_chapter_includes_header_image() {
    let chapter = ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    };
    let style = TextStyle::default();
    let header = ImageAsset {
        name: "chapter-header.png".to_string(),
        bytes: Bytes::from_static(b"img"),
        mime: "image/png".to_string(),
        caption: None,
    };
    let html = render_chapter(
        &chapter,
        "en",
        &style,
        crate::CssTemplate::Classic,
        1,
        Some(&header),
        true,
    );
    assert!(html.contains("chapter-head-image"));
    assert!(html.contains("fullbleed"));
    assert!(html.contains("images/chapter-header.png"));
}

#[test]
fn render_chapter_applies_marker_classes() {
    let chapter = ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "[class=note important]Hello\n\nWorld".to_string(),
    };
    let style = TextStyle {
        extra_paragraph_class: "base".to_string(),
        ..Default::default()
    };
    let html = render_chapter(
        &chapter,
        "en",
        &style,
        crate::CssTemplate::Classic,
        1,
        None,
        false,
    );
    assert!(
        html.contains("class=\"chapter-paragraph chapter-paragraph-first base note important\"")
    );
}

#[test]
fn render_gallery_includes_captions() {
    let images = vec![
        ImageAsset {
            name: "image1.png".to_string(),
            bytes: Bytes::from_static(b"123"),
            mime: "image/png".to_string(),
            caption: Some("Cover".to_string()),
        },
        ImageAsset {
            name: "image2.png".to_string(),
            bytes: Bytes::from_static(b"456"),
            mime: "image/png".to_string(),
            caption: None,
        },
    ];
    let html = render_gallery(&images, "zh-CN", "插图");
    assert!(html.contains("<figcaption>Cover</figcaption>"));
    assert!(html.contains("images/image2.png"));
}

#[test]
fn render_text_cover_includes_title_author_and_frame() {
    let book = BookInfo {
        title: "Cover Title".to_string(),
        author: "Cover Author".to_string(),
        publisher: "Pub".to_string(),
        publish_date: "2025".to_string(),
        ..Default::default()
    };
    let html = render_text_cover(&book, "en", crate::CssTemplate::Classic);
    assert!(html.contains("cover-frame"));
    assert!(html.contains("Cover Title"));
    assert!(html.contains("Cover Author"));
    assert!(html.contains("Pub · 2025"));
}

#[test]
fn build_stylesheet_includes_custom_css_and_font() {
    let style = TextStyle {
        custom_css: "p { color: red; }".to_string(),
        font_size: 18.0,
        ..Default::default()
    };
    let font = FontAsset {
        name: "custom.ttf".to_string(),
        family: "CustomFont".to_string(),
        bytes: Bytes::from_static(b"font"),
        mime: "font/ttf".to_string(),
    };

    let css = build_stylesheet(&style, Some(&font)).expect("css");
    assert!(css.contains("@font-face"));
    assert!(css.contains("CustomFont"));
    assert!(css.contains("/* === custom css === */"));
    assert!(css.contains("p { color: red; }"));
}

#[test]
fn build_stylesheet_includes_folio_overrides() {
    let style = TextStyle {
        css_template: crate::CssTemplate::Folio,
        ..Default::default()
    };
    let css = build_stylesheet(&style, None).expect("css");
    assert!(css.contains("/* === folio chapter header overrides === */"));
    assert!(css.contains("folio-divider.svg"));
}

#[test]
fn build_stylesheet_includes_fantasy_overrides() {
    let style = TextStyle {
        css_template: crate::CssTemplate::Fantasy,
        ..Default::default()
    };
    let css = build_stylesheet(&style, None).expect("css");
    assert!(css.contains("/* === fantasy chapter header overrides === */"));
    assert!(css.contains("fantasy-divider.svg"));
    assert!(css.contains("Header-image-dk"));
    assert!(css.contains("fonts/kt.ttf"));
    assert!(css.contains("body.intro2"));
}

#[test]
fn generate_filename_sanitizes_and_appends_extension() {
    let book = BookInfo {
        title: "My/Book".to_string(),
        author: "A:B".to_string(),
        ..Default::default()
    };
    let name = generate_filename(&book, "my*file");
    assert_eq!(name, "myfile.epub");
}

#[test]
fn normalize_output_dir_defaults_to_current_dir() {
    let empty = PathBuf::new();
    let current = std::env::current_dir().expect("cwd");
    let resolved = normalize_output_dir(&empty).expect("dir");
    assert_eq!(resolved, current);

    let dot = PathBuf::from(".");
    let resolved = normalize_output_dir(&dot).expect("dir");
    assert_eq!(resolved, current);
}

#[test]
fn color_to_hex_formats_uppercase() {
    let hex = color_to_hex(egui::Color32::from_rgb(255, 0, 1));
    assert_eq!(hex, "#FF0001");
}

#[test]
fn build_epub_writes_file() {
    let dir = unique_temp_dir("reasypub-epub");
    let book = BookInfo {
        title: "Test Book".to_string(),
        author: "Tester".to_string(),
        ..Default::default()
    };

    let options = EpubBuildOptions {
        book_info: book,
        output_dir: dir.clone(),
        filename_template: "test_output".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    assert!(Path::new(&output).exists());
    let meta = std::fs::metadata(&output).expect("metadata");
    assert!(meta.len() > 0);

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_contains_assets() {
    let dir = unique_temp_dir("reasypub-assets");
    let book = BookInfo {
        title: "Assets Book".to_string(),
        ..Default::default()
    };

    let cover = ImageAsset {
        name: "cover.jpg".to_string(),
        bytes: Bytes::from_static(b"cover"),
        mime: "image/jpeg".to_string(),
        caption: None,
    };
    let images = vec![ImageAsset {
        name: "gallery.png".to_string(),
        bytes: Bytes::from_static(b"img"),
        mime: "image/png".to_string(),
        caption: Some("Gallery".to_string()),
    }];
    let font = FontAsset {
        name: "custom.ttf".to_string(),
        family: "CustomFont".to_string(),
        bytes: Bytes::from_static(b"font"),
        mime: "font/ttf".to_string(),
    };

    let options = EpubBuildOptions {
        book_info: book,
        output_dir: dir.clone(),
        filename_template: "assets_output".to_string(),
        style: TextStyle::default(),
        cover: Some(cover),
        images,
        font: Some(font),
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: true,
        toc_options: TocOptions::default(),
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let entries = zip_entries(Path::new(&output));

    assert!(entries.iter().any(|name| name.ends_with("stylesheet.css")));
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("chapter_0001.xhtml"))
    );
    assert!(entries.iter().any(|name| name.ends_with("images.xhtml")));
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("images/gallery.png"))
    );
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("fonts/custom.ttf"))
    );
    assert!(entries.iter().any(|name| name.ends_with("cover.jpg")));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_contains_chapter_header_image() {
    let dir = unique_temp_dir("reasypub-header");
    let book = BookInfo {
        title: "Header Book".to_string(),
        ..Default::default()
    };

    let header = ImageAsset {
        name: "chapter-header.png".to_string(),
        bytes: Bytes::from_static(b"header"),
        mime: "image/png".to_string(),
        caption: None,
    };

    let options = EpubBuildOptions {
        book_info: book,
        output_dir: dir.clone(),
        filename_template: "header_output".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: Some(header),
        chapter_header_fullbleed: true,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let entries = zip_entries(Path::new(&output));
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("images/chapter-header.png"))
    );

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_contains_fantasy_assets() {
    let dir = unique_temp_dir("reasypub-fantasy-assets");
    let book = BookInfo {
        title: "Fantasy Assets".to_string(),
        ..Default::default()
    };

    let style = TextStyle {
        css_template: crate::CssTemplate::Fantasy,
        ..Default::default()
    };

    let options = EpubBuildOptions {
        book_info: book,
        output_dir: dir.clone(),
        filename_template: "fantasy_assets".to_string(),
        style,
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let chapters = vec![ChapterDraft {
        title: "第十二章 星落".to_string(),
        content: "内容".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let entries = zip_entries(Path::new(&output));

    for required in [
        "images/头图.webp",
        "images/头图1.webp",
        "images/4star.webp",
        "images/ttl.webp",
        "images/ttr.webp",
        "images/背景.webp",
        "images/背景1.webp",
        "images/纹理.webp",
        "images/纸纹.webp",
        "fonts/kt.ttf",
        "fonts/rbs.ttf",
        "fonts/dbs.ttf",
        "fonts/ys.ttf",
        "fonts/hyss.ttf",
        "ornaments/fantasy-divider.svg",
    ] {
        assert!(
            entries.iter().any(|name| name.ends_with(required)),
            "missing fantasy asset: {}",
            required
        );
    }

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_writes_metadata() {
    let dir = unique_temp_dir("reasypub-meta");
    let book = BookInfo {
        title: "Meta Title".to_string(),
        author: "Meta Author".to_string(),
        language: "en".to_string(),
        publisher: "Meta Pub".to_string(),
        isbn: "ISBN-123".to_string(),
        category: "Category".to_string(),
        publish_date: "2025-01-01".to_string(),
        description: "A description.".to_string(),
    };

    let options = EpubBuildOptions {
        book_info: book,
        output_dir: dir.clone(),
        filename_template: "meta_output".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let opf = zip_read_to_string(Path::new(&output), ".opf");

    assert!(opf.contains("Meta Title"));
    assert!(opf.contains("Meta Author"));
    assert!(opf.contains("Meta Pub"));
    assert!(opf.contains("ISBN-123"));
    assert!(opf.contains("Category"));
    assert!(opf.contains("2025-01-01"));
    assert!(opf.contains("A description."));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_skips_empty_metadata() {
    let dir = unique_temp_dir("reasypub-meta-empty");
    let options = EpubBuildOptions {
        book_info: BookInfo::default(),
        output_dir: dir.clone(),
        filename_template: "meta_empty".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let opf = zip_read_to_string(Path::new(&output), ".opf");

    assert!(!opf.contains("name=\"publisher\""));
    assert!(!opf.contains("name=\"identifier\""));
    assert!(!opf.contains("name=\"date\""));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_skips_images_section_when_disabled() {
    let dir = unique_temp_dir("reasypub-no-gallery");
    let options = EpubBuildOptions {
        book_info: BookInfo::default(),
        output_dir: dir.clone(),
        filename_template: "no_gallery".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: vec![ImageAsset {
            name: "gallery.png".to_string(),
            bytes: Bytes::from_static(b"img"),
            mime: "image/png".to_string(),
            caption: None,
        }],
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let entries = zip_entries(Path::new(&output));

    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("images/gallery.png"))
    );
    assert!(!entries.iter().any(|name| name.ends_with("images.xhtml")));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_uses_custom_toc_title_override() {
    let dir = unique_temp_dir("reasypub-toc-title-custom");
    let options = EpubBuildOptions {
        book_info: BookInfo {
            language: "en".to_string(),
            ..Default::default()
        },
        output_dir: dir.clone(),
        filename_template: "toc_title_custom".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: true,
            toc_title_override: "Contents (Custom)".to_string(),
            include_gallery_in_toc: true,
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let opf = zip_read_to_string(Path::new(&output), ".opf");
    assert!(opf.contains("Contents (Custom)"));

    let nav = zip_read_to_string(Path::new(&output), "nav.xhtml");
    assert!(nav.contains("Contents (Custom)"));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_uses_default_toc_title_when_override_empty() {
    let dir = unique_temp_dir("reasypub-toc-title-default");
    let options = EpubBuildOptions {
        book_info: BookInfo {
            language: "en".to_string(),
            ..Default::default()
        },
        output_dir: dir.clone(),
        filename_template: "toc_title_default".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: true,
            toc_title_override: String::new(),
            include_gallery_in_toc: true,
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let opf = zip_read_to_string(Path::new(&output), ".opf");
    assert!(opf.contains("Table Of Contents"));

    let nav = zip_read_to_string(Path::new(&output), "nav.xhtml");
    assert!(nav.contains("Table Of Contents"));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_can_exclude_gallery_from_toc() {
    let dir = unique_temp_dir("reasypub-gallery-toc-off");
    let options = EpubBuildOptions {
        book_info: BookInfo {
            language: "en".to_string(),
            ..Default::default()
        },
        output_dir: dir.clone(),
        filename_template: "gallery_toc_off".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: vec![ImageAsset {
            name: "gallery.png".to_string(),
            bytes: Bytes::from_static(b"img"),
            mime: "image/png".to_string(),
            caption: Some("Gallery".to_string()),
        }],
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: true,
        toc_options: TocOptions {
            insert_toc_page: true,
            toc_title_override: String::new(),
            include_gallery_in_toc: false,
        },
    };
    let chapters = vec![ChapterDraft {
        title: "Chapter 1".to_string(),
        content: "Hello".to_string(),
    }];

    let output = build_epub(&chapters, &options).expect("build epub");
    let entries = zip_entries(Path::new(&output));
    assert!(entries.iter().any(|name| name.ends_with("images.xhtml")));

    let nav = zip_read_to_string(Path::new(&output), "nav.xhtml");
    assert!(!nav.contains("images.xhtml"));

    let ncx = zip_read_to_string_optional(Path::new(&output), "toc.ncx").unwrap_or_default();
    assert!(!ncx.contains("images.xhtml"));

    let _ = std::fs::remove_file(&output);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn build_epub_rejects_empty_chapters() {
    let dir = unique_temp_dir("reasypub-empty");
    let options = EpubBuildOptions {
        book_info: BookInfo::default(),
        output_dir: dir,
        filename_template: "empty".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        include_images_section: false,
        toc_options: TocOptions {
            insert_toc_page: false,
            ..Default::default()
        },
    };
    let err = build_epub(&[], &options).expect_err("error");
    match err {
        BuildError::InvalidInput(_) => {}
        other => panic!("unexpected error: {other:?}"),
    }
}
