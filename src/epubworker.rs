use std::env;
use std::fmt::format;
use std::fs::read;
use std::fs::File;
use std::io::Read;
use std::io::Write;

use bytes::Bytes;
use epub_builder::EpubBuilder;
use epub_builder::EpubContent;
use epub_builder::ReferenceType;
use epub_builder::Result;
use epub_builder::TocElement;
use epub_builder::ZipLibrary;
use rfd::FileDialog;

use std::io;

use crate::ImageFileReader;
use crate::Pattern;
use crate::TextFileReader;

// txt_file: TextFileReader, pattern: Pattern, cover_image: ImageFileReader, images: vec<Bytes>
pub fn txt_build(txt: &Vec<String>, reg: Pattern) -> Result<()> {
    // 输出路径
    let curr_dir = env::current_dir().unwrap();
    let outpath = curr_dir.join("book.epub");
    log::debug!("write file to: {}", &outpath.display());
    let writer = File::create(outpath).unwrap();

    // 创建新的 EpubBuilder
    // using the zip library
    let mut builder = EpubBuilder::new(ZipLibrary::new()?)?;

    // 写入元数据
    builder
        .metadata("author", "Wikipedia Contributors")?
        .metadata("title", "Ada Lovelace: first programmer")?
        // // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
        .stylesheet(File::open("assets/book/book.css")?)?
        // Add a image cover file
        .add_cover_image(
            "cover.png",
            File::open("assets/book/Ada_Lovelace_color.svg")?,
            "image/svg",
        )?;
    let mut begin= String::new();
    let _ = File::read_to_string(&mut File::open("assets/begin.txt")?,&mut begin);
    println!("开始转换");
    println!("{}",begin);
    let task_result = txt.iter().enumerate().for_each(|(i, chapter)| {
        //处理章节列表
        let mut html_content = String::new();
        html_content.push_str(&begin);
        html_content.push_str("<html><body>\n");
        let lines: Vec<&str> = chapter.split('\n').collect();
        let title_line = lines[0];
        //println!("{:?}",chapter);
        let re = reg.to_regex();

        if re.is_match(title_line) {
            if let Some((chapter_label, title_text)) = title_line.split_once(" ") {
                html_content.push_str(&format!(
                    "<div class=\"chapter-label\">{}</div>\n",
                    chapter_label.trim()
                ));
                html_content.push_str(&format!("<h2>{}</h2>\n", title_text.trim()));
            } else {
                html_content.push_str(&format!("<p>{}</p>\n", title_line));
            }
        } else {
            html_content.push_str(&format!("<p>{}</p>\n", title_line));
        }
        
        let content_lines: Vec<&str> = lines[1..].iter().map(|&s| s).collect();
        let paragraphs = get_paragraphs(&content_lines);
        for paragraph in paragraphs {
            for line in paragraph {
                html_content.push_str(&format!("<p>{}</p>\n", line));
            }
        }
        html_content.push_str("</body></html>\n");
        let index: String = format!("chapter_{}.xhtml", i);
        builder
            .add_content(
                EpubContent::new(index, html_content.as_bytes())
                .title(title_line)
                .reftype(ReferenceType::Text)
                ,
            )
            .unwrap();
    });
    //println!("{:?}",task_result);
    builder.generate(writer)?; // generate into file to see epub

    log::debug!("sample book generation is done");
    Ok(())
}

/* fn main() {
    match txt_build() {
        Ok(_) => writeln!(
            &mut io::stderr(),
            "Successfully wrote epub document to stdout!"
        )
        .unwrap(),
        Err(err) => writeln!(&mut io::stderr(), "Error: {}", err).unwrap(),
    };
} */

fn get_paragraphs(content_lines: &[&str]) -> Vec<Vec<String>> {
    let mut paragraphs = Vec::new();
    let mut current_paragraph = Vec::new();

    for line in content_lines {
        if line.trim().is_empty() {
            if !current_paragraph.is_empty() {
                paragraphs.push(current_paragraph);
                current_paragraph = Vec::new();
            }
        } else {
            current_paragraph.push(line.to_string());
        }
    }

    if !current_paragraph.is_empty() {
        paragraphs.push(current_paragraph);
    }

    paragraphs
}
