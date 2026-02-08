use crate::{BookInfo, ChapterDraft, CssTemplate, ImageAsset, TextStyle};

pub(super) fn render_chapter(
    chapter: &ChapterDraft,
    language: &str,
    style: &TextStyle,
    template: CssTemplate,
    chapter_index: usize,
    header_image: Option<&ImageAsset>,
    header_fullbleed: bool,
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
    html.push_str(
        r#"<meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8"/>"#,
    );
    html.push('\n');
    html.push_str(r#"<link rel="stylesheet" type="text/css" href="stylesheet.css"/>"#);
    html.push('\n');
    html.push_str("</head>");
    html.push('\n');
    let base_body_class = if matches!(template, CssTemplate::Fantasy) {
        "chapter intro2 fantasy"
    } else {
        "chapter"
    };
    let body_class = merge_classes(base_body_class, &style.extra_body_class);
    html.push_str(&format!("<body class=\"{}\">", escape_html(&body_class)));
    html.push('\n');

    if matches!(template, CssTemplate::Fantasy) {
        if let Some((chapter_no, chapter_title)) = split_chinese_chapter_title(chapter.title.trim())
        {
            html.push_str("<div class=\"Header-image-dk\">");
            let fantasy_header_src = header_image
                .map(|asset| format!("images/{}", asset.name))
                .unwrap_or_else(|| "images/头图.webp".to_string());
            html.push_str(&format!(
                "<img class=\"width100\" src=\"{}\" alt=\"\"/>",
                escape_html(&fantasy_header_src)
            ));
            html.push_str("</div>\n");
            let hidden_class = merge_classes("chapter-title-hidden", &style.extra_title_class);
            html.push_str(&format!(
                "<h2 class=\"{}\">{}</h2>\n",
                escape_html(&hidden_class),
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
            append_standard_chapter_header(&mut html, chapter.title.trim(), language, style);
        }
    } else {
        if let Some(header) = header_image {
            let header_class = if header_fullbleed {
                "chapter-head-image fullbleed"
            } else {
                "chapter-head-image"
            };
            html.push_str(&format!(
                "<div class=\"{}\"><img src=\"images/{}\" alt=\"\"/></div>\n",
                escape_html(header_class),
                escape_html(&header.name)
            ));
        }
        append_standard_chapter_header(&mut html, chapter.title.trim(), language, style);
    }

    let indent = format!("{:.2}", style.text_indent);
    for (idx, mut paragraph) in split_paragraphs(&chapter.content).into_iter().enumerate() {
        let marker_class = extract_marker_class(&mut paragraph);
        let joined = paragraph
            .iter()
            .map(|line| escape_html(line))
            .collect::<Vec<_>>()
            .join("<br/>");
        let mut paragraph_class = String::from("chapter-paragraph");
        if idx == 0 {
            paragraph_class.push_str(" chapter-paragraph-first");
        }
        paragraph_class = merge_classes(&paragraph_class, &style.extra_paragraph_class);
        if let Some(marker_class) = marker_class.as_ref() {
            paragraph_class = merge_classes(&paragraph_class, marker_class);
        }
        if idx == 0 {
            html.push_str(&format!(
                "<p class=\"{}\" style=\"text-indent: 0.00em;\">{}</p>\n",
                escape_html(&paragraph_class),
                joined
            ));
        } else {
            html.push_str(&format!(
                "<p class=\"{}\" style=\"text-indent: {}em;\">{}</p>\n",
                escape_html(&paragraph_class),
                indent,
                joined
            ));
        }
    }

    html.push_str("</body>");
    html.push('\n');
    html.push_str("</html>");
    html
}

fn append_standard_chapter_header(
    html: &mut String,
    title: &str,
    language: &str,
    style: &TextStyle,
) {
    let (label, title) = format_chapter_heading(title, language);
    let header_class = merge_classes("chapter-header", &style.extra_chapter_class);
    html.push_str(&format!("<div class=\"{}\">\n", header_class));
    html.push_str("<div class=\"chapter-ornament\"></div>\n");
    if let Some(title) = title {
        html.push_str(&format!(
            "<div class=\"chapter-label\">{}</div>\n",
            escape_html(&label)
        ));
        if style.extra_title_class.trim().is_empty() {
            html.push_str(&format!("<h2>{}</h2>\n", escape_html(&title)));
        } else {
            html.push_str(&format!(
                "<h2 class=\"{}\">{}</h2>\n",
                escape_html(style.extra_title_class.trim()),
                escape_html(&title)
            ));
        }
    } else if style.extra_title_class.trim().is_empty() {
        html.push_str(&format!("<h2>{}</h2>\n", escape_html(&label)));
    } else {
        html.push_str(&format!(
            "<h2 class=\"{}\">{}</h2>\n",
            escape_html(style.extra_title_class.trim()),
            escape_html(&label)
        ));
    }
    html.push_str("<div class=\"chapter-ornament\"></div>\n");
    html.push_str("</div>\n");
}

fn merge_classes(base: &str, extra: &str) -> String {
    if extra.trim().is_empty() {
        return base.to_string();
    }
    let mut classes = vec![base.to_string()];
    classes.extend(extra.split_whitespace().map(|s| s.to_string()));
    classes.join(" ")
}

fn extract_marker_class(lines: &mut Vec<String>) -> Option<String> {
    if lines.is_empty() {
        return None;
    }
    let first_line = lines[0].clone();
    let first = first_line.trim_start();
    let lower = first.to_ascii_lowercase();
    if !lower.starts_with("[class=") {
        return None;
    }
    let end = first.find(']')?;
    let class_value = first[7..end].trim();
    if class_value.is_empty() {
        return None;
    }
    let rest = first[end + 1..].trim_start();
    if rest.is_empty() {
        lines.remove(0);
    } else {
        lines[0] = rest.to_string();
    }
    Some(class_value.to_string())
}

pub(super) fn gallery_title(language: &str) -> &'static str {
    let lang = language.trim().to_ascii_lowercase();
    if lang.is_empty() || lang.starts_with("zh") {
        "插图"
    } else {
        "Illustrations"
    }
}

pub(super) fn render_gallery(images: &[ImageAsset], language: &str, title: &str) -> String {
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
    html.push_str(
        r#"<meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8"/>"#,
    );
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

pub(super) fn render_text_cover(
    book_info: &BookInfo,
    language: &str,
    template: CssTemplate,
) -> String {
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
    html.push_str(
        r#"<meta http-equiv="Content-Type" content="application/xhtml+xml; charset=utf-8"/>"#,
    );
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

pub(super) fn escape_html(input: &str) -> String {
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

pub(super) fn split_title_line(line: &str) -> (String, Option<String>) {
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
        if let Some(first) = parts.next()
            && first.eq_ignore_ascii_case("chapter")
            && let Some(num_token) = parts.next()
        {
            let digits: String = num_token
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            if let Ok(num) = digits.parse::<u32>() {
                let roman = to_roman(num);
                let mut rest = parts.collect::<Vec<_>>().join(" ");
                if rest.starts_with([':', '：', '-', '—']) {
                    rest = rest
                        .trim_start_matches([':', '：', '-', '—'])
                        .trim()
                        .to_string();
                }
                let label = format!("Chapter {}", roman);
                if rest.trim().is_empty() {
                    return (label, None);
                }
                return (label, Some(rest));
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

pub(super) fn split_paragraphs(content: &str) -> Vec<Vec<String>> {
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
    let chars = text.chars().rev();
    for ch in chars {
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
