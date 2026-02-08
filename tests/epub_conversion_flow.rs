use reasypub::conversion::{ConversionFacade, ConversionRequest, StrategyFactory};
use reasypub::{BookInfo, ChapterDraft, ConversionMethod, FontAsset, ImageAsset, TextStyle};
use regex::Regex;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use zip::ZipArchive;

fn temp_output_dir(prefix: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{suffix}"))
}

fn zip_read_to_string(path: &Path, suffix: &str) -> String {
    let file = std::fs::File::open(path).expect("open epub");
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

fn zip_entries(path: &Path) -> Vec<String> {
    let file = std::fs::File::open(path).expect("open epub");
    let mut archive = ZipArchive::new(file).expect("zip");
    (0..archive.len())
        .map(|idx| archive.by_index(idx).expect("entry").name().to_string())
        .collect()
}

fn extract_tag_value(text: &str, tag: &str) -> Option<String> {
    let re = Regex::new(&format!(r"<{tag}>([^<]*)</{tag}>")).expect("regex");
    re.captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn extract_creator(text: &str) -> Option<String> {
    let re = Regex::new(r"<dc:creator[^>]*>([^<]*)</dc:creator>").expect("regex");
    re.captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn extract_meta_content(text: &str, name: &str) -> Option<String> {
    let re = Regex::new(&format!(r#"<meta name="{name}" content="([^"]*)"\s*/?>"#)).expect("regex");
    re.captures(text)
        .and_then(|caps| caps.get(1).map(|m| m.as_str().to_string()))
}

fn nav_hrefs(nav: &str) -> Vec<String> {
    let re = Regex::new(r#"href="([^"]+)""#).expect("regex");
    re.captures_iter(nav)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn toc_ncx_hrefs(ncx: &str) -> Vec<String> {
    let re = Regex::new(r#"content src="([^"]+)""#).expect("regex");
    re.captures_iter(ncx)
        .filter_map(|caps| caps.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

fn toc_href_basename(href: &str) -> &str {
    href.rsplit('/').next().unwrap_or(href)
}

fn is_chapter_href(href: &str) -> bool {
    let re = Regex::new(r"^chapter_\d+\.xhtml$").expect("regex");
    re.is_match(toc_href_basename(href))
}

fn chapter_entries(entries: &[String]) -> Vec<String> {
    let re = Regex::new(r"^chapter_\d+\.xhtml$").expect("regex");
    entries
        .iter()
        .filter(|name| {
            let basename = name.rsplit('/').next().unwrap_or(name.as_str());
            re.is_match(basename)
        })
        .cloned()
        .collect()
}

fn chapter_count(epub: &Path) -> usize {
    let entries = zip_entries(epub);
    chapter_entries(&entries).len()
}

fn chapter_path(index: usize) -> String {
    format!("chapter_{:04}.xhtml", index)
}

fn assert_chapter_contains(epub: &Path, index: usize, needle: &str) {
    let chapter = zip_read_to_string(epub, &chapter_path(index));
    assert!(
        chapter.contains(needle),
        "chapter_{index:04} missing expected content: {needle}"
    );
}

fn split_chapters(text: &str, method: ConversionMethod, custom_regex: &str) -> Vec<ChapterDraft> {
    let strategy = StrategyFactory::create(method, custom_regex, None).expect("strategy");
    strategy.split(text).expect("split")
}

fn find_chapter_index(chapters: &[ChapterDraft], needle: &str) -> usize {
    chapters
        .iter()
        .position(|chapter| chapter.title.contains(needle))
        .map(|idx| idx + 1)
        .expect("chapter not found")
}

fn find_chapter_index_all(chapters: &[ChapterDraft], needles: &[&str]) -> usize {
    chapters
        .iter()
        .position(|chapter| needles.iter().all(|needle| chapter.title.contains(needle)))
        .map(|idx| idx + 1)
        .expect("chapter not found")
}

fn assert_chapter_count_matches_toc(epub: &Path) {
    let entries = zip_entries(epub);
    let chapters = chapter_entries(&entries);
    let nav = zip_read_to_string(epub, "nav.xhtml");
    let mut toc_hrefs = nav_hrefs(&nav);
    let mut toc_chapters = toc_hrefs
        .iter()
        .filter(|href| is_chapter_href(href))
        .count();
    if toc_chapters == 0 {
        let ncx = zip_read_to_string(epub, "toc.ncx");
        toc_hrefs = toc_ncx_hrefs(&ncx);
        toc_chapters = toc_hrefs
            .iter()
            .filter(|href| is_chapter_href(href))
            .count();
    }
    assert_eq!(chapters.len(), toc_chapters);
}

fn assert_images_after_chapters_in_toc(epub: &Path) {
    let nav = zip_read_to_string(epub, "nav.xhtml");
    let mut toc_hrefs = nav_hrefs(&nav);
    let has_chapters = toc_hrefs.iter().any(|href| is_chapter_href(href));
    if !has_chapters {
        let ncx = zip_read_to_string(epub, "toc.ncx");
        toc_hrefs = toc_ncx_hrefs(&ncx);
    }
    let mut last_chapter_idx = None;
    for (idx, href) in toc_hrefs.iter().enumerate() {
        if is_chapter_href(href) {
            last_chapter_idx = Some(idx);
        }
    }
    let images_idx = toc_hrefs
        .iter()
        .position(|href| toc_href_basename(href) == "images.xhtml");
    if let (Some(last_chapter), Some(images)) = (last_chapter_idx, images_idx) {
        assert!(images > last_chapter);
    }
}

const TOC_MARKER: &str = "\u{76ee}\u{5f55}";

fn toc_titles_from_text(text: &str) -> Vec<String> {
    let mut titles = Vec::new();
    let mut seen = HashSet::new();
    let mut in_toc = false;
    for raw in text.lines() {
        let line = raw.trim();
        if !in_toc {
            if line == TOC_MARKER {
                in_toc = true;
            }
            continue;
        }
        if line.is_empty() {
            continue;
        }
        let len = line.chars().count();
        if len > 12
            || line.contains('。')
            || line.contains('，')
            || line.contains('？')
            || line.contains('！')
        {
            break;
        }
        if seen.insert(line.to_string()) {
            titles.push(line.to_string());
        }
    }
    titles
}

fn strip_toc_list(text: &str, titles: &[String]) -> String {
    let mut out = String::new();
    let mut in_toc = false;
    let mut skip_index = 0usize;
    for raw in text.lines() {
        let line = raw.trim();
        if !in_toc {
            if line == TOC_MARKER {
                in_toc = true;
            }
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        if skip_index >= titles.len() {
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        if line.is_empty() {
            out.push_str(raw);
            out.push('\n');
            continue;
        }
        if line == titles[skip_index] {
            skip_index += 1;
            continue;
        }
        skip_index = titles.len();
        out.push_str(raw);
        out.push('\n');
    }
    out
}

fn toc_regex_from_titles(titles: &[String]) -> String {
    let joined = titles
        .iter()
        .map(|title| regex::escape(title))
        .collect::<Vec<_>>()
        .join("|");
    format!(r"(?m)^\s*(?:{})\s*$", joined)
}

#[test]
fn export_epubs_for_manual_check() {
    if std::env::var("REASYPUB_EXPORT_EPUBS").is_err() {
        return;
    }

    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("artifacts")
        .join("epubs");
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let hongloumeng = fixtures_dir.join("红楼梦.txt");
    let chulong = fixtures_dir.join("黜龙.txt");
    let shubuqing = fixtures_dir.join("数不清的井.txt");

    let export_one = |path: PathBuf,
                      title: &str,
                      author: &str,
                      regex: Option<String>,
                      template: reasypub::CssTemplate,
                      suffix: &str,
                      label: &str| {
        let mut text = read_fixture_text(&path);
        let custom_regex = if let Some(regex) = regex {
            let toc_titles = toc_titles_from_text(&text);
            text = strip_toc_list(&text, &toc_titles);
            regex
        } else {
            String::new()
        };

        let book = BookInfo {
            title: title.to_string(),
            author: author.to_string(),
            language: "zh-CN".to_string(),
            ..Default::default()
        };
        let style = TextStyle {
            css_template: template,
            ..Default::default()
        };

        let request = ConversionRequest {
            text,
            method: ConversionMethod::Regex,
            custom_regex,
            custom_config_path: None,
            book_info: book,
            output_dir: output_dir.clone(),
            filename_template: format!("{}{}", title, suffix),
            style,
            cover: None,
            images: Vec::new(),
            font: None,
            chapter_header_image: None,
            chapter_header_fullbleed: false,
            chapters_override: None,
            include_images_section: false,
            inline_toc: true,
        };

        let result = ConversionFacade::convert(request).expect("convert");
        println!("{} ({})", result.output_path, label);
    };

    export_one(
        hongloumeng.clone(),
        "红楼梦",
        "曹雪芹",
        None,
        reasypub::CssTemplate::Classic,
        "",
        "Classic",
    );
    export_one(
        hongloumeng.clone(),
        "红楼梦",
        "曹雪芹",
        None,
        reasypub::CssTemplate::Folio,
        "-folio",
        "Folio",
    );
    export_one(
        hongloumeng,
        "红楼梦",
        "曹雪芹",
        None,
        reasypub::CssTemplate::Fantasy,
        "-fantasy",
        "Fantasy",
    );
    export_one(
        chulong.clone(),
        "黜龙",
        "榴弹怕水",
        None,
        reasypub::CssTemplate::Classic,
        "",
        "Classic",
    );
    export_one(
        chulong.clone(),
        "黜龙",
        "榴弹怕水",
        None,
        reasypub::CssTemplate::Folio,
        "-folio",
        "Folio",
    );
    export_one(
        chulong,
        "黜龙",
        "榴弹怕水",
        None,
        reasypub::CssTemplate::Fantasy,
        "-fantasy",
        "Fantasy",
    );

    let shubuqing_text = read_fixture_text(&shubuqing);
    let toc_titles = toc_titles_from_text(&shubuqing_text);
    let shubuqing_regex = toc_regex_from_titles(&toc_titles);
    export_one(
        shubuqing.clone(),
        "数不清的井",
        "京极夏彦",
        Some(shubuqing_regex.clone()),
        reasypub::CssTemplate::Classic,
        "",
        "Classic",
    );
    export_one(
        shubuqing.clone(),
        "数不清的井",
        "京极夏彦",
        Some(shubuqing_regex.clone()),
        reasypub::CssTemplate::Folio,
        "-folio",
        "Folio",
    );
    export_one(
        shubuqing,
        "数不清的井",
        "京极夏彦",
        Some(shubuqing_regex),
        reasypub::CssTemplate::Fantasy,
        "-fantasy",
        "Fantasy",
    );
}

fn first_non_empty_token(text: &str) -> Option<String> {
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(token) = trimmed.split_whitespace().next()
            && !token.is_empty()
        {
            return Some(token.to_string());
        }
        let token: String = trimmed.chars().take(6).collect();
        if !token.is_empty() {
            return Some(token);
        }
    }
    None
}

fn read_fixture_text(path: &Path) -> String {
    let bytes = std::fs::read(path).expect("read fixture bytes");
    if let Ok(text) = String::from_utf8(bytes.clone()) {
        return text;
    }
    let (decoded, _, _) = encoding_rs::GB18030.decode(&bytes);
    decoded.into_owned()
}

#[test]
fn chinese_novel_txt_conversion_flow() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("novel_zh.txt");
    let text = read_fixture_text(&fixture);

    let book = BookInfo {
        title: "雾桥夜灯".to_string(),
        author: "测试作者".to_string(),
        language: "zh-CN".to_string(),
        publisher: "测试出版社".to_string(),
        isbn: "ISBN-0000".to_string(),
        publish_date: "2025-01-01".to_string(),
        category: "幻想".to_string(),
        description: "测试描述".to_string(),
    };

    let out_dir = temp_output_dir("reasypub-flow");
    let request = ConversionRequest {
        text,
        method: ConversionMethod::Regex, // use built-in Chinese regex
        custom_regex: String::new(),
        custom_config_path: None,
        book_info: book,
        output_dir: out_dir.clone(),
        filename_template: "novel_flow".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        chapters_override: None,
        include_images_section: false,
        inline_toc: true,
    };

    let result = ConversionFacade::convert(request).expect("convert");
    let output = Path::new(&result.output_path);
    assert!(output.exists());

    assert_chapter_count_matches_toc(output);

    // Validate first chapter content was generated.
    let chapter1 = zip_read_to_string(output, "chapter_0001.xhtml");
    assert!(chapter1.contains("第1章"));
    assert!(chapter1.contains("远行"));
    assert!(chapter1.contains("清晨的雾像薄纱"));

    // Validate later chapter exists.
    let chapter5 = zip_read_to_string(output, "chapter_0005.xhtml");
    assert!(chapter5.contains("第5章"));
    assert!(chapter5.contains("归途"));

    let _ = std::fs::remove_file(output);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn chinese_novel_full_pipeline_with_assets() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("novel_zh.txt");
    let text = read_fixture_text(&fixture);

    let book = BookInfo {
        title: "雾桥夜灯".to_string(),
        author: "测试作者".to_string(),
        language: "zh-CN".to_string(),
        publisher: "测试出版社".to_string(),
        isbn: "ISBN-0000".to_string(),
        publish_date: "2025-01-01".to_string(),
        category: "幻想".to_string(),
        description: "测试描述".to_string(),
    };

    let style = TextStyle {
        custom_css: "p { letter-spacing: 0.2px; }".to_string(),
        ..Default::default()
    };

    let cover = ImageAsset {
        name: "cover.jpg".to_string(),
        bytes: bytes::Bytes::from_static(b"cover"),
        mime: "image/jpeg".to_string(),
        caption: None,
    };
    let images = vec![
        ImageAsset {
            name: "scene1.png".to_string(),
            bytes: bytes::Bytes::from_static(b"img1"),
            mime: "image/png".to_string(),
            caption: Some("山路".to_string()),
        },
        ImageAsset {
            name: "scene2.png".to_string(),
            bytes: bytes::Bytes::from_static(b"img2"),
            mime: "image/png".to_string(),
            caption: Some("夜灯".to_string()),
        },
    ];
    let font = FontAsset {
        name: "custom.ttf".to_string(),
        family: "CustomFont".to_string(),
        bytes: bytes::Bytes::from_static(b"font"),
        mime: "font/ttf".to_string(),
    };

    let out_dir = temp_output_dir("reasypub-full");
    let request = ConversionRequest {
        text,
        method: ConversionMethod::Regex,
        custom_regex: String::new(),
        custom_config_path: None,
        book_info: book,
        output_dir: out_dir.clone(),
        filename_template: "novel_full".to_string(),
        style,
        cover: Some(cover),
        images,
        font: Some(font),
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        chapters_override: None,
        include_images_section: true,
        inline_toc: true,
    };

    let result = ConversionFacade::convert(request).expect("convert");
    let output = Path::new(&result.output_path);
    assert!(output.exists());

    assert_chapter_count_matches_toc(output);
    assert_images_after_chapters_in_toc(output);

    let stylesheet = zip_read_to_string(output, "stylesheet.css");
    assert!(stylesheet.contains("@font-face"));
    assert!(stylesheet.contains("CustomFont"));
    assert!(stylesheet.contains("letter-spacing: 0.2px"));

    let opf = zip_read_to_string(output, ".opf");
    assert_eq!(
        extract_tag_value(&opf, "dc:title").as_deref(),
        Some("雾桥夜灯")
    );
    assert_eq!(
        extract_tag_value(&opf, "dc:language").as_deref(),
        Some("zh-CN")
    );
    assert_eq!(extract_creator(&opf).as_deref(), Some("测试作者"));
    assert_eq!(
        extract_tag_value(&opf, "dc:subject").as_deref(),
        Some("幻想")
    );
    assert_eq!(
        extract_tag_value(&opf, "dc:description").as_deref(),
        Some("测试描述")
    );
    assert_eq!(
        extract_meta_content(&opf, "publisher").as_deref(),
        Some("测试出版社")
    );
    assert_eq!(
        extract_meta_content(&opf, "identifier").as_deref(),
        Some("ISBN-0000")
    );
    assert_eq!(
        extract_meta_content(&opf, "date").as_deref(),
        Some("2025-01-01")
    );

    let gallery = zip_read_to_string(output, "images.xhtml");
    assert!(gallery.contains("scene1.png"));
    assert!(gallery.contains("山路"));
    assert!(gallery.contains("scene2.png"));
    assert!(gallery.contains("夜灯"));

    let entries = {
        let file = std::fs::File::open(output).expect("open epub");
        let mut archive = ZipArchive::new(file).expect("zip");
        (0..archive.len())
            .map(|idx| archive.by_index(idx).expect("entry").name().to_string())
            .collect::<Vec<_>>()
    };
    assert!(entries.iter().any(|name| name.ends_with("cover.jpg")));
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("images/scene1.png"))
    );
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("images/scene2.png"))
    );
    assert!(
        entries
            .iter()
            .any(|name| name.ends_with("fonts/custom.ttf"))
    );

    let _ = std::fs::remove_file(output);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn fixtures_txt_conversion_flow() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    let entries = std::fs::read_dir(&fixtures_dir).expect("read fixtures dir");

    for entry in entries {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("txt") {
            continue;
        }
        let text = read_fixture_text(&path);
        let token = first_non_empty_token(&text).expect("token");

        let book = BookInfo {
            title: path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string(),
            author: "测试作者".to_string(),
            language: "zh-CN".to_string(),
            ..Default::default()
        };

        let out_dir = temp_output_dir("reasypub-fixture");
        let request = ConversionRequest {
            text,
            method: ConversionMethod::SimpleRules,
            custom_regex: String::new(),
            custom_config_path: None,
            book_info: book,
            output_dir: out_dir.clone(),
            filename_template: "fixture_flow".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            chapter_header_image: None,
            chapter_header_fullbleed: false,
            chapters_override: None,
            include_images_section: false,
            inline_toc: true,
        };

        let result = ConversionFacade::convert(request).expect("convert");
        let output = Path::new(&result.output_path);
        assert!(output.exists());

        assert_chapter_count_matches_toc(output);

        let chapter1 = zip_read_to_string(output, "chapter_0001.xhtml");
        assert!(chapter1.contains(&token));

        let _ = std::fs::remove_file(output);
        let _ = std::fs::remove_dir_all(&out_dir);
    }
}

#[test]
fn hongloumeng_custom_case() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("红楼梦.txt");
    let text = read_fixture_text(&fixture);

    let chapters = split_chapters(&text, ConversionMethod::Regex, "");
    assert!(chapters.len() >= 120);

    let out_dir = temp_output_dir("reasypub-hlm");
    let book = BookInfo {
        title: "红楼梦".to_string(),
        author: "曹雪芹".to_string(),
        language: "zh-CN".to_string(),
        ..Default::default()
    };

    let request = ConversionRequest {
        text,
        method: ConversionMethod::Regex,
        custom_regex: String::new(),
        custom_config_path: None,
        book_info: book,
        output_dir: out_dir.clone(),
        filename_template: "hongloumeng".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        chapters_override: None,
        include_images_section: false,
        inline_toc: true,
    };

    let result = ConversionFacade::convert(request).expect("convert");
    let output = Path::new(&result.output_path);
    assert!(output.exists());

    assert_chapter_count_matches_toc(output);
    assert_eq!(chapter_count(output), chapters.len());

    let first_idx = find_chapter_index(&chapters, "第一回");
    let last_idx = find_chapter_index(&chapters, "第一二零回");
    assert_chapter_contains(output, first_idx, "第一回");
    assert_chapter_contains(output, first_idx, "甄士隐梦幻识通灵");
    assert_chapter_contains(output, last_idx, "第一二零回");
    assert_chapter_contains(output, last_idx, "贾雨村归结红楼梦");

    let _ = std::fs::remove_file(output);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn chulong_custom_case() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("黜龙.txt");
    let text = read_fixture_text(&fixture);

    let chapters = split_chapters(&text, ConversionMethod::Regex, "");
    assert!(chapters.len() >= 500);

    let out_dir = temp_output_dir("reasypub-chulong");
    let book = BookInfo {
        title: "黜龙".to_string(),
        author: "榴弹怕水".to_string(),
        language: "zh-CN".to_string(),
        ..Default::default()
    };

    let request = ConversionRequest {
        text,
        method: ConversionMethod::Regex,
        custom_regex: String::new(),
        custom_config_path: None,
        book_info: book,
        output_dir: out_dir.clone(),
        filename_template: "chulong".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        chapters_override: None,
        include_images_section: false,
        inline_toc: true,
    };

    let result = ConversionFacade::convert(request).expect("convert");
    let output = Path::new(&result.output_path);
    assert!(output.exists());

    assert_chapter_count_matches_toc(output);
    assert_eq!(chapter_count(output), chapters.len());

    let first_idx = find_chapter_index(&chapters, "第一卷");
    let second_idx = find_chapter_index_all(&chapters, &["第一章", "踉跄行"]);
    let last_idx = find_chapter_index_all(&chapters, &["第一百二十一章", "跨海行"]);
    assert_chapter_contains(output, first_idx, "第一卷");
    assert_chapter_contains(output, second_idx, "第一章");
    assert_chapter_contains(output, second_idx, "踉跄行");
    assert_chapter_contains(output, last_idx, "第一百二十一章");
    assert_chapter_contains(output, last_idx, "跨海行");
    assert_chapter_contains(output, last_idx, "全书完");

    let _ = std::fs::remove_file(output);
    let _ = std::fs::remove_dir_all(&out_dir);
}

#[test]
fn shubuqing_custom_case() {
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("数不清的井.txt");
    let text = read_fixture_text(&fixture);
    let toc_titles = toc_titles_from_text(&text);
    let cleaned_text = strip_toc_list(&text, &toc_titles);
    assert_eq!(toc_titles.len(), 23);
    assert_eq!(toc_titles.first().map(String::as_str), Some("序幕"));
    assert_eq!(toc_titles.last().map(String::as_str), Some("数不清的井"));

    let custom_regex = toc_regex_from_titles(&toc_titles);
    let chapters = split_chapters(&cleaned_text, ConversionMethod::Regex, &custom_regex);
    assert!(chapters.len() >= toc_titles.len());
    assert!(chapters.len() <= toc_titles.len() + 1);

    let out_dir = temp_output_dir("reasypub-shubuqing");
    let book = BookInfo {
        title: "数不清的井".to_string(),
        author: "京极夏彦".to_string(),
        language: "zh-CN".to_string(),
        ..Default::default()
    };

    let request = ConversionRequest {
        text: cleaned_text,
        method: ConversionMethod::Regex,
        custom_regex,
        custom_config_path: None,
        book_info: book,
        output_dir: out_dir.clone(),
        filename_template: "shubuqing".to_string(),
        style: TextStyle::default(),
        cover: None,
        images: Vec::new(),
        font: None,
        chapter_header_image: None,
        chapter_header_fullbleed: false,
        chapters_override: None,
        include_images_section: false,
        inline_toc: true,
    };

    let result = ConversionFacade::convert(request).expect("convert");
    let output = Path::new(&result.output_path);
    assert!(output.exists());

    assert_chapter_count_matches_toc(output);
    assert_eq!(chapter_count(output), chapters.len());

    let first_idx = find_chapter_index(&chapters, "序幕");
    let last_idx = find_chapter_index(&chapters, "数不清的井");
    assert_chapter_contains(output, first_idx, "序幕");
    assert_chapter_contains(output, last_idx, "数不清的井");

    let _ = std::fs::remove_file(output);
    let _ = std::fs::remove_dir_all(&out_dir);
}
