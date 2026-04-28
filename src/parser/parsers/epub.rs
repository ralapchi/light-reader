/*!
EPUB 解析器模块

实现 EPUB 格式书籍的解析逻辑，包括解析 container.xml、OPF 文件和 HTML 内容。
*/

use crate::parser::parsers::base::{BookParser, ParseResult};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use zip::ZipArchive;

/// EPUB 解析器
/// 
/// 负责解析 EPUB 格式的书籍文件
pub struct EpubParser;

impl EpubParser {
    /// 创建 EPUB 解析器实例
    pub fn new() -> Self {
        Self
    }
    
    /// 解析 container.xml 文件，获取 OPF 文件路径
    /// 
    /// # 参数
    /// * `content` - container.xml 文件内容
    /// 
    /// # 返回值
    /// * `Some(String)` - 成功解析到 OPF 文件路径
    /// * `None` - 解析失败
    fn parse_container_xml(&self, content: &str) -> Option<String> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"rootfile"[..]) {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"full-path"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        return Some(value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        None
    }
    
    /// 解析 OPF 文件，获取 manifest 和 spine 信息
    /// 
    /// # 参数
    /// * `content` - OPF 文件内容
    /// * `_base_path` - 基础路径
    /// 
    /// # 返回值
    /// * `(HashMap<String, String>, Vec<String>)` - (manifest 映射, spine ID 列表)
    fn parse_opf_file(&self, content: &str, _base_path: &str) -> (HashMap<String, String>, Vec<String>) {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut manifest: HashMap<String, String> = HashMap::new();
        let mut spine_ids: Vec<String> = Vec::new();
        let mut in_manifest = false;
        let mut in_spine = false;

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"manifest"[..]) {
                        in_manifest = true;
                    } else if name.starts_with(&b"spine"[..]) {
                        in_spine = true;
                    } else if name.starts_with(&b"item"[..]) && in_manifest {
                        let mut id = String::new();
                        let mut href = String::new();

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"id"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        id = value.to_string();
                                    }
                                } else if attr_name.starts_with(&b"href"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        href = value.to_string();
                                    }
                                }
                            }
                        }

                        if !id.is_empty() && !href.is_empty() {
                            manifest.insert(id, href);
                        }
                    } else if name.starts_with(&b"itemref"[..]) && in_spine {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"idref"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        spine_ids.push(value.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"manifest"[..]) {
                        in_manifest = false;
                    } else if name.starts_with(&b"spine"[..]) {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        (manifest, spine_ids)
    }
    
    /// 获取完整路径
    /// 
    /// # 参数
    /// * `base` - 基础路径
    /// * `href` - 相对路径
    /// 
    /// # 返回值
    /// * `String` - 完整路径
    fn get_full_path(&self, base: &str, href: &str) -> String {
        if href.starts_with('/') {
            href.to_string()
        } else if base.is_empty() {
            href.to_string()
        } else {
            format!("{}/{}", base, href)
        }
    }
    
    /// 剥离 HTML 标签，提取纯文本内容
    /// 
    /// # 参数
    /// * `html` - HTML 内容
    /// 
    /// # 返回值
    /// * `String` - 纯文本内容
    fn strip_html_tags(&self, html: &str) -> String {
        let mut result = Vec::new();
        let mut current_paragraph = String::new();
        let mut text_indent = false;

        let mut reader = Reader::from_str(html);
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"p"[..]) || name.starts_with(&b"div"[..]) {
                        if !current_paragraph.trim().is_empty() {
                            result.push(current_paragraph.clone());
                        }
                        current_paragraph.clear();

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"style"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        if value.contains("text-indent") {
                                            text_indent = true;
                                        }
                                    }
                                }
                            }
                        }
                    } else if name.starts_with(&b"br"[..]) {
                        if !current_paragraph.trim().is_empty() {
                            result.push(current_paragraph.clone());
                        }
                        current_paragraph.clear();
                        text_indent = false;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"p"[..]) || name.starts_with(&b"div"[..]) {
                        if !current_paragraph.trim().is_empty() {
                            result.push(current_paragraph.clone());
                        }
                        current_paragraph.clear();
                        text_indent = false;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Ok(text) = e.unescape() {
                        if text_indent && current_paragraph.is_empty() {
                            current_paragraph.push_str("    ");
                            text_indent = false;
                        }
                        current_paragraph.push_str(&text);
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        if !current_paragraph.trim().is_empty() {
            result.push(current_paragraph);
        }

        result.join("\n\n")
    }
    
    /// 从 HTML 内容中提取标题
    /// 
    /// # 参数
    /// * `html` - HTML 内容
    /// 
    /// # 返回值
    /// * `String` - 提取的标题
    fn extract_title(&self, html: &str) -> String {
        let mut reader = Reader::from_str(html);
        reader.config_mut().trim_text(true);

        let mut title = String::new();
        let mut depth = 0;

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"h1"[..]) || name.starts_with(&b"h2"[..]) || name.starts_with(&b"h3"[..]) {
                        depth += 1;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"h1"[..]) || name.starts_with(&b"h2"[..]) || name.starts_with(&b"h3"[..]) {
                        if depth > 0 {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if depth > 0 {
                        if let Ok(text) = e.unescape() {
                            title.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        title.trim().to_string()
    }
}

impl BookParser for EpubParser {
    /// 解析 EPUB 文件
    /// 
    /// # 参数
    /// * `path` - EPUB 文件路径
    /// 
    /// # 返回值
    /// * `Ok(ParseResult)` - 解析成功，返回解析结果
    /// * `Err(String)` - 解析失败，返回错误信息
    fn parse(&self, path: &str) -> Result<ParseResult, String> {
        let file = File::open(path).map_err(|e| format!("文件打开失败: {}", e))?;
        let mut archive = ZipArchive::new(BufReader::new(file)).map_err(|e| format!("ZIP 解析失败: {}", e))?;

        let opf_path = {
            let mut container_content = String::new();
            if let Ok(mut container_file) = archive.by_name("META-INF/container.xml") {
                container_file.read_to_string(&mut container_content).ok();
            }
            self.parse_container_xml(&container_content).unwrap_or_else(|| "content.opf".to_string())
        };

        let opf_base_path = if let Some(last_slash) = opf_path.rfind('/') {
            opf_path[..last_slash].to_string()
        } else {
            String::new()
        };

        let mut opf_content = String::new();
        if let Ok(mut opf_file) = archive.by_name(&opf_path) {
            opf_file.read_to_string(&mut opf_content).ok();
        }

        let (manifest, spine_ids) = self.parse_opf_file(&opf_content, &opf_base_path);

        let mut content = Vec::new();
        let mut chapter_titles = Vec::new();

        for idref in spine_ids {
            if let Some(href) = manifest.get(&idref) {
                let full_path = self.get_full_path(&opf_base_path, href);
                if let Ok(mut chapter_file) = archive.by_name(&full_path) {
                    let mut html_content = String::new();
                    chapter_file.read_to_string(&mut html_content).ok();

                    let text_content = self.strip_html_tags(&html_content);
                    if !text_content.is_empty() {
                        content.push(text_content);
                        let title = self.extract_title(&html_content);
                        let chapter_title = if title.is_empty() {
                            format!("章节 {}", content.len())
                        } else {
                            title
                        };
                        chapter_titles.push(chapter_title);
                    }
                }
            }
        }

        Ok(ParseResult {
            content,
            chapter_titles,
        })
    }
}
