use std::fs;

use crate::{CssTemplate, FontAsset, TextStyle};

use super::BuildError;

pub(super) fn build_stylesheet(
    style: &TextStyle,
    font: Option<&FontAsset>,
) -> Result<String, BuildError> {
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
    css.push_str(
        ".cover-author { font-size: 1.1em; letter-spacing: 0.2em; margin: 1.2em 0 0.2em; }\n",
    );
    css.push_str(".cover-meta { font-size: 0.85em; letter-spacing: 0.2em; color: #6b5b4b; margin-top: 1.4em; }\n");
    css.push_str(".cover-ornament { height: 1.8em; width: 70%; margin: 0.8em auto; border-top: 1px solid #6b5b4b; border-bottom: 1px solid #cbbda9; }\n");

    css.push_str("\n\n/* === chapter header === */\n");
    css.push_str(".chapter { page-break-before: always; break-before: page; }\n");
    css.push_str(".chapter-head-image { text-align: center; margin: 0 0 1.2em; }\n");
    css.push_str(".chapter-head-image img { width: 100%; max-width: 100%; border: none; box-shadow: none; background: none; }\n");
    css.push_str(
        ".chapter-head-image.fullbleed { duokan-bleed: lefttopright; margin: 0 0 -30% 0; }\n",
    );
    css.push_str(
        ".chapter-header { text-align: center; margin: 2.4em 0 2.1em; position: relative; padding: 0.8em 0 1em; background: linear-gradient(#8a7a66, #8a7a66) left top/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) left top/1px 1.4em no-repeat, linear-gradient(#8a7a66, #8a7a66) right top/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) right top/1px 1.4em no-repeat, linear-gradient(#8a7a66, #8a7a66) left bottom/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) left bottom/1px 1.4em no-repeat, linear-gradient(#8a7a66, #8a7a66) right bottom/1.4em 1px no-repeat, linear-gradient(#8a7a66, #8a7a66) right bottom/1px 1.4em no-repeat; }\n",
    );
    css.push_str(
        ".chapter-header::before, .chapter-header::after { content: \"\"; position: absolute; top: 0.25em; width: 0.45em; height: 0.45em; border: 1px solid #8a7a66; background: transparent; transform: rotate(45deg); }\n",
    );
    css.push_str(".chapter-header::before { left: 0.35em; }\n");
    css.push_str(".chapter-header::after { right: 0.35em; }\n");
    css.push_str(
        ".chapter-header h2 { display: inline-block; padding: 0 0.7em; position: relative; }\n",
    );
    css.push_str(
        ".chapter-ornament { border-top: 1px solid #6b5b4b; border-bottom: 1px solid #c0b5a4; height: 0; margin: 0.9em auto; width: 54%; text-align: center; }\n",
    );
    css.push_str(
        ".chapter-ornament::after { content: \"\"; display: inline-block; margin-top: -0.75em; width: 0.3em; height: 0.3em; border: 1px solid #6b5b4b; border-radius: 50%; background: transparent; box-shadow: -1.2em 0 0 #6b5b4b, 1.2em 0 0 #6b5b4b, -2.4em 0 0 #c0b5a4, 2.4em 0 0 #c0b5a4, -3.6em 0 0 #6b5b4b, 3.6em 0 0 #6b5b4b; }\n",
    );
    css.push_str(".chapter-label { string-set: chapter content(); }\n");
    css.push_str(
        "@page { @top-center { content: string(chapter); font-family: \"Garamond\", \"Times New Roman\", serif; font-size: 0.7em; letter-spacing: 0.2em; color: #6b5b4b; } }\n",
    );
    css.push_str("@page :first { @top-center { content: normal; } }\n");
    css.push_str(".chapter-paragraph-first { text-indent: 0 !important; }\n");
    css.push_str(
        ".chapter-paragraph-first::first-letter { float: left; font-size: 3.2em; line-height: 0.85; padding: 0.04em 0.1em 0 0; font-weight: 600; color: #5a4a3b; }\n",
    );

    if matches!(style.css_template, CssTemplate::Folio) {
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

    if matches!(style.css_template, CssTemplate::Fantasy) {
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

pub(super) fn folio_divider_svg() -> &'static str {
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

pub(super) fn fantasy_divider_svg() -> &'static str {
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

pub(super) fn color_to_hex(color: egui::Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}
