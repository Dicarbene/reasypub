use std::path::PathBuf;

use crate::BookInfo;

use super::BuildError;

pub(super) fn generate_filename(book_info: &BookInfo, template: &str) -> String {
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

pub(super) fn normalize_output_dir(path: &PathBuf) -> Result<PathBuf, BuildError> {
    if path.as_os_str().is_empty() || path == &PathBuf::from(".") {
        Ok(std::env::current_dir()?)
    } else {
        Ok(path.clone())
    }
}
