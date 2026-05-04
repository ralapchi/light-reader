/*!
EPUB 解析器模块

实现 EPUB 格式书籍的解析逻辑，包括解析 container.xml、OPF 文件和 HTML 内容。
*/

use crate::domain::book_metadata::BookMetadata;
use crate::domain::toc_item::TocItem;
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
    
    /// 从 OPF 内容中提取元信息
    fn parse_opf_metadata(&self, content: &str) -> Option<BookMetadata> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut in_metadata = false;
        let mut title = String::new();
        let mut creator = String::new();
        let mut language = String::new();
        let mut identifier = String::new();
        let mut current_tag = String::new();
        let mut warnings = Vec::new();

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) => {
                    let qname = e.name();
                    let name = std::str::from_utf8(qname.as_ref()).unwrap_or("");
                    if name == "metadata" {
                        in_metadata = true;
                    } else if in_metadata {
                        current_tag = name.to_string();
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = std::str::from_utf8(qname.as_ref()).unwrap_or("");
                    if name == "metadata" {
                        in_metadata = false;
                    } else if in_metadata {
                        current_tag.clear();
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if in_metadata {
                        if let Ok(text) = e.unescape() {
                            let text = text.trim().to_string();
                            if text.is_empty() {
                                continue;
                            }
                            let tag = current_tag.as_str();
                            // Handle dc: prefixed tags (strip namespace)
                            let local = if let Some(pos) = tag.find(':') {
                                &tag[pos + 1..]
                            } else {
                                tag
                            };
                            match local {
                                "title" if title.is_empty() => title = text,
                                "creator" if creator.is_empty() => creator = text,
                                "language" if language.is_empty() => language = text,
                                "identifier" if identifier.is_empty() => identifier = text,
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        if title.is_empty() {
            warnings.push("EPUB 缺少 dc:title 元信息".to_string());
            return None;
        }

        Some(BookMetadata {
            title,
            author: if creator.is_empty() { None } else { Some(creator) },
            language: if language.is_empty() { None } else { Some(language) },
            identifier: if identifier.is_empty() { None } else { Some(identifier) },
            publisher: None,
            description: None,
            series: None,
            cover_title: None,
            created_at: None,
            modified_at: None,
        })
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

    /// 解析 EPUB3 nav.xhtml 文件（Navigation Document）
    ///
    /// # 参数
    /// * `content` - nav.xhtml 文件内容
    /// * `base_href` - 基础路径
    ///
    /// # 返回值
    /// * `Vec<TocItem>` - 解析得到的目录树
    fn parse_nav_document(&self, content: &str, base_href: &str) -> Vec<TocItem> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut in_nav = false;
        let mut in_ol = false;
        let mut depth: u8 = 0;
        let mut current_text = String::new();
        let mut current_href = String::new();
        let mut collecting = false;

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"nav"[..]) {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"type"[..]) || attr_name.starts_with(&b"epub:type"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        if value.contains("toc") {
                                            in_nav = true;
                                        }
                                    }
                                }
                            }
                        }
                    } else if in_nav && name.starts_with(&b"ol"[..]) {
                        in_ol = true;
                        depth += 1;
                    } else if in_nav && in_ol && name.starts_with(&b"a"[..]) {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"href"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        current_href = value.to_string();
                                    }
                                }
                            }
                        }
                        collecting = true;
                        current_text.clear();
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"nav"[..]) {
                        in_nav = false;
                    } else if in_nav && name.starts_with(&b"ol"[..]) {
                        in_ol = false;
                        depth = depth.saturating_sub(1);
                    } else if in_nav && in_ol && name.starts_with(&b"a"[..]) {
                        if collecting && !current_text.is_empty() {
                            let full_href = if current_href.starts_with('#') {
                                current_href.clone()
                            } else {
                                self.get_full_path(base_href, &current_href)
                            };
                            items.push(TocItem {
                                id: format!("toc-{}", items.len()),
                                title: current_text.trim().to_string(),
                                chapter_index: None,
                                href: Some(full_href),
                                depth: depth.saturating_sub(1),
                                children: Vec::new(),
                                is_generated: false,
                            });
                        }
                        collecting = false;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if collecting {
                        if let Ok(text) = e.unescape() {
                            current_text.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        items
    }

    /// 解析 EPUB2 toc.ncx 文件（NCX Navigation）
    ///
    /// # 参数
    /// * `content` - toc.ncx 文件内容
    /// * `base_href` - 基础路径
    ///
    /// # 返回值
    /// * `Vec<TocItem>` - 解析得到的目录树
    fn parse_ncx(&self, content: &str, base_href: &str) -> Vec<TocItem> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut items = Vec::new();
        let mut in_nav_map = false;
        let mut in_nav_point = false;
        let mut depth: u8 = 0;
        let mut current_text = String::new();
        let mut current_src = String::new();
        let mut collecting_text = false;

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"navMap"[..]) {
                        in_nav_map = true;
                    } else if in_nav_map && name.starts_with(&b"navPoint"[..]) {
                        in_nav_point = true;
                        depth += 1;
                    } else if in_nav_point && name.starts_with(&b"content"[..]) {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.starts_with(&b"src"[..]) {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        current_src = value.to_string();
                                    }
                                }
                            }
                        }
                    } else if in_nav_point && name.starts_with(&b"text"[..]) {
                        collecting_text = true;
                        current_text.clear();
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.starts_with(&b"navMap"[..]) {
                        in_nav_map = false;
                    } else if name.starts_with(&b"navPoint"[..]) {
                        if in_nav_point && !current_text.is_empty() {
                            let full_src = if current_src.starts_with('#') {
                                current_src.clone()
                            } else {
                                self.get_full_path(base_href, &current_src)
                            };
                            items.push(TocItem {
                                id: format!("toc-{}", items.len()),
                                title: current_text.trim().to_string(),
                                chapter_index: None,
                                href: Some(full_src),
                                depth: depth.saturating_sub(1),
                                children: Vec::new(),
                                is_generated: false,
                            });
                        }
                        in_nav_point = false;
                        depth = depth.saturating_sub(1);
                        current_src.clear();
                    } else if name.starts_with(&b"text"[..]) {
                        collecting_text = false;
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if collecting_text {
                        if let Ok(text) = e.unescape() {
                            current_text.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        items
    }

    /// 构建目录树（将扁平列表转换为层级结构）
    ///
    /// # 参数
    /// * `items` - 扁平的目录项列表
    ///
    /// # 返回值
    /// * `Vec<TocItem>` - 层级目录树
    fn build_toc_tree(&self, items: Vec<TocItem>) -> Vec<TocItem> {
        if items.is_empty() {
            return items;
        }

        let mut root: Vec<TocItem> = Vec::new();
        let mut stack: Vec<(u8, usize)> = Vec::new(); // (depth, index_in_root_or_parent)

        for item in items {
            let depth = item.depth;

            // 弹出栈中深度 >= 当前深度的项
            while let Some(&(_, _)) = stack.last() {
                if stack.last().map_or(false, |&(d, _)| d >= depth) {
                    stack.pop();
                } else {
                    break;
                }
            }

            if stack.is_empty() {
                // 顶层项
                let index = root.len();
                root.push(item);
                stack.push((depth, index));
            } else {
                // 子项：找到父节点并添加
                let &(_, _parent_idx) = stack.last().unwrap();
                // 需要递归找到正确的父节点
                let parent = self.find_parent_mut(&mut root, &stack);
                let child_index = parent.children.len();
                parent.children.push(item);
                stack.push((depth, child_index));
            }
        }

        root
    }

    /// 辅助方法：根据栈找到可变父节点
    fn find_parent_mut<'a>(&self, root: &'a mut Vec<TocItem>, stack: &[(u8, usize)]) -> &'a mut TocItem {
        let mut current = &mut root[stack[0].1];
        for &(_, idx) in &stack[1..] {
            current = &mut current.children[idx];
        }
        current
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
        let mut warnings = Vec::new();

        let opf_path = {
            let mut container_content = String::new();
            if let Ok(mut container_file) = archive.by_name("META-INF/container.xml") {
                container_file.read_to_string(&mut container_content).ok();
            } else {
                warnings.push("缺少 META-INF/container.xml，使用默认路径".to_string());
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
        } else {
            warnings.push(format!("无法读取 OPF 文件: {}", opf_path));
        }

        let (manifest, spine_ids) = self.parse_opf_file(&opf_content, &opf_base_path);
        let metadata = self.parse_opf_metadata(&opf_content);
        if metadata.is_none() {
            warnings.push("EPUB 缺少元信息，使用文件名作为标题".to_string());
        }

        // 尝试解析结构化目录（优先 nav.xhtml，回退 toc.ncx）
        let mut toc_items: Option<Vec<TocItem>> = None;

        // 1. 尝试 EPUB3 nav.xhtml
        if let Some(nav_href) = manifest.values().find(|href| href.ends_with("nav.xhtml") || href.contains("nav")) {
            let nav_path = self.get_full_path(&opf_base_path, nav_href);
            if let Ok(mut nav_file) = archive.by_name(&nav_path) {
                let mut nav_content = String::new();
                nav_file.read_to_string(&mut nav_content).ok();
                let items = self.parse_nav_document(&nav_content, &opf_base_path);
                if !items.is_empty() {
                    let tree = self.build_toc_tree(items);
                    toc_items = Some(tree);
                    warnings.push("使用 EPUB3 nav.xhtml 目录".to_string());
                }
            }
        }

        // 2. 回退到 EPUB2 toc.ncx
        if toc_items.is_none() {
            if let Some(ncx_href) = manifest.values().find(|href| href.ends_with(".ncx")) {
                let ncx_path = self.get_full_path(&opf_base_path, ncx_href);
                if let Ok(mut ncx_file) = archive.by_name(&ncx_path) {
                    let mut ncx_content = String::new();
                    ncx_file.read_to_string(&mut ncx_content).ok();
                    let items = self.parse_ncx(&ncx_content, &opf_base_path);
                    if !items.is_empty() {
                        let tree = self.build_toc_tree(items);
                        toc_items = Some(tree);
                        warnings.push("使用 EPUB2 toc.ncx 目录".to_string());
                    }
                }
            }
        }

        // 3. 回退到 spine（当前逻辑）
        if toc_items.is_none() {
            warnings.push("未找到结构化目录，使用 spine 顺序".to_string());
        }

        let mut content = Vec::new();
        let mut chapter_titles = Vec::new();
        let mut spine_hrefs = Vec::new();

        for idref in &spine_ids {
            if let Some(href) = manifest.get(idref.as_str()) {
                let full_path = self.get_full_path(&opf_base_path, href);
                if let Ok(mut chapter_file) = archive.by_name(&full_path) {
                    let mut html_content = String::new();
                    chapter_file.read_to_string(&mut html_content).ok();

                    let text_content = self.strip_html_tags(&html_content);
                    if !text_content.is_empty() {
                        content.push(text_content);
                        spine_hrefs.push(href.clone());
                        let title = self.extract_title(&html_content);
                        let chapter_title = if title.is_empty() {
                            format!("章节 {}", content.len())
                        } else {
                            title
                        };
                        chapter_titles.push(chapter_title);
                    } else {
                        warnings.push(format!("章节 {} 内容为空，已跳过", idref));
                    }
                } else {
                    warnings.push(format!("无法读取章节文件: {}", full_path));
                }
            } else {
                warnings.push(format!("spine 引用了不存在的 manifest 项: {}", idref));
            }
        }

        Ok(ParseResult {
            content,
            chapter_titles,
            spine_hrefs,
            toc: toc_items,
            metadata,
            warnings,
        })
    }
}
