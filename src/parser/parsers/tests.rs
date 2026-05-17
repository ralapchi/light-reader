/*!
解析层回归测试

覆盖 EPUB 元信息、目录解析、HTML 清洗、TXT 自动切章、告警收集等场景。
*/

use std::io::Write;
use zip::ZipWriter;
use zip::write::FileOptions;

use super::base::BookParser;
use super::epub::EpubParser;
use super::txt::TxtParser;

// ---------------------------------------------------------------------------
// EPUB 测试辅助
// ---------------------------------------------------------------------------

fn build_epub(opf_body: &str, chapters: &[(&str, &str)]) -> Vec<u8> {
    build_epub_with_files(opf_body, chapters, &[])
}

fn build_epub_with_files(
    opf_body: &str,
    chapters: &[(&str, &str)],
    files: &[(&str, &[u8])],
) -> Vec<u8> {
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = FileOptions::default();

        zip.start_file("META-INF/container.xml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#,
        )
        .unwrap();

        zip.start_file("OEBPS/content.opf", opts).unwrap();
        zip.write_all(opf_body.as_bytes()).unwrap();

        for (name, html) in chapters {
            zip.start_file(&format!("OEBPS/{}", name), opts).unwrap();
            zip.write_all(html.as_bytes()).unwrap();
        }

        for (name, bytes) in files {
            zip.start_file(*name, opts).unwrap();
            zip.write_all(bytes).unwrap();
        }

        zip.finish().unwrap();
    }
    buf
}

fn write_temp_epub(data: &[u8]) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::with_suffix(".epub").unwrap();
    f.write_all(data).unwrap();
    f
}

fn write_temp_txt(content: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::with_suffix(".txt").unwrap();
    f.write_all(content.as_bytes()).unwrap();
    f
}

// ---------------------------------------------------------------------------
// EPUB 元信息测试
// ---------------------------------------------------------------------------

#[test]
fn epub_metadata_extracts_dc_fields() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <metadata>
    <dc:title>Test Book</dc:title>
    <dc:creator>Author Name</dc:creator>
    <dc:language>en</dc:language>
    <dc:identifier>isbn-123</dc:identifier>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Chapter content.</p></body></html>"#;
    let data = build_epub(opf, &[("ch1.xhtml", html)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    let meta = result.metadata.expect("metadata should be present");

    assert_eq!(meta.title, "Test Book");
    assert_eq!(meta.author.as_deref(), Some("Author Name"));
    assert_eq!(meta.language.as_deref(), Some("en"));
    assert_eq!(meta.identifier.as_deref(), Some("isbn-123"));
}

#[test]
fn epub_metadata_missing_title_yields_none() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <metadata>
    <dc:language>en</dc:language>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Text.</p></body></html>"#;
    let data = build_epub(opf, &[("ch1.xhtml", html)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert!(result.metadata.is_none());
    assert!(result.warnings.iter().any(|w| w.contains("元信息")));
}

// ---------------------------------------------------------------------------
// EPUB 目录解析测试
// ---------------------------------------------------------------------------

#[test]
fn epub_ncx_toc_parsed() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="ncx" href="toc.ncx" media-type="application/x-dtbncx+xml"/>
  </manifest>
  <spine toc="ncx">
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let ncx = r#"<?xml version="1.0" encoding="UTF-8"?>
<ncx xmlns="http://www.daisy.org/z3986/2005/ncx/">
  <navMap>
    <navPoint id="np1"><navLabel><text>Chapter 1</text></navLabel><content src="ch1.xhtml"/></navPoint>
  </navMap>
</ncx>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Content.</p></body></html>"#;
    let data = build_epub(opf, &[("ch1.xhtml", html), ("toc.ncx", ncx)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    let toc = result.toc.expect("toc should be present");
    assert!(!toc.is_empty());
    assert_eq!(toc[0].title, "Chapter 1");
}

// ---------------------------------------------------------------------------
// HTML 清洗测试
// ---------------------------------------------------------------------------

#[test]
fn epub_html_stripped_to_paragraphs() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <body>
    <h1>Title Here</h1>
    <p>First paragraph.</p>
    <p>Second paragraph.</p>
  </body>
</html>"#;

    let data = build_epub(opf, &[("ch1.xhtml", html)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 1);
    let text = &result.content[0];
    assert!(text.contains("First paragraph."));
    assert!(text.contains("Second paragraph."));
    assert!(!text.contains("<p>"));
    assert_eq!(result.chapter_titles[0], "Title Here");
}

// ---------------------------------------------------------------------------
// TXT 章节检测测试
// ---------------------------------------------------------------------------

#[test]
fn txt_chinese_chapter_detection() {
    let content = "第1章 起始\n\n段落一。\n\n段落二。\n\n第2章 发展\n\n段落三。";
    let tmp = write_temp_txt(content);

    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 2);
    assert_eq!(result.chapter_titles[0], "第1章 起始");
    assert_eq!(result.chapter_titles[1], "第2章 发展");
    assert!(result.content[0].contains("段落一"));
    assert!(result.content[1].contains("段落三"));
}

#[test]
fn txt_english_chapter_detection() {
    let content = "Chapter 1 Beginning\n\nSome text.\n\nChapter 2 Middle\n\nMore text.";
    let tmp = write_temp_txt(content);

    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 2);
    assert_eq!(result.chapter_titles[0], "Chapter 1 Beginning");
    assert_eq!(result.chapter_titles[1], "Chapter 2 Middle");
}

#[test]
fn txt_no_chapter_fallback() {
    let content = "Just some text.\n\nAnother paragraph.\n\nYet another.";
    let tmp = write_temp_txt(content);

    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    // Fallback: split by double newline, single chapter titled "文本文件"
    assert_eq!(result.chapter_titles.len(), 1);
    assert_eq!(result.chapter_titles[0], "文本文件");
    assert!(result.content.len() >= 1);
}

#[test]
fn txt_arabic_cn_mixed_chapter() {
    let content = "第1章 开始\n\n段落一。\n\n第12章 结束\n\n段落二。";
    let tmp = write_temp_txt(content);
    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 2);
    assert_eq!(result.chapter_titles[0], "第1章 开始");
    assert_eq!(result.chapter_titles[1], "第12章 结束");
}

#[test]
fn txt_special_cn_chapters() {
    let content = "序章 引子\n\n段落一。\n\n第一章 正文\n\n段落二。\n\n终章 结局\n\n段落三。";
    let tmp = write_temp_txt(content);
    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 3);
    assert_eq!(result.chapter_titles[0], "序章 引子");
    assert_eq!(result.chapter_titles[1], "第一章 正文");
    assert_eq!(result.chapter_titles[2], "终章 结局");
}

#[test]
fn txt_english_part_detection() {
    let content = "Part 1 Beginning\n\nText.\n\nPart 2 End\n\nMore text.";
    let tmp = write_temp_txt(content);
    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 2);
    assert_eq!(result.chapter_titles[0], "Part 1 Beginning");
    assert_eq!(result.chapter_titles[1], "Part 2 End");
}

#[test]
fn txt_number_dot_prefix() {
    let content = "1. 开篇\n\n段落一。\n\n12. 结尾\n\n段落二。";
    let tmp = write_temp_txt(content);
    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 2);
    assert_eq!(result.chapter_titles[0], "1. 开篇");
    assert_eq!(result.chapter_titles[1], "12. 结尾");
}

#[test]
fn txt_cn_number_prefix() {
    let content = "一、开篇\n\n段落一。\n\n二十、结尾\n\n段落二。";
    let tmp = write_temp_txt(content);
    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 2);
    assert_eq!(result.chapter_titles[0], "一、开篇");
    assert_eq!(result.chapter_titles[1], "二十、结尾");
}

// ---------------------------------------------------------------------------
// 告警收集测试
// ---------------------------------------------------------------------------

#[test]
fn epub_warns_on_missing_container() {
    // Build EPUB without container.xml
    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        let opts = FileOptions::default();

        // No META-INF/container.xml — just put content.opf at root
        zip.start_file("content.opf", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#,
        )
        .unwrap();

        zip.start_file("ch1.xhtml", opts).unwrap();
        zip.write_all(
            br#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Text.</p></body></html>"#,
        )
        .unwrap();

        zip.finish().unwrap();
    }

    let tmp = write_temp_epub(&buf);
    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert!(result.warnings.iter().any(|w| w.contains("container.xml")));
}

#[test]
fn epub_warns_on_empty_chapter() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    // Chapter with no text content
    let empty_html = r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body></body></html>"#;
    let data = build_epub(opf, &[("ch1.xhtml", empty_html)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert!(result.content.is_empty());
    assert!(result.warnings.iter().any(|w| w.contains("内容为空")));
}

// ---------------------------------------------------------------------------
// ParseResult 完整性测试
// ---------------------------------------------------------------------------

#[test]
fn epub_parse_result_has_all_fields() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf" xmlns:dc="http://purl.org/dc/elements/1.1/">
  <metadata>
    <dc:title>Complete Book</dc:title>
  </metadata>
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml"><body><h1>Ch1</h1><p>Hello.</p></body></html>"#;
    let data = build_epub(opf, &[("ch1.xhtml", html)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert!(!result.content.is_empty());
    assert!(!result.chapter_titles.is_empty());
    assert!(result.metadata.is_some());
    // warnings is always a Vec (may or may not be empty)
    let _ = result.warnings;
}

#[test]
fn txt_parse_result_has_all_fields() {
    let content = "第1章 测试\n\n内容段落。";
    let tmp = write_temp_txt(content);

    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert!(!result.content.is_empty());
    assert!(!result.chapter_titles.is_empty());
    assert!(result.metadata.is_none()); // TXT has no metadata
    assert!(result.warnings.is_empty());
}

// ---------------------------------------------------------------------------
// 超长内容回归
// ---------------------------------------------------------------------------

#[test]
fn txt_large_file_does_not_panic() {
    // Generate a large TXT with many chapters
    let mut content = String::new();
    for i in 1..=100 {
        content.push_str(&format!("第{}章 章节标题\n\n", i));
        for j in 1..=50 {
            content.push_str(&format!(
                "这是第{}章的第{}个段落，包含一些测试内容用于验证大文件解析不会崩溃。\n\n",
                i, j
            ));
        }
    }
    let tmp = write_temp_txt(&content);

    let result = TxtParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert_eq!(result.content.len(), 100);
    assert_eq!(result.chapter_titles.len(), 100);
}

// ---------------------------------------------------------------------------
// 损坏 EPUB 回归
// ---------------------------------------------------------------------------

#[test]
fn corrupted_epub_returns_error() {
    let tmp = write_temp_epub(b"this is not a valid zip file");
    let result = EpubParser::new().parse(tmp.path().to_str().unwrap());
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// 无封面 EPUB（仅验证不 panic）
// ---------------------------------------------------------------------------

#[test]
fn epub_without_cover_parses() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="ch1.xhtml" media-type="application/xhtml+xml"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?><html xmlns="http://www.w3.org/1999/xhtml"><body><p>Text.</p></body></html>"#;
    let data = build_epub(opf, &[("ch1.xhtml", html)]);
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();
    assert!(!result.content.is_empty());
}

#[test]
fn epub_image_paths_are_normalized_and_deduplicated() {
    let opf = r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="2.0" xmlns="http://www.idpf.org/2007/opf">
  <manifest>
    <item id="ch1" href="Text/ch1.xhtml" media-type="application/xhtml+xml"/>
    <item id="img1" href="images/pic.jpg" media-type="image/jpeg"/>
  </manifest>
  <spine>
    <itemref idref="ch1"/>
  </spine>
</package>"#;

    let html = r#"<?xml version="1.0" encoding="UTF-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <body>
    <p>Text.</p>
    <img src="../images/pic.jpg#cover" alt="Cover A" />
    <img src="../images/pic.jpg?version=1" alt="Cover B" />
  </body>
</html>"#;

    let data = build_epub_with_files(
        opf,
        &[("Text/ch1.xhtml", html)],
        &[("OEBPS/images/pic.jpg", b"fake-image-bytes")],
    );
    let tmp = write_temp_epub(&data);

    let result = EpubParser::new()
        .parse(tmp.path().to_str().unwrap())
        .unwrap();

    assert_eq!(result.image_assets.len(), 1);
    assert_eq!(result.image_assets[0].asset_path, "OEBPS/images/pic.jpg");
    assert_eq!(result.image_assets[0].source_href, "../images/pic.jpg");
    assert_eq!(result.chapter_image_blocks.len(), 1);
    assert_eq!(result.chapter_image_blocks[0].len(), 2);
    assert_eq!(
        result.chapter_image_blocks[0][0].1.asset_id,
        result.chapter_image_blocks[0][1].1.asset_id
    );
}
