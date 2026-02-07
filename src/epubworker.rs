use std::fs::{self, File};
use std::io::Cursor;
use std::path::PathBuf;

use epub_builder::{EpubBuilder, EpubContent, MetadataOpf, ReferenceType, ZipLibrary};

use crate::{BookInfo, ChapterDraft, CssTemplate, FontAsset, ImageAsset, TextStyle};

#[derive(Debug)]
pub enum BuildError {
    Io(std::io::Error),
    Epub(epub_builder::Error),
    InvalidInput(String),
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::Io(err) => write!(f, "IO error: {}", err),
            BuildError::Epub(err) => write!(f, "EPUB error: {}", err),
            BuildError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<std::io::Error> for BuildError {
    fn from(err: std::io::Error) -> Self {
        BuildError::Io(err)
    }
}

impl From<epub_builder::Error> for BuildError {
    fn from(err: epub_builder::Error) -> Self {
        BuildError::Epub(err)
    }
}

pub struct EpubBuildOptions {
    pub book_info: BookInfo,
    pub output_dir: PathBuf,
    pub filename_template: String,
    pub style: TextStyle,
    pub cover: Option<ImageAsset>,
    pub images: Vec<ImageAsset>,
    pub font: Option<FontAsset>,
    pub include_images_section: bool,
    pub inline_toc: bool,
}

pub fn build_epub(
    chapters: &[ChapterDraft],
    options: &EpubBuildOptions,
) -> Result<String, BuildError> {
    if chapters.is_empty() {
        return Err(BuildError::InvalidInput("No chapters provided.".to_string()));
    }

    let output_dir = normalize_output_dir(&options.output_dir)?;
    fs::create_dir_all(&output_dir)?;

    let filename = generate_filename(&options.book_info, &options.filename_template);
    let outpath = output_dir.join(&filename);
    let writer = File::create(&outpath)?;

    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    add_optional_metadata(&mut builder, "author", &options.book_info.author)?;
    add_optional_metadata(&mut builder, "title", &options.book_info.title)?;
    add_optional_metadata(&mut builder, "lang", &options.book_info.language)?;
    let lang = options.book_info.language.trim();
    let toc_name = if lang.is_empty() || lang.starts_with("zh") {
        "目录"
    } else {
        "Table Of Contents"
    };
    add_optional_metadata(&mut builder, "toc_name", toc_name)?;
    add_optional_metadata(&mut builder, "subject", &options.book_info.category)?;
    add_optional_metadata(&mut builder, "description", &options.book_info.description)?;
    add_optional_meta_tag(&mut builder, "publisher", &options.book_info.publisher);
    add_optional_meta_tag(&mut builder, "identifier", &options.book_info.isbn);
    add_optional_meta_tag(&mut builder, "date", &options.book_info.publish_date);

    let stylesheet = build_stylesheet(&options.style, options.font.as_ref())?;
    builder.stylesheet(Cursor::new(stylesheet))?;

    if let Some(cover) = &options.cover {
        builder.add_cover_image(&cover.name, cover.bytes.as_ref(), &cover.mime)?;
    }

    if let Some(font) = &options.font {
        let path = format!("fonts/{}", font.name);
        builder.add_resource(path, Cursor::new(font.bytes.clone()), &font.mime)?;
    }

    if matches!(options.style.css_template, CssTemplate::Folio) {
        builder.add_resource(
            "ornaments/folio-divider.svg".to_string(),
            Cursor::new(folio_divider_svg().as_bytes()),
            "image/svg+xml",
        )?;
    }
    if matches!(options.style.css_template, CssTemplate::Fantasy) {
        builder.add_resource(
            "ornaments/fantasy-divider.svg".to_string(),
            Cursor::new(fantasy_divider_svg().as_bytes()),
            "image/svg+xml",
        )?;
        add_fantasy_assets(&mut builder)?;
    }

    for image in &options.images {
        let path = format!("images/{}", image.name);
        builder.add_resource(path, Cursor::new(image.bytes.clone()), &image.mime)?;
    }

    let language = if options.book_info.language.trim().is_empty() {
        "zh-CN"
    } else {
        options.book_info.language.trim()
    };

    if options.cover.is_none() {
        let cover_html = render_text_cover(&options.book_info, language, options.style.css_template);
        builder.add_content(
            EpubContent::new("cover.xhtml", cover_html.as_bytes())
                .reftype(ReferenceType::Cover),
        )?;
    }

    if options.inline_toc {
        builder.inline_toc();
    }

    for (index, chapter) in chapters.iter().enumerate() {
        let html = render_chapter(
            chapter,
            language,
            options.style.text_indent,
            options.style.css_template,
            index + 1,
        );
        let filename = format!("chapter_{:04}.xhtml", index + 1);
        builder.add_content(
            EpubContent::new(filename, html.as_bytes())
                .title(chapter.title.as_str())
                .reftype(ReferenceType::Text),
        )?;
    }

    if options.include_images_section && !options.images.is_empty() {
        let gallery_title = gallery_title(language);
        let html = render_gallery(&options.images, language, gallery_title);
        builder.add_content(
            EpubContent::new("images.xhtml", html.as_bytes())
                .title(gallery_title)
                .reftype(ReferenceType::Text),
        )?;
    }

    builder.generate(writer)?;

    Ok(outpath.display().to_string())
}

fn add_optional_metadata(
    builder: &mut EpubBuilder<ZipLibrary>,
    key: &str,
    value: &str,
) -> Result<(), BuildError> {
    if !value.trim().is_empty() {
        builder.metadata(key, value)?;
    }
    Ok(())
}

fn add_optional_meta_tag(builder: &mut EpubBuilder<ZipLibrary>, name: &str, value: &str) {
    if value.trim().is_empty() {
        return;
    }
    builder.add_metadata_opf(Box::new(MetadataOpf {
        name: name.to_string(),
        content: value.trim().to_string(),
    }));
}

fn add_fantasy_assets(builder: &mut EpubBuilder<ZipLibrary>) -> Result<(), BuildError> {
    let image_assets = [
        ("assets/fantasy/images/头图.webp", "images/头图.webp"),
        ("assets/fantasy/images/头图1.webp", "images/头图1.webp"),
        ("assets/fantasy/images/4star.webp", "images/4star.webp"),
        ("assets/fantasy/images/ttl.webp", "images/ttl.webp"),
        ("assets/fantasy/images/ttr.webp", "images/ttr.webp"),
        ("assets/fantasy/images/背景.webp", "images/背景.webp"),
        ("assets/fantasy/images/背景1.webp", "images/背景1.webp"),
        ("assets/fantasy/images/纹理.webp", "images/纹理.webp"),
        ("assets/fantasy/images/纸纹.webp", "images/纸纹.webp"),
    ];
    for (source, dest) in image_assets {
        let bytes = fs::read(source)?;
        builder.add_resource(dest.to_string(), Cursor::new(bytes), "image/webp")?;
    }

    let font_assets = [
        ("assets/fantasy/fonts/kt.ttf", "fonts/kt.ttf"),
        ("assets/fantasy/fonts/rbs.ttf", "fonts/rbs.ttf"),
        ("assets/fantasy/fonts/dbs.ttf", "fonts/dbs.ttf"),
        ("assets/fantasy/fonts/ys.ttf", "fonts/ys.ttf"),
        ("assets/fantasy/fonts/hyss.ttf", "fonts/hyss.ttf"),
    ];
    for (source, dest) in font_assets {
        let bytes = fs::read(source)?;
        builder.add_resource(dest.to_string(), Cursor::new(bytes), "font/ttf")?;
    }

    Ok(())
}

fn build_stylesheet(style: &TextStyle, font: Option<&FontAsset>) -> Result<String, BuildError> {
    let base_css = fs::read_to_string("assets/book/book.css").unwrap_or_default();
    let mut css = String::new();
    css.push_str(&base_css);
    css.push_str("\n\n/* === template === */\n");
    css.push_str(style.css_template.css());

    let text_color = color_to_hex(style.font_color);
    css.push_str("\n\n/* === typography === */\n");
    css.push_str(&format!(
        "body {{ color: {}; font-size: {}px; }}\n",
        text_color, style.font_size
    ));
    css.push_str(&format!(
        "p {{ line-height: {}em; margin: 0 0 {}em 0; text-indent: {}em; font-size: {}px; color: {}; }}\n",
        style.line_height,
        style.paragraph_spacing,
        style.text_indent,
        style.font_size,
        text_color
    ));
    css.push_str(&format!(
        "h1 + p, h2 + p, h3 + p, h4 + p, h5 + p, h6 + p {{ text-indent: {}em; }}\n",
        style.text_indent
    ));

    css.push_str("\n\n/* === cover === */\n");
    css.push_str(".cover-page { text-align: center; page-break-after: always; }\n");
    css.push_str(".cover-frame { position: relative; margin: 2.8em 1.6em; padding: 2.4em 1.8em; border: 2px double #6b5b4b; background: #fbf8f2; }\n");
    css.push_str(".cover-title { font-size: 2.2em; letter-spacing: 0.12em; line-height: 1.2; margin: 0.6em 0 0.2em; }\n");
    css.push_str(".cover-subtitle { font-size: 1.05em; letter-spacing: 0.08em; color: #6b5b4b; margin: 0.2em 0 0.6em; }\n");
    css.push_str(".cover-author { font-size: 1.1em; letter-spacing: 0.2em; margin: 1.2em 0 0.2em; }\n");
    css.push_str(".cover-meta { font-size: 0.85em; letter-spacing: 0.2em; color: #6b5b4b; margin-top: 1.4em; }\n");
    css.push_str(".cover-ornament { height: 1.8em; width: 70%; margin: 0.8em auto; border-top: 1px solid #6b5b4b; border-bottom: 1px solid #cbbda9; }\n");

    css.push_str("\n\n/* === chapter header === */\n");
    css.push_str(".chapter { page-break-before: always; break-before: page; }\n");
    css.push_str(
        ".chapter-header { text-align: center; margin: 2.4em 0 2.1em; position: relative; padding: 0.8em 0 1em; background: linear-gradient(#8a7a66, #8a7a66) left top/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) left top/1px 1.4em no-repeat, linear-gradient(#8a7a66, #8a7a66) right top/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) right top/1px 1.4em no-repeat, linear-gradient(#8a7a66, #8a7a66) left bottom/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) left bottom/1px 1.4em no-repeat, linear-gradient(#8a7a66, #8a7a66) right bottom/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) right bottom/1px 1.4em no-repeat; }\n",
    );
    css.push_str(
        ".chapter-header::before, .chapter-header::after { content: \"\"; position: absolute; top: 0.25em; width: 0.45em; height: 0.45em; border: 1px solid #8a7a66; background: transparent; transform: rotate(45deg); }\n",
    );
    css.push_str(
        ".chapter-header::before { left: 0.35em; }\n",
    );
    css.push_str(
        ".chapter-header::after { right: 0.35em; }\n",
    );
    css.push_str(
        ".chapter-header h2 { display: inline-block; padding: 0 0.7em; position: relative; }\n",
    );
    css.push_str(
        ".chapter-ornament { border-top: 1px solid #6b5b4b; border-bottom: 1px solid #c0b5a4; height: 0; margin: 0.9em auto; width: 54%; text-align: center; }\n",
    );
    css.push_str(
        ".chapter-ornament::after { content: \"\"; display: inline-block; margin-top: -0.75em; width: 0.3em; height: 0.3em; border: 1px solid #6b5b4b; border-radius: 50%; background: transparent; box-shadow: -1.2em 0 0 #6b5b4b, 1.2em 0 0 #6b5b4b, -2.4em 0 0 #c0b5a4, 2.4em 0 0 #c0b5a4, -3.6em 0 0 #6b5b4b, 3.6em 0 0 #6b5b4b; }\n",
    );
    css.push_str(
        ".chapter-label { string-set: chapter content(); }\n",
    );
    css.push_str(
        "@page { @top-center { content: string(chapter); font-family: \"Garamond\", \"Times New Roman\", serif; font-size: 0.7em; letter-spacing: 0.2em; color: #6b5b4b; } }\n",
    );
    css.push_str(
        "@page :first { @top-center { content: normal; } }\n",
    );
    css.push_str(
        ".chapter-paragraph-first { text-indent: 0 !important; }\n",
    );
    css.push_str(
        ".chapter-paragraph-first::first-letter { float: left; font-size: 3.2em; line-height: 0.85; padding: 0.04em 0.1em 0 0; font-weight: 600; color: #5a4a3b; }\n",
    );

    if matches!(style.css_template, crate::CssTemplate::Folio) {
        css.push_str("\n\n/* === folio chapter header overrides === */\n");
        css.push_str(".chapter-header { margin: 2.4em 0 2.1em; padding: 0.9em 0 1.1em; border-top: 1px solid #6b5b4b; border-bottom: 1px solid #cbbda9; background: #fbf8f2; }\n");
        css.push_str(".chapter-ornament { border: none; height: 1.7em; width: 62%; margin: 0.75em auto; background: url(\"ornaments/folio-divider.svg\") center / 62% auto no-repeat; }\n");
        css.push_str(".chapter-ornament::after { display: none; }\n");
        css.push_str(".chapter-label { letter-spacing: 0.35em; color: #5a4a3b; }\n");
        css.push_str("\n\n/* === folio cover === */\n");
        css.push_str(".cover-frame { border-color: #6b5b4b; background: #fcfaf6; box-shadow: inset 0 0 0 3px rgba(107,91,75,0.08); }\n");
        css.push_str(".cover-frame::before { content: \"\"; position: absolute; inset: 0.9em; border: 1px solid rgba(107,91,75,0.28); }\n");
        css.push_str(".cover-ornament { border: none; height: 2.0em; width: 72%; margin: 0.95em auto; background: url(\"ornaments/folio-divider.svg\") center / 70% auto no-repeat; }\n");
        css.push_str(".cover-title { letter-spacing: 0.2em; font-size: 2.35em; }\n");
        css.push_str(".cover-author { letter-spacing: 0.28em; }\n");
        css.push_str(".cover-meta { letter-spacing: 0.22em; }\n");
    }

    if matches!(style.css_template, crate::CssTemplate::Fantasy) {
        css.push_str("\n\n/* === fantasy assets === */\n");
        css.push_str("@font-face { font-family: \"kt\"; src: url(\"fonts/kt.ttf\"); }\n");
        css.push_str("@font-face { font-family: \"rbs\"; src: url(\"fonts/rbs.ttf\"); }\n");
        css.push_str("@font-face { font-family: \"dbs\"; src: url(\"fonts/dbs.ttf\"); }\n");
        css.push_str("@font-face { font-family: \"ys\"; src: url(\"fonts/ys.ttf\"); }\n");
        css.push_str("@font-face { font-family: \"hyss\"; src: url(\"fonts/hyss.ttf\"); }\n");
        css.push_str(&format!(
            "p {{ duokan-text-indent: {}em; }}\n",
            style.text_indent
        ));
        css.push_str("body.intro { background-image: url(\"images/背景.webp\"); background-size: cover; background-position: center; }\n");
        css.push_str("body.intro1 { background-image: url(\"images/背景1.webp\"); background-size: cover; background-position: center; }\n");
        css.push_str("body.intro2 { background-image: url(\"images/纹理.webp\"); background-repeat: repeat; background-size: 100% auto; }\n");
        css.push_str("body.cover-fantasy { background-image: url(\"images/背景.webp\"); background-size: cover; background-position: center; background-repeat: no-repeat; }\n");

        css.push_str("\n\n/* === fantasy chapter header (duokan) === */\n");
        css.push_str(".Header-image-dk { text-align: right; text-indent: 0em; duokan-text-indent: 0em; margin: 0 0 -30% 0; margin-left: auto; page-break-before: always; duokan-bleed: lefttopright; }\n");
        css.push_str(".Header-image-dk img, img.width100 { width: 100%; max-width: 100%; border: none; box-shadow: none; background: none; }\n");
        css.push_str(".chapter-title-hidden { display: none; }\n");
        css.push_str("p.nt { font-family: \"dbs\"; color: #a66c44; font-weight: normal; font-size: 1em; margin: 4px 0; duokan-text-indent: 0em; text-indent: 0em; text-align: center; }\n");
        css.push_str("p.et { font-family: \"rbs\"; color: #bca68a; font-weight: normal; font-size: 0.8em; margin: 4px 0; duokan-text-indent: 0em; text-indent: 0em; text-align: center; letter-spacing: 0.6em; }\n");
        css.push_str("p.ct { font-family: \"rbs\"; color: #7a3a24; font-weight: normal; font-size: 1.3em; margin: 4px 0 3em; duokan-text-indent: 0em; text-indent: 0em; text-align: center; }\n");
        css.push_str("img.emoji { height: 0.9em; vertical-align: -1px; border: none; box-shadow: none; background: none; }\n");
        css.push_str("img.emoji1 { height: 0.6em; vertical-align: 0; border: none; box-shadow: none; background: none; }\n");
        css.push_str("div.tip { width: 90%; background-image: url(\"images/纸纹.webp\"); background-size: 100% auto; border-radius: 8px; padding: 8px; margin: 1em auto; }\n");
        css.push_str(".tip p { text-indent: 0em; duokan-text-indent: 0em; }\n");

        css.push_str("\n\n/* === fantasy chapter header overrides === */\n");
        css.push_str(".chapter-header { margin: 2.6em 0 2.2em; padding: 1.0em 0 1.2em; border-top: 1px solid #a66c44; border-bottom: 1px solid #bca68a; background: linear-gradient(#fbf8f2, #f5ede2); }\n");
        css.push_str(".chapter-ornament { border: none; height: 1.9em; width: 68%; margin: 0.85em auto; background: url(\"ornaments/fantasy-divider.svg\") center / 70% auto no-repeat; }\n");
        css.push_str(".chapter-ornament::after { display: none; }\n");
        css.push_str(".chapter-label { letter-spacing: 0.45em; color: #a66c44; }\n");

        css.push_str("\n\n/* === fantasy cover === */\n");
        css.push_str(".cover-frame { border-color: #a66c44; background: #f6efe3 url(\"images/纸纹.webp\") center / cover no-repeat; box-shadow: inset 0 0 0 3px rgba(166,108,68,0.14); }\n");
        css.push_str(".cover-frame::before { content: \"\"; position: absolute; inset: 0.8em; border: 1px solid rgba(166,108,68,0.32); border-radius: 2px; }\n");
        css.push_str(".cover-ornament { border: none; height: 2.1em; width: 74%; margin: 1.0em auto; background: url(\"ornaments/fantasy-divider.svg\") center / 72% auto no-repeat; }\n");
        css.push_str(".cover-title { letter-spacing: 0.22em; font-size: 2.45em; color: #7a3a24; text-shadow: 0 1px 0 #fff6ea; }\n");
        css.push_str(".cover-subtitle { letter-spacing: 0.26em; color: #a66c44; }\n");
        css.push_str(".cover-author { letter-spacing: 0.34em; color: #3c2a1c; }\n");
        css.push_str(".cover-meta { letter-spacing: 0.26em; color: #6b5b4b; }\n");
    }

    if let Some(font_asset) = font {
        css.push_str("\n\n/* === embedded font === */\n");
        css.push_str(&format!(
            "@font-face {{ font-family: \"{}\"; src: url(\"fonts/{}\"); }}\n",
            font_asset.family, font_asset.name
        ));
        css.push_str(&format!(
            "body, p, li {{ font-family: \"{}\", \"Palatino\", \"Times New Roman\", serif; }}\n",
            font_asset.family
        ));
    }

    if !style.custom_css.trim().is_empty() {
        css.push_str("\n\n/* === custom css === */\n");
        css.push_str(style.custom_css.trim());
        css.push('\n');
    }

    Ok(css)
}

fn folio_divider_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 80">
  <g fill="none" stroke="#6b5b4b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M300 40 C270 22 230 18 190 30" />
    <path d="M300 40 C270 58 230 62 190 50" />
    <path d="M190 30 C170 26 150 20 130 10" />
    <path d="M190 50 C170 54 150 60 130 70" />
  </g>
  <g fill="none" stroke="#6b5b4b" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" transform="translate(600,0) scale(-1,1)">
    <path d="M300 40 C270 22 230 18 190 30" />
    <path d="M300 40 C270 58 230 62 190 50" />
    <path d="M190 30 C170 26 150 20 130 10" />
    <path d="M190 50 C170 54 150 60 130 70" />
  </g>
  <g fill="none" stroke="#6b5b4b" stroke-width="2">
    <circle cx="300" cy="40" r="9" />
    <circle cx="300" cy="40" r="3" />
    <path d="M292 40 H308" />
  </g>
</svg>"##
}

fn fantasy_divider_svg() -> &'static str {
    r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 90">
  <defs>
    <linearGradient id="g1" x1="0" x2="1">
      <stop offset="0%" stop-color="#a66c44"/>
      <stop offset="50%" stop-color="#bca68a"/>
      <stop offset="100%" stop-color="#a66c44"/>
    </linearGradient>
  </defs>
  <g fill="none" stroke="url(#g1)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <path d="M300 45 C265 20 220 18 175 30" />
    <path d="M300 45 C265 70 220 72 175 60" />
    <path d="M175 30 C155 24 140 16 124 8" />
    <path d="M175 60 C155 66 140 74 124 82" />
    <path d="M210 34 C196 28 186 26 172 28" />
    <path d="M210 56 C196 62 186 64 172 62" />
  </g>
  <g fill="none" stroke="url(#g1)" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" transform="translate(600,0) scale(-1,1)">
    <path d="M300 45 C265 20 220 18 175 30" />
    <path d="M300 45 C265 70 220 72 175 60" />
    <path d="M175 30 C155 24 140 16 124 8" />
    <path d="M175 60 C155 66 140 74 124 82" />
    <path d="M210 34 C196 28 186 26 172 28" />
    <path d="M210 56 C196 62 186 64 172 62" />
  </g>
  <g fill="none" stroke="url(#g1)" stroke-width="2">
    <circle cx="300" cy="45" r="12" />
    <circle cx="300" cy="45" r="4" />
    <path d="M288 45 H312" />
    <path d="M300 33 L310 45 L300 57 L290 45 Z" />
  </g>
</svg>"##
}

fn render_chapter(
    chapter: &ChapterDraft,
    language: &str,
    text_indent: f32,
    template: CssTemplate,
    chapter_index: usize,
) -> String {
    let mut html = String::new();
    html.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    html.push('\n');
    html.push_str(
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#,
    );
    html.push('\n');
    html.push_str(&format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{}">"#,
        language
    ));
    html.push('\n');
    html.push_str("<head>");
    html.push('\n');
    html.push_str(r#"<meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8"/>"#);
    html.push('\n');
    html.push_str(r#"<link rel="stylesheet" type="text/css" href="stylesheet.css"/>"#);
    html.push('\n');
    html.push_str("</head>");
    html.push('\n');
    let body_class = if matches!(template, CssTemplate::Fantasy) {
        "chapter intro2 fantasy"
    } else {
        "chapter"
    };
    html.push_str(&format!("<body class=\"{}\">", body_class));
    html.push('\n');

    if matches!(template, CssTemplate::Fantasy) {
        if let Some((chapter_no, chapter_title)) =
            split_chinese_chapter_title(chapter.title.trim())
        {
            html.push_str("<div class=\"Header-image-dk\">");
            html.push_str("<img class=\"width100\" src=\"images/头图.webp\" alt=\"\"/>");
            html.push_str("</div>\n");
            html.push_str(&format!(
                "<h2 class=\"chapter-title-hidden\">{}</h2>\n",
                escape_html(chapter.title.trim())
            ));
            html.push_str(&format!(
                "<p class=\"nt\"><img class=\"emoji\" src=\"images/4star.webp\" alt=\"\"/> {} <img class=\"emoji\" src=\"images/4star.webp\" alt=\"\"/></p>\n",
                escape_html(&chapter_no)
            ));
            html.push_str(&format!(
                "<p class=\"et\">CHAPTER{:02}</p>\n",
                chapter_index
            ));
            html.push_str(&format!(
                "<p class=\"ct\"><img class=\"emoji1\" src=\"images/ttl.webp\" alt=\"\"/> {} <img class=\"emoji1\" src=\"images/ttr.webp\" alt=\"\"/></p>\n",
                escape_html(&chapter_title)
            ));
        } else {
            append_standard_chapter_header(&mut html, chapter.title.trim(), language);
        }
    } else {
        append_standard_chapter_header(&mut html, chapter.title.trim(), language);
    }

    let indent = format!("{:.2}", text_indent);
    for (idx, paragraph) in split_paragraphs(&chapter.content).into_iter().enumerate() {
        let joined = paragraph
            .iter()
            .map(|line| escape_html(line))
            .collect::<Vec<_>>()
            .join("<br/>");
        if idx == 0 {
            html.push_str(&format!(
                "<p class=\"chapter-paragraph chapter-paragraph-first\" style=\"text-indent: 0.00em;\">{}</p>\n",
                joined
            ));
        } else {
            html.push_str(&format!(
                "<p class=\"chapter-paragraph\" style=\"text-indent: {}em;\">{}</p>\n",
                indent, joined
            ));
        }
    }

    html.push_str("</body>");
    html.push('\n');
    html.push_str("</html>");
    html
}

fn append_standard_chapter_header(html: &mut String, title: &str, language: &str) {
    let (label, title) = format_chapter_heading(title, language);
    html.push_str("<div class=\"chapter-header\">\n");
    html.push_str("<div class=\"chapter-ornament\"></div>\n");
    if let Some(title) = title {
        html.push_str(&format!(
            "<div class=\"chapter-label\">{}</div>\n",
            escape_html(&label)
        ));
        html.push_str(&format!("<h2>{}</h2>\n", escape_html(&title)));
    } else {
        html.push_str(&format!("<h2>{}</h2>\n", escape_html(&label)));
    }
    html.push_str("<div class=\"chapter-ornament\"></div>\n");
    html.push_str("</div>\n");
}

fn gallery_title(language: &str) -> &'static str {
    let lang = language.trim().to_ascii_lowercase();
    if lang.is_empty() || lang.starts_with("zh") {
        "插图"
    } else {
        "Illustrations"
    }
}

fn render_gallery(images: &[ImageAsset], language: &str, title: &str) -> String {
    let mut html = String::new();
    html.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    html.push('\n');
    html.push_str(
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#,
    );
    html.push('\n');
    html.push_str(&format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{}">"#,
        language
    ));
    html.push('\n');
    html.push_str("<head>");
    html.push('\n');
    html.push_str(r#"<meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8"/>"#);
    html.push('\n');
    html.push_str(r#"<link rel="stylesheet" type="text/css" href="stylesheet.css"/>"#);
    html.push('\n');
    html.push_str("</head>");
    html.push('\n');
    html.push_str("<body>");
    html.push('\n');
    html.push_str(&format!("<h2>{}</h2>\n", escape_html(title)));

    for image in images {
        let caption = image
            .caption
            .as_ref()
            .map(|text| escape_html(text))
            .unwrap_or_default();
        html.push_str("<figure>\n");
        html.push_str(&format!(
            "<img src=\"images/{}\" alt=\"{}\"/>\n",
            escape_html(&image.name),
            caption
        ));
        if !caption.is_empty() {
            html.push_str(&format!("<figcaption>{}</figcaption>\n", caption));
        }
        html.push_str("</figure>\n");
    }

    html.push_str("</body>");
    html.push('\n');
    html.push_str("</html>");
    html
}

fn render_text_cover(book_info: &BookInfo, language: &str, template: CssTemplate) -> String {
    let title = if book_info.title.trim().is_empty() {
        "Untitled"
    } else {
        book_info.title.trim()
    };
    let author = if book_info.author.trim().is_empty() {
        "Unknown"
    } else {
        book_info.author.trim()
    };
    let subtitle = book_info.category.trim();
    let publisher = book_info.publisher.trim();
    let publish_date = book_info.publish_date.trim();

    let mut meta_parts = Vec::new();
    if !publisher.is_empty() {
        meta_parts.push(publisher);
    }
    if !publish_date.is_empty() {
        meta_parts.push(publish_date);
    }
    let meta = meta_parts.join(" · ");

    let mut html = String::new();
    html.push_str(r#"<?xml version="1.0" encoding="utf-8"?>"#);
    html.push('\n');
    html.push_str(
        r#"<!DOCTYPE html PUBLIC "-//W3C//DTD XHTML 1.1//EN" "http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd">"#,
    );
    html.push('\n');
    html.push_str(&format!(
        r#"<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="{}">"#,
        language
    ));
    html.push('\n');
    html.push_str("<head>\n");
    html.push_str(r#"<meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8"/>"#);
    html.push('\n');
    html.push_str(r#"<link rel="stylesheet" type="text/css" href="stylesheet.css"/>"#);
    html.push('\n');
    html.push_str("</head>\n");
    let body_class = if matches!(template, CssTemplate::Folio) {
        "cover-page cover-folio"
    } else if matches!(template, CssTemplate::Fantasy) {
        "cover-page cover-fantasy"
    } else {
        "cover-page"
    };
    html.push_str(&format!("<body class=\"{}\">\n", body_class));
    html.push_str("<div class=\"cover-frame\">\n");
    html.push_str("<div class=\"cover-ornament\"></div>\n");
    html.push_str(&format!(
        "<div class=\"cover-title\">{}</div>\n",
        escape_html(title)
    ));
    if !subtitle.is_empty() {
        html.push_str(&format!(
            "<div class=\"cover-subtitle\">{}</div>\n",
            escape_html(subtitle)
        ));
    }
    html.push_str(&format!(
        "<div class=\"cover-author\">{}</div>\n",
        escape_html(author)
    ));
    html.push_str("<div class=\"cover-ornament\"></div>\n");
    if !meta.is_empty() {
        html.push_str(&format!(
            "<div class=\"cover-meta\">{}</div>\n",
            escape_html(&meta)
        ));
    }
    html.push_str("</div>\n");
    html.push_str("</body>\n</html>");
    html
}

fn escape_html(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn split_title_line(line: &str) -> (String, Option<String>) {
    if let Some(idx) = line.find(char::is_whitespace) {
        let (label, rest) = line.split_at(idx);
        let rest = rest.trim();
        if rest.is_empty() {
            (label.trim().to_string(), None)
        } else {
            (label.trim().to_string(), Some(rest.to_string()))
        }
    } else {
        (line.to_string(), None)
    }
}

fn split_chinese_chapter_title(line: &str) -> Option<(String, String)> {
    let trimmed = line.trim();
    if !trimmed.starts_with('第') {
        return None;
    }
    let markers = ['章', '回', '节', '卷', '部', '篇'];
    for marker in markers {
        if let Some(idx) = trimmed.find(marker) {
            let end = idx + marker.len_utf8();
            let prefix = trimmed[..end].trim();
            let mut rest = trimmed[end..].trim();
            rest = rest
                .trim_start_matches([
                    ':', '：', '-', '—', '–', '―', '·', '・', ' ', '\t', '\u{3000}',
                ])
                .trim();
            if !rest.is_empty() {
                return Some((prefix.to_string(), rest.to_string()));
            }
        }
    }
    None
}

fn format_chapter_heading(line: &str, language: &str) -> (String, Option<String>) {
    let trimmed = line.trim();
    let lower = trimmed.to_ascii_lowercase();
    let is_english = language.trim().to_ascii_lowercase().starts_with("en");

    if is_english || lower.starts_with("chapter ") {
        let mut parts = trimmed.split_whitespace();
        if let Some(first) = parts.next() {
            if first.eq_ignore_ascii_case("chapter") {
                if let Some(num_token) = parts.next() {
                    let digits: String = num_token.chars().take_while(|c| c.is_ascii_digit()).collect();
                    if let Ok(num) = digits.parse::<u32>() {
                        let roman = to_roman(num);
                        let mut rest = parts.collect::<Vec<_>>().join(" ");
                        if rest.starts_with([':', '：', '-', '—']) {
                            rest = rest.trim_start_matches([':', '：', '-', '—']).trim().to_string();
                        }
                        let label = format!("Chapter {}", roman);
                        if rest.trim().is_empty() {
                            return (label, None);
                        }
                        return (label, Some(rest));
                    }
                }
            }
        }
    }

    split_title_line(trimmed)
}

fn to_roman(mut num: u32) -> String {
    let mut out = String::new();
    let numerals = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];
    for (value, symbol) in numerals {
        while num >= value {
            out.push_str(symbol);
            num -= value;
        }
    }
    out
}

fn split_paragraphs(content: &str) -> Vec<Vec<String>> {
    let lines: Vec<&str> = content.lines().collect();
    let has_blank = lines.iter().any(|line| line.trim().is_empty());

    if !has_blank {
        let mut non_empty = 0usize;
        let mut punct_lines = 0usize;
        let mut cleaned = Vec::new();
        for line in &lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            non_empty += 1;
            if ends_with_sentence_punct(trimmed) {
                punct_lines += 1;
            }
            cleaned.push(trimmed.to_string());
        }

        if non_empty > 0 && punct_lines * 3 >= non_empty * 2 {
            return cleaned.into_iter().map(|line| vec![line]).collect();
        }

        if !cleaned.is_empty() {
            return vec![cleaned];
        }
    }

    let mut paragraphs = Vec::new();
    let mut current = Vec::new();

    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current);
                current = Vec::new();
            }
        } else {
            current.push(trimmed.to_string());
        }
    }

    if !current.is_empty() {
        paragraphs.push(current);
    }

    if paragraphs.is_empty() && !content.trim().is_empty() {
        paragraphs.push(vec![content.trim().to_string()]);
    }

    paragraphs
}

fn ends_with_sentence_punct(text: &str) -> bool {
    let mut chars = text.chars().rev();
    while let Some(ch) = chars.next() {
        if matches!(
            ch,
            '”' | '’' | '）' | '】' | '》' | '」' | '』' | '〉' | ')' | ']' | '}' | '"' | '\''
        ) {
            continue;
        }
        return matches!(
            ch,
            '。' | '！' | '？' | '…' | '!' | '?' | '.' | '；' | ';' | '：' | ':'
        );
    }
    false
}

fn generate_filename(book_info: &BookInfo, template: &str) -> String {
    let mut filename = template.to_string();
    let title = if book_info.title.trim().is_empty() {
        "Untitled"
    } else {
        book_info.title.trim()
    };
    let author = if book_info.author.trim().is_empty() {
        "Unknown"
    } else {
        book_info.author.trim()
    };

    filename = filename.replace("{书名}", title);
    filename = filename.replace("{作者}", author);
    filename = filename.replace("{日期}", book_info.publish_date.trim());

    filename = sanitize_filename_component(&filename);

    if !filename.ends_with(".epub") {
        filename.push_str(".epub");
    }

    if filename == ".epub" {
        filename = format!("{}_{}.epub", title, author);
    }

    filename
}

fn sanitize_filename_component(input: &str) -> String {
    let invalid_chars = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];
    let mut cleaned = input.to_string();
    for &c in &invalid_chars {
        cleaned = cleaned.replace(c, "");
    }
    cleaned.trim().to_string()
}

fn normalize_output_dir(path: &PathBuf) -> Result<PathBuf, BuildError> {
    if path.as_os_str().is_empty() || path == &PathBuf::from(".") {
        Ok(std::env::current_dir()?)
    } else {
        Ok(path.clone())
    }
}

fn color_to_hex(color: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let html = render_chapter(&chapter, "en", 2.0, crate::CssTemplate::Classic, 1);
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
        let html = render_chapter(&chapter, "zh-CN", 2.0, crate::CssTemplate::Fantasy, 12);
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
        let mut book = BookInfo::default();
        book.title = "Cover Title".to_string();
        book.author = "Cover Author".to_string();
        book.publisher = "Pub".to_string();
        book.publish_date = "2025".to_string();
        let html = render_text_cover(&book, "en", crate::CssTemplate::Classic);
        assert!(html.contains("cover-frame"));
        assert!(html.contains("Cover Title"));
        assert!(html.contains("Cover Author"));
        assert!(html.contains("Pub · 2025"));
    }

    #[test]
    fn build_stylesheet_includes_custom_css_and_font() {
        let mut style = TextStyle::default();
        style.custom_css = "p { color: red; }".to_string();
        style.font_size = 18.0;
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
        let mut style = TextStyle::default();
        style.css_template = crate::CssTemplate::Folio;
        let css = build_stylesheet(&style, None).expect("css");
        assert!(css.contains("/* === folio chapter header overrides === */"));
        assert!(css.contains("folio-divider.svg"));
    }

    #[test]
    fn build_stylesheet_includes_fantasy_overrides() {
        let mut style = TextStyle::default();
        style.css_template = crate::CssTemplate::Fantasy;
        let css = build_stylesheet(&style, None).expect("css");
        assert!(css.contains("/* === fantasy chapter header overrides === */"));
        assert!(css.contains("fantasy-divider.svg"));
        assert!(css.contains("Header-image-dk"));
        assert!(css.contains("fonts/kt.ttf"));
        assert!(css.contains("body.intro2"));
    }

    #[test]
    fn generate_filename_sanitizes_and_appends_extension() {
        let mut book = BookInfo::default();
        book.title = "My/Book".to_string();
        book.author = "A:B".to_string();
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
        let mut book = BookInfo::default();
        book.title = "Test Book".to_string();
        book.author = "Tester".to_string();

        let options = EpubBuildOptions {
            book_info: book,
            output_dir: dir.clone(),
            filename_template: "test_output".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            include_images_section: false,
            inline_toc: false,
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
        let mut book = BookInfo::default();
        book.title = "Assets Book".to_string();

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
            include_images_section: true,
            inline_toc: true,
        };
        let chapters = vec![ChapterDraft {
            title: "Chapter 1".to_string(),
            content: "Hello".to_string(),
        }];

        let output = build_epub(&chapters, &options).expect("build epub");
        let entries = zip_entries(Path::new(&output));

        assert!(entries.iter().any(|name| name.ends_with("stylesheet.css")));
        assert!(entries.iter().any(|name| name.ends_with("chapter_0001.xhtml")));
        assert!(entries.iter().any(|name| name.ends_with("images.xhtml")));
        assert!(entries.iter().any(|name| name.ends_with("images/gallery.png")));
        assert!(entries.iter().any(|name| name.ends_with("fonts/custom.ttf")));
        assert!(entries.iter().any(|name| name.ends_with("cover.jpg")));

        let _ = std::fs::remove_file(&output);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn build_epub_contains_fantasy_assets() {
        let dir = unique_temp_dir("reasypub-fantasy-assets");
        let mut book = BookInfo::default();
        book.title = "Fantasy Assets".to_string();

        let mut style = TextStyle::default();
        style.css_template = crate::CssTemplate::Fantasy;

        let options = EpubBuildOptions {
            book_info: book,
            output_dir: dir.clone(),
            filename_template: "fantasy_assets".to_string(),
            style,
            cover: None,
            images: Vec::new(),
            font: None,
            include_images_section: false,
            inline_toc: false,
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
        let mut book = BookInfo::default();
        book.title = "Meta Title".to_string();
        book.author = "Meta Author".to_string();
        book.language = "en".to_string();
        book.publisher = "Meta Pub".to_string();
        book.isbn = "ISBN-123".to_string();
        book.category = "Category".to_string();
        book.publish_date = "2025-01-01".to_string();
        book.description = "A description.".to_string();

        let options = EpubBuildOptions {
            book_info: book,
            output_dir: dir.clone(),
            filename_template: "meta_output".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            include_images_section: false,
            inline_toc: false,
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
            include_images_section: false,
            inline_toc: false,
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
            include_images_section: false,
            inline_toc: false,
        };
        let chapters = vec![ChapterDraft {
            title: "Chapter 1".to_string(),
            content: "Hello".to_string(),
        }];

        let output = build_epub(&chapters, &options).expect("build epub");
        let entries = zip_entries(Path::new(&output));

        assert!(entries.iter().any(|name| name.ends_with("images/gallery.png")));
        assert!(!entries.iter().any(|name| name.ends_with("images.xhtml")));

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
            include_images_section: false,
            inline_toc: false,
        };
        let err = build_epub(&[], &options).err().expect("error");
        match err {
            BuildError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
