use std::fs;
use std::io::Cursor;

use epub_builder::{EpubBuilder, ZipLibrary};

use super::BuildError;

pub(super) fn add_fantasy_assets(builder: &mut EpubBuilder<ZipLibrary>) -> Result<(), BuildError> {
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
