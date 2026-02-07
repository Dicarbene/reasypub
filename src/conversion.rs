use std::path::PathBuf;

use regex::Regex;

use crate::epubworker::{build_epub, BuildError, EpubBuildOptions};
use crate::{
    BookInfo, ChapterDraft, ConversionMethod, FontAsset, ImageAsset, Pattern, TextProcessor,
    TextStyle,
};

pub struct ConversionRequest {
    pub text: String,
    pub method: ConversionMethod,
    pub custom_regex: String,
    pub custom_config_path: Option<PathBuf>,
    pub book_info: BookInfo,
    pub output_dir: PathBuf,
    pub filename_template: String,
    pub style: TextStyle,
    pub cover: Option<ImageAsset>,
    pub images: Vec<ImageAsset>,
    pub font: Option<FontAsset>,
    pub chapters_override: Option<Vec<ChapterDraft>>,
    pub include_images_section: bool,
    pub inline_toc: bool,
}

pub struct ConversionResult {
    pub output_path: String,
}

pub struct EpubPlanBuilder {
    book_info: BookInfo,
    output_dir: PathBuf,
    filename_template: String,
    style: TextStyle,
    cover: Option<ImageAsset>,
    images: Vec<ImageAsset>,
    font: Option<FontAsset>,
    include_images_section: bool,
    inline_toc: bool,
}

impl EpubPlanBuilder {
    pub fn new(book_info: BookInfo) -> Self {
        Self {
            book_info,
            output_dir: PathBuf::from("."),
            filename_template: "{书名}_{作者}.epub".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            include_images_section: true,
            inline_toc: true,
        }
    }

    pub fn output_dir(mut self, output_dir: PathBuf) -> Self {
        self.output_dir = output_dir;
        self
    }

    pub fn filename_template(mut self, filename_template: String) -> Self {
        self.filename_template = filename_template;
        self
    }

    pub fn style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    pub fn cover(mut self, cover: Option<ImageAsset>) -> Self {
        self.cover = cover;
        self
    }

    pub fn images(mut self, images: Vec<ImageAsset>) -> Self {
        self.images = images;
        self
    }

    pub fn font(mut self, font: Option<FontAsset>) -> Self {
        self.font = font;
        self
    }

    pub fn include_images_section(mut self, include: bool) -> Self {
        self.include_images_section = include;
        self
    }

    pub fn inline_toc(mut self, inline: bool) -> Self {
        self.inline_toc = inline;
        self
    }

    pub fn build(self, chapters: &[ChapterDraft]) -> Result<String, ConversionError> {
        let options = EpubBuildOptions {
            book_info: self.book_info,
            output_dir: self.output_dir,
            filename_template: self.filename_template,
            style: self.style,
            cover: self.cover,
            images: self.images,
            font: self.font,
            include_images_section: self.include_images_section,
            inline_toc: self.inline_toc,
        };
        Ok(build_epub(chapters, &options)?)
    }
}

#[derive(Debug)]
pub enum ConversionError {
    InvalidInput(String),
    Regex(regex::Error),
    Io(std::io::Error),
    Build(BuildError),
}

impl std::fmt::Display for ConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConversionError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ConversionError::Regex(err) => write!(f, "Regex error: {}", err),
            ConversionError::Io(err) => write!(f, "IO error: {}", err),
            ConversionError::Build(err) => write!(f, "EPUB build failed: {}", err),
        }
    }
}

impl std::error::Error for ConversionError {}

impl From<regex::Error> for ConversionError {
    fn from(err: regex::Error) -> Self {
        ConversionError::Regex(err)
    }
}

impl From<std::io::Error> for ConversionError {
    fn from(err: std::io::Error) -> Self {
        ConversionError::Io(err)
    }
}

impl From<BuildError> for ConversionError {
    fn from(err: BuildError) -> Self {
        ConversionError::Build(err)
    }
}

pub trait ChapterSplitStrategy {
    fn split(&self, text: &str) -> Result<Vec<ChapterDraft>, ConversionError>;
}

pub struct RegexSplitStrategy {
    pattern: Pattern,
}

impl RegexSplitStrategy {
    pub fn new(pattern: Pattern) -> Self {
        Self { pattern }
    }
}

impl ChapterSplitStrategy for RegexSplitStrategy {
    fn split(&self, text: &str) -> Result<Vec<ChapterDraft>, ConversionError> {
        let processor = TextProcessor::new(self.pattern.clone(), text.to_string());
        Ok(processor.split_to_drafts())
    }
}

pub struct SimpleRulesStrategy;

impl ChapterSplitStrategy for SimpleRulesStrategy {
    fn split(&self, text: &str) -> Result<Vec<ChapterDraft>, ConversionError> {
        let processor = TextProcessor::new(Pattern::SimpleRules, text.to_string());
        Ok(processor.split_to_drafts())
    }
}

pub struct StrategyFactory;

impl StrategyFactory {
    pub fn create(
        method: ConversionMethod,
        custom_regex: &str,
        config_path: Option<&PathBuf>,
    ) -> Result<Box<dyn ChapterSplitStrategy>, ConversionError> {
        match method {
            ConversionMethod::Regex => {
                let pattern = if custom_regex.trim().is_empty() {
                    Pattern::ChineseChapter
                } else {
                    Pattern::Custom(Regex::new(custom_regex.trim())?)
                };
                Ok(Box::new(RegexSplitStrategy::new(pattern)))
            }
            ConversionMethod::CustomConfig => {
                let path = config_path.ok_or_else(|| {
                    ConversionError::InvalidInput("Please choose a valid regex config file.".to_string())
                })?;
                let regex_str = std::fs::read_to_string(path)?;
                let pattern = Pattern::Custom(Regex::new(regex_str.trim())?);
                Ok(Box::new(RegexSplitStrategy::new(pattern)))
            }
            ConversionMethod::SimpleRules => Ok(Box::new(SimpleRulesStrategy)),
        }
    }
}

pub struct ConversionFacade;

impl ConversionFacade {
    pub fn convert(req: ConversionRequest) -> Result<ConversionResult, ConversionError> {
        if req.text.trim().is_empty() {
            return Err(ConversionError::InvalidInput("Text content is empty.".to_string()));
        }

        let chapters = if let Some(chapters) = req.chapters_override {
            chapters
        } else {
            let strategy = StrategyFactory::create(
                req.method,
                &req.custom_regex,
                req.custom_config_path.as_ref(),
            )?;
            strategy.split(&req.text)?
        };

        if chapters.is_empty() {
            return Err(ConversionError::InvalidInput("No chapters detected.".to_string()));
        }

        let output_path = EpubPlanBuilder::new(req.book_info)
            .output_dir(req.output_dir)
            .filename_template(req.filename_template)
            .style(req.style)
            .cover(req.cover)
            .images(req.images)
            .font(req.font)
            .include_images_section(req.include_images_section)
            .inline_toc(req.inline_toc)
            .build(&chapters)?;

        Ok(ConversionResult { output_path })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::path::Path;

    #[test]
    fn custom_regex_strategy_splits_chapters() {
        let text = "CHAPTER 1\nHello\nCHAPTER 2\nWorld\n";
        let strategy = StrategyFactory::create(
            ConversionMethod::Regex,
            r"(?m)^CHAPTER\s+\d+",
            None,
        )
        .expect("strategy");
        let chapters = strategy.split(text).expect("split");
        assert_eq!(chapters.len(), 2);
    }

    #[test]
    fn custom_config_without_path_is_error() {
        let err = StrategyFactory::create(ConversionMethod::CustomConfig, "", None)
            .err()
            .expect("error");
        match err {
            ConversionError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn simple_rules_strategy_splits_chapters() {
        let text = "第1章 开始\n内容\n\n第2章 继续\n更多";
        let strategy = StrategyFactory::create(ConversionMethod::SimpleRules, "", None)
            .expect("strategy");
        let chapters = strategy.split(text).expect("split");
        assert_eq!(chapters.len(), 2);
        assert_eq!(chapters[0].title, "第1章 开始");
        assert_eq!(chapters[1].title, "第2章 继续");
    }

    #[test]
    fn default_regex_strategy_matches_chinese_titles() {
        let text = "第1章 你好\n内容\n第2章 再见\n内容";
        let strategy = StrategyFactory::create(ConversionMethod::Regex, "", None)
            .expect("strategy");
        let chapters = strategy.split(text).expect("split");
        assert_eq!(chapters.len(), 2);
    }

    #[test]
    fn invalid_custom_regex_is_error() {
        let err = StrategyFactory::create(ConversionMethod::Regex, "(", None)
            .err()
            .expect("error");
        match err {
            ConversionError::Regex(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn conversion_facade_rejects_empty_text() {
        let req = ConversionRequest {
            text: "  ".to_string(),
            method: ConversionMethod::Regex,
            custom_regex: String::new(),
            custom_config_path: None,
            book_info: BookInfo::default(),
            output_dir: PathBuf::from("."),
            filename_template: "out".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            chapters_override: None,
            include_images_section: false,
            inline_toc: false,
        };
        let err = ConversionFacade::convert(req).err().expect("error");
        match err {
            ConversionError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn conversion_facade_rejects_empty_override() {
        let req = ConversionRequest {
            text: "content".to_string(),
            method: ConversionMethod::Regex,
            custom_regex: String::new(),
            custom_config_path: None,
            book_info: BookInfo::default(),
            output_dir: PathBuf::from("."),
            filename_template: "out".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            chapters_override: Some(Vec::new()),
            include_images_section: false,
            inline_toc: false,
        };
        let err = ConversionFacade::convert(req).err().expect("error");
        match err {
            ConversionError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn conversion_facade_writes_output_with_override() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let out_dir = std::env::temp_dir().join(format!("reasypub-convert-{suffix}"));
        let chapter = ChapterDraft {
            title: "Chapter 1".to_string(),
            content: "Hello".to_string(),
        };
        let req = ConversionRequest {
            text: "content".to_string(),
            method: ConversionMethod::Regex,
            custom_regex: String::new(),
            custom_config_path: None,
            book_info: BookInfo::default(),
            output_dir: out_dir.clone(),
            filename_template: "convert_test".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            chapters_override: Some(vec![chapter]),
            include_images_section: false,
            inline_toc: false,
        };
        let result = ConversionFacade::convert(req).expect("convert");
        assert!(!result.output_path.is_empty());
        let _ = std::fs::remove_file(&result.output_path);
        let _ = std::fs::remove_dir_all(&out_dir);
    }

    #[test]
    fn custom_config_strategy_reads_file() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("reasypub-regex-{suffix}.txt"));
        std::fs::write(&path, r"(?m)^CHAPTER\s+\d+").expect("write regex");

        let strategy = StrategyFactory::create(
            ConversionMethod::CustomConfig,
            "",
            Some(&path),
        )
        .expect("strategy");
        let chapters = strategy
            .split("CHAPTER 1\nA\nCHAPTER 2\nB\n")
            .expect("split");
        assert_eq!(chapters.len(), 2);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn custom_config_invalid_regex_is_error() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("reasypub-regex-bad-{suffix}.txt"));
        std::fs::write(&path, "(").expect("write regex");

        let err = StrategyFactory::create(
            ConversionMethod::CustomConfig,
            "",
            Some(&path),
        )
        .err()
        .expect("error");
        match err {
            ConversionError::Regex(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn custom_config_missing_file_is_io_error() {
        let missing = Path::new("this-file-should-not-exist-regex.txt").to_path_buf();
        let err = StrategyFactory::create(
            ConversionMethod::CustomConfig,
            "",
            Some(&missing),
        )
        .err()
        .expect("error");
        match err {
            ConversionError::Io(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn conversion_facade_rejects_empty_text_even_with_override() {
        let req = ConversionRequest {
            text: " ".to_string(),
            method: ConversionMethod::Regex,
            custom_regex: String::new(),
            custom_config_path: None,
            book_info: BookInfo::default(),
            output_dir: PathBuf::from("."),
            filename_template: "out".to_string(),
            style: TextStyle::default(),
            cover: None,
            images: Vec::new(),
            font: None,
            chapters_override: Some(vec![ChapterDraft {
                title: "Chapter 1".to_string(),
                content: "Hello".to_string(),
            }]),
            include_images_section: false,
            inline_toc: false,
        };
        let err = ConversionFacade::convert(req).err().expect("error");
        match err {
            ConversionError::InvalidInput(_) => {}
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
