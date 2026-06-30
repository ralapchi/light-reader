/*!
EPUB 元数据解析

负责解析 container.xml、OPF 文件及其中的元信息。
*/

use crate::domain::book_metadata::BookMetadata;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;

use super::epub_parser::EpubParser;

impl EpubParser {
    /// 解析 container.xml 文件，获取 OPF 文件路径
    ///
    /// # 参数
    /// * `content` - container.xml 文件内容
    ///
    /// # 返回值
    /// * `Some(String)` - 成功解析到 OPF 文件路径
    /// * `None` - 解析失败
    pub(super) fn parse_container_xml(&self, content: &str) -> Option<String> {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.eq_ignore_ascii_case(b"rootfile") {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.eq_ignore_ascii_case(b"full-path") {
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
    /// * `(HashMap<String, String>, Vec<String>, Option<String>)` - (manifest 映射, spine ID 列表, 封面 href)
    pub(super) fn parse_opf_file(
        &self,
        content: &str,
        _base_path: &str,
    ) -> (HashMap<String, String>, Vec<String>, Option<String>) {
        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut manifest: HashMap<String, String> = HashMap::new();
        let mut spine_ids: Vec<String> = Vec::new();
        let mut cover_id: Option<String> = None;
        let mut in_manifest = false;
        let mut in_spine = false;
        let mut in_metadata = false;

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.eq_ignore_ascii_case(b"metadata") {
                        in_metadata = true;
                    } else if name.eq_ignore_ascii_case(b"manifest") {
                        in_manifest = true;
                    } else if name.eq_ignore_ascii_case(b"spine") {
                        in_spine = true;
                    } else if name.eq_ignore_ascii_case(b"item") && in_manifest {
                        let mut id = String::new();
                        let mut href = String::new();
                        let mut properties = String::new();

                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.eq_ignore_ascii_case(b"id") {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        id = value.to_string();
                                    }
                                } else if attr_name.eq_ignore_ascii_case(b"href") {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        href = value.to_string();
                                    }
                                } else if attr_name.eq_ignore_ascii_case(b"properties") {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        properties = value.to_string();
                                    }
                                }
                            }
                        }

                        // EPUB 3: properties="cover-image" marks the cover
                        if properties.contains("cover-image") && !href.is_empty() {
                            cover_id = Some(href.clone());
                        }

                        if !id.is_empty() && !href.is_empty() {
                            manifest.insert(id, href);
                        }
                    } else if name.eq_ignore_ascii_case(b"meta") && in_metadata {
                        // EPUB 2: <meta name="cover" content="cover-image-id"/>
                        let mut meta_name = String::new();
                        let mut meta_content = String::new();
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.eq_ignore_ascii_case(b"name") {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        meta_name = value.to_string();
                                    }
                                } else if attr_name.eq_ignore_ascii_case(b"content") {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        meta_content = value.to_string();
                                    }
                                }
                            }
                        }
                        if meta_name == "cover" && !meta_content.is_empty() {
                            // Resolve id to href via manifest (may not be populated yet,
                            // so we store the id and resolve later)
                            if let Some(href) = manifest.get(&meta_content) {
                                cover_id = Some(href.clone());
                            } else if cover_id.is_none() {
                                // Store as id marker — resolved after manifest is fully parsed
                                cover_id = Some(format!("__id__:{}", meta_content));
                            }
                        }
                    } else if name.eq_ignore_ascii_case(b"itemref") && in_spine {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.eq_ignore_ascii_case(b"idref") {
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
                    if name.eq_ignore_ascii_case(b"metadata") {
                        in_metadata = false;
                    } else if name.eq_ignore_ascii_case(b"manifest") {
                        in_manifest = false;
                    } else if name.eq_ignore_ascii_case(b"spine") {
                        in_spine = false;
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        // Resolve EPUB 2 id marker to href
        if let Some(ref marker) = cover_id {
            if let Some(id) = marker.strip_prefix("__id__:") {
                cover_id = manifest.get(id).cloned();
            }
        }

        (manifest, spine_ids, cover_id)
    }

    /// 从 OPF 内容中提取元信息
    pub(super) fn parse_opf_metadata(&self, content: &str) -> Option<BookMetadata> {
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
            author: if creator.is_empty() {
                None
            } else {
                Some(creator)
            },
            language: if language.is_empty() {
                None
            } else {
                Some(language)
            },
            identifier: if identifier.is_empty() {
                None
            } else {
                Some(identifier)
            },
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
    pub(super) fn get_full_path(&self, base: &str, href: &str) -> String {
        if href.starts_with('/') {
            href.to_string()
        } else if base.is_empty() {
            href.to_string()
        } else {
            format!("{}/{}", base, href)
        }
    }
}
