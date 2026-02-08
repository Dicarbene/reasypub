use std::fs::{self, File};
use std::io::Cursor;
use std::path::PathBuf;

use epub_builder::{EpubBuilder, EpubContent, ReferenceType, ZipLibrary};

use crate::{BookInfo, ChapterDraft, CssTemplate, FontAsset, ImageAsset, TextStyle, TocOptions};

mod assets;
mod css;
mod metadata;
mod render;
mod utils;

use assets::add_fantasy_assets;
use css::{build_stylesheet, fantasy_divider_svg, folio_divider_svg};
use metadata::{add_optional_meta_tag, add_optional_metadata};
use render::{gallery_title, render_chapter, render_gallery, render_text_cover};
use utils::{generate_filename, normalize_output_dir};

#[cfg(test)]
use css::color_to_hex;
#[cfg(test)]
use render::{escape_html, split_paragraphs, split_title_line};

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
    pub chapter_header_image: Option<ImageAsset>,
    pub chapter_header_fullbleed: bool,
    pub include_images_section: bool,
    pub toc_options: TocOptions,
}

pub fn build_epub(
    chapters: &[ChapterDraft],
    options: &EpubBuildOptions,
) -> Result<String, BuildError> {
    if chapters.is_empty() {
        return Err(BuildError::InvalidInput(
            "No chapters provided.".to_string(),
        ));
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
    let toc_name = if options.toc_options.toc_title_override.trim().is_empty() {
        if lang.is_empty() || lang.starts_with("zh") {
            "目录"
        } else {
            "Table Of Contents"
        }
        .to_string()
    } else {
        options.toc_options.toc_title_override.trim().to_string()
    };
    add_optional_metadata(&mut builder, "toc_name", &toc_name)?;
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

    if let Some(header) = &options.chapter_header_image {
        let path = format!("images/{}", header.name);
        builder.add_resource(path, Cursor::new(header.bytes.clone()), &header.mime)?;
    }

    if matches!(options.style.css_template, CssTemplate::Folio) {
        builder.add_resource(
            "ornaments/folio-divider.svg",
            Cursor::new(folio_divider_svg().as_bytes()),
            "image/svg+xml",
        )?;
    }
    if matches!(options.style.css_template, CssTemplate::Fantasy) {
        builder.add_resource(
            "ornaments/fantasy-divider.svg",
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
        let cover_html =
            render_text_cover(&options.book_info, language, options.style.css_template);
        builder.add_content(
            EpubContent::new("cover.xhtml", cover_html.as_bytes()).reftype(ReferenceType::Cover),
        )?;
    }

    if options.toc_options.insert_toc_page {
        builder.inline_toc();
    }

    for (index, chapter) in chapters.iter().enumerate() {
        let html = render_chapter(
            chapter,
            language,
            &options.style,
            options.style.css_template,
            index + 1,
            options.chapter_header_image.as_ref(),
            options.chapter_header_fullbleed,
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
        let mut content =
            EpubContent::new("images.xhtml", html.as_bytes()).reftype(ReferenceType::Text);
        if options.toc_options.include_gallery_in_toc {
            content = content.title(gallery_title);
        }
        builder.add_content(content)?;
    }

    builder.generate(writer)?;

    Ok(outpath.display().to_string())
}

#[cfg(test)]
mod tests;
