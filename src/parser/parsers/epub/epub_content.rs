/*!
EPUB 内容提取与清洗

负责从 HTML 中提取段落文本、图片位置、链接片段、锚点，以及标题提取和文本清洗。
*/

use crate::domain::paragraph::TextLink;
use quick_xml::Reader;
use quick_xml::events::Event;

use super::epub_parser::{EpubParser, is_img_tag, read_img_attrs, record_anchor};

impl EpubParser {
    /// 单次遍历 HTML，同时提取段落文本、图片位置、链接片段和锚点。
    /// 返回 (paragraphs, images, paragraph_links, anchors, heading_flags, inline_images)。
    pub(super) fn extract_html_with_positions(
        &self,
        html: &str,
    ) -> (
        Vec<String>,
        Vec<(isize, String, Option<String>)>,
        Vec<Vec<TextLink>>,
        Vec<(String, usize)>,
        Vec<bool>,
        Vec<(usize, String, Option<String>)>,
    ) {
        let mut paragraphs: Vec<String> = Vec::new();
        let mut current_para = String::new();
        let mut text_indent = false;
        let mut images: Vec<(isize, String, Option<String>)> = Vec::new();
        let mut para_count: isize = 0;

        let mut paragraph_links: Vec<Vec<TextLink>> = Vec::new();
        let mut current_links: Vec<TextLink> = Vec::new();
        let mut anchors: Vec<(String, usize)> = Vec::new();
        let mut in_link = false;
        let mut link_href = String::new();
        let mut link_title: Option<String> = None;
        let mut link_start: usize = 0;
        let mut in_title_tag = false;
        let mut heading_depth: u32 = 0;
        let mut heading_flags: Vec<bool> = Vec::new();
        let mut in_sup = false;
        let mut link_is_footnote = false;
        let mut inline_img_counter: usize = 0;
        let mut inline_images: Vec<(usize, String, Option<String>)> = Vec::new();
        let mut in_style_or_script: u8 = 0;

        // Helper: flush current paragraph on br/hr
        let flush_para = |current_para: &mut String,
                              paragraphs: &mut Vec<String>,
                              paragraph_links: &mut Vec<Vec<TextLink>>,
                              current_links: &mut Vec<TextLink>,
                              text_indent: &mut bool,
                              para_count: &mut isize| {
            if !current_para.trim().is_empty() {
                paragraphs.push(std::mem::take(current_para));
                paragraph_links.push(std::mem::take(current_links));
            }
            *text_indent = false;
            *para_count += 1;
        };

        // Helper: extract href/title from <a> tag attributes
        let handle_a_attrs = |attrs: quick_xml::events::attributes::Attributes,
                                   link_href: &mut String,
                                   link_title: &mut Option<String>,
                                   in_link: &mut bool,
                                   link_start: &mut usize,
                                   is_footnote: &mut bool,
                                   current_para: &str| {
            link_href.clear();
            *link_title = None;
            *is_footnote = false;
            for attr in attrs {
                if let Ok(attr) = attr {
                    let an = attr.key.as_ref();
                    if an.eq_ignore_ascii_case(b"href") {
                        if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                            let v = v.trim();
                            if !v.is_empty()
                                && !v.starts_with("javascript:")
                                && !v.starts_with("mailto:")
                                && !v.starts_with("http://")
                                && !v.starts_with("https://")
                            {
                                *link_href = v.to_string();
                            }
                        }
                    } else if an.eq_ignore_ascii_case(b"title") {
                        if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                            *link_title = Some(v.trim().to_string());
                        }
                    } else if an.eq_ignore_ascii_case(b"type") {
                        if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                            if v.contains("noteref") {
                                *is_footnote = true;
                            }
                        }
                    }
                }
            }
            if !link_href.is_empty() {
                *in_link = true;
                *link_start = current_para.chars().count();
            }
        };

        let mut reader = Reader::from_str(html);
        reader.config_mut().trim_text(true);
        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.eq_ignore_ascii_case(b"title") {
                        in_title_tag = true;
                    }
                    if name.eq_ignore_ascii_case(b"style") || name.eq_ignore_ascii_case(b"script") {
                        in_style_or_script = in_style_or_script.saturating_add(1);
                    }

                    // 提取锚点 id/name
                    record_anchor(e.attributes(), &mut anchors, paragraphs.len());

                    let is_heading_tag = name.eq_ignore_ascii_case(b"h1")
                        || name.eq_ignore_ascii_case(b"h2")
                        || name.eq_ignore_ascii_case(b"h3")
                        || name.eq_ignore_ascii_case(b"h4")
                        || name.eq_ignore_ascii_case(b"h5")
                        || name.eq_ignore_ascii_case(b"h6");

                    if is_heading_tag {
                        heading_depth += 1;
                    }
                    if name.eq_ignore_ascii_case(b"p") || name.eq_ignore_ascii_case(b"div") || name.eq_ignore_ascii_case(b"li") || is_heading_tag {
                        if !current_para.trim().is_empty() {
                            paragraphs.push(std::mem::take(&mut current_para));
                            paragraph_links.push(std::mem::take(&mut current_links));
                            heading_flags.push(heading_depth > 0);
                        }
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
                    } else if name.eq_ignore_ascii_case(b"br") || name.eq_ignore_ascii_case(b"hr") {
                        if !current_para.trim().is_empty() {
                            heading_flags.push(heading_depth > 0);
                        }
                        flush_para(&mut current_para, &mut paragraphs, &mut paragraph_links, &mut current_links, &mut text_indent, &mut para_count);
                    } else if name.eq_ignore_ascii_case(b"sup") {
                        in_sup = true;
                    } else if name.eq_ignore_ascii_case(b"a") {
                        handle_a_attrs(e.attributes(), &mut link_href, &mut link_title, &mut in_link, &mut link_start, &mut link_is_footnote, &current_para);
                    }
                }
                Ok(Event::Empty(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();

                    // 提取锚点 id/name
                    record_anchor(e.attributes(), &mut anchors, paragraphs.len());

                    if is_img_tag(name) {
                        let (src, alt, style) = read_img_attrs(e.attributes());
                        if !src.is_empty() {
                            let is_inline = !current_para.is_empty()
                                && style.as_deref().map_or(false, |s| {
                                    s.contains("height:1em") || s.contains("display:inline") || s.contains("display: inline")
                                });
                            if is_inline {
                                current_para.push_str(&format!("\u{E000}{}\u{E001}", inline_img_counter));
                                inline_images.push((inline_img_counter, src, alt));
                                inline_img_counter += 1;
                            } else {
                                images.push((para_count - 1, src, alt));
                            }
                        }
                    } else if name.eq_ignore_ascii_case(b"br") || name.eq_ignore_ascii_case(b"hr") {
                        if !current_para.trim().is_empty() {
                            heading_flags.push(heading_depth > 0);
                        }
                        flush_para(&mut current_para, &mut paragraphs, &mut paragraph_links, &mut current_links, &mut text_indent, &mut para_count);
                    } else if name.eq_ignore_ascii_case(b"a") {
                        handle_a_attrs(e.attributes(), &mut link_href, &mut link_title, &mut in_link, &mut link_start, &mut link_is_footnote, &current_para);
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.eq_ignore_ascii_case(b"title") {
                        in_title_tag = false;
                        // Discard any title text that leaked into current_para
                        current_para.clear();
                    }
                    if name.eq_ignore_ascii_case(b"style") || name.eq_ignore_ascii_case(b"script") {
                        in_style_or_script = in_style_or_script.saturating_sub(1);
                    }
                    let is_heading_tag = name.eq_ignore_ascii_case(b"h1")
                        || name.eq_ignore_ascii_case(b"h2")
                        || name.eq_ignore_ascii_case(b"h3")
                        || name.eq_ignore_ascii_case(b"h4")
                        || name.eq_ignore_ascii_case(b"h5")
                        || name.eq_ignore_ascii_case(b"h6");

                    if name.eq_ignore_ascii_case(b"p") || name.eq_ignore_ascii_case(b"div") || name.eq_ignore_ascii_case(b"li") || is_heading_tag {
                        if !current_para.trim().is_empty() {
                            paragraphs.push(std::mem::take(&mut current_para));
                            paragraph_links.push(std::mem::take(&mut current_links));
                            heading_flags.push(heading_depth > 0);
                        }
                        text_indent = false;
                        para_count += 1;
                        if is_heading_tag {
                            heading_depth = heading_depth.saturating_sub(1);
                        }
                    } else if name.eq_ignore_ascii_case(b"sup") {
                        in_sup = false;
                    } else if name.eq_ignore_ascii_case(b"a") && in_link {
                        let end = current_para.chars().count();
                        if end > link_start {
                            current_links.push(TextLink {
                                start: link_start,
                                end,
                                href: link_href.clone(),
                                title: link_title.clone(),
                                is_footnote: link_is_footnote || in_sup,
                            });
                        }
                        in_link = false;
                        link_href.clear();
                        link_title = None;
                        link_is_footnote = false;
                    }
                }
                Ok(Event::Text(ref _e)) if in_title_tag => {
                    // Skip text inside <title> tag — it's metadata, not content
                }
                Ok(Event::Text(ref _e)) if in_style_or_script > 0 => {
                    // Skip text inside <style>/<script> tags
                }
                Ok(Event::Text(ref e)) => {
                    if let Ok(text) = e.unescape() {
                        if text_indent && current_para.is_empty() {
                            current_para.push_str("\x01INDENT\x01");
                            text_indent = false;
                        }
                        current_para.push_str(&text);
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        if !current_para.trim().is_empty() {
            paragraphs.push(current_para);
            paragraph_links.push(std::mem::take(&mut current_links));
            heading_flags.push(heading_depth > 0);
        }

        (paragraphs, images, paragraph_links, anchors, heading_flags, inline_images)
    }

    /// 从 HTML 内容中提取标题
    pub(super) fn extract_title(&self, html: &str) -> String {
        let mut reader = Reader::from_str(html);
        reader.config_mut().trim_text(true);

        let mut h_title = String::new();
        let mut html_title = String::new();
        let mut depth: u8 = 0;
        let mut in_title_tag = false;
        let mut in_class_title = false;
        let mut class_title_depth: u8 = 0;

        let mut buffer = Vec::new();

        loop {
            match reader.read_event_into(&mut buffer) {
                Ok(Event::Start(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();

                    // h1-h3 标签
                    if name.starts_with(&b"h1"[..])
                        || name.starts_with(&b"h2"[..])
                        || name.starts_with(&b"h3"[..])
                    {
                        if depth == 0 {
                            h_title.clear();
                        }
                        depth += 1;
                    }

                    // <title> 标签
                    if name.starts_with(&b"title"[..]) {
                        in_title_tag = true;
                        html_title.clear();
                    }

                    // class 包含 "title" 或 "heading" 的元素
                    if depth == 0 && !in_class_title {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                if attr.key.as_ref() == b"class" {
                                    if let Ok(val) = std::str::from_utf8(attr.value.as_ref()) {
                                        if val.contains("title") || val.contains("heading") {
                                            in_class_title = true;
                                            class_title_depth = 1;
                                            h_title.clear();
                                        }
                                    }
                                }
                            }
                        }
                    } else if in_class_title {
                        class_title_depth += 1;
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();

                    if name.starts_with(&b"h1"[..])
                        || name.starts_with(&b"h2"[..])
                        || name.starts_with(&b"h3"[..])
                    {
                        if depth > 0 {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                    }

                    if name.starts_with(&b"title"[..]) {
                        in_title_tag = false;
                    }

                    if in_class_title {
                        class_title_depth = class_title_depth.saturating_sub(1);
                        if class_title_depth == 0 {
                            in_class_title = false;
                            if !h_title.is_empty() {
                                break;
                            }
                        }
                    }
                }
                Ok(Event::Text(ref e)) => {
                    if let Ok(text) = e.unescape() {
                        if depth > 0 {
                            h_title.push_str(&text);
                        }
                        if in_title_tag {
                            html_title.push_str(&text);
                        }
                        if in_class_title {
                            h_title.push_str(&text);
                        }
                    }
                }
                Ok(Event::Eof) => break,
                _ => {}
            }
            buffer.clear();
        }

        let result = if !h_title.is_empty() {
            h_title.trim().to_string()
        } else if !html_title.is_empty() {
            html_title.trim().to_string()
        } else {
            String::new()
        };

        Self::clean_title(&result)
    }

    /// 清理标题：去除多余空白和编号前缀
    pub(super) fn clean_title(title: &str) -> String {
        use std::sync::LazyLock;

        // 中文章节前缀：第X章/节/回/卷（支持中文数字和阿拉伯数字）
        static RE_CN_CHAPTER: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"^第[一二三四五六七八九十百千零\d]+[章节回卷]\s*").unwrap()
        });
        // 英文章节前缀：Chapter N / CHAPTER N / CH N
        static RE_EN_CHAPTER: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"(?i)^(?:chapter|ch)\s*\d*\s*[:.\-–—\s]*").unwrap()
        });
        // 数字+点号前缀：如 "1. 标题"
        static RE_NUM_DOT: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"^\d+\.\s+").unwrap()
        });

        let cleaned: String = title.split_whitespace().collect::<Vec<&str>>().join(" ");
        let trimmed = cleaned.trim();
        if trimmed.is_empty() {
            return String::new();
        }

        for re in [&RE_CN_CHAPTER, &RE_EN_CHAPTER, &RE_NUM_DOT] {
            if let Some(m) = re.find(trimmed) {
                let rest = trimmed[m.end()..].trim();
                if !rest.is_empty() {
                    return rest.to_string();
                }
            }
        }

        trimmed.to_string()
    }

    /// 判断段落是否为 XML/SVG 噪声
    pub(super) fn is_xml_noise(text: &str) -> bool {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return false;
        }
        if trimmed.starts_with("<?xml")
            || trimmed.starts_with("<svg")
            || trimmed.starts_with("<html")
            || trimmed.starts_with("<body")
            || trimmed.starts_with("<metadata")
        {
            return true;
        }
        let tag_count = trimmed.matches('<').count();
        let total = trimmed.chars().count();
        if total > 0 && tag_count > 0 {
            let ratio = tag_count as f64 / total as f64;
            if ratio > 0.08 && total < 200 {
                return true;
            }
        }
        false
    }

    /// 判断提取的文本是否为仅含孤立封面标记的空内容
    pub(super) fn is_cover_only_text(text: &str) -> bool {
        let trimmed = text.trim();
        let cover_labels = ["Cover", "COVER", "封面", "COVER PAGE"];
        let is_only_cover_label = cover_labels.iter().any(|l| trimmed == *l);
        if is_only_cover_label {
            return true;
        }
        // 如果所有段落都是 XML 噪声，也视为空
        if !trimmed.is_empty()
            && trimmed
                .split("\n\n")
                .all(|p| p.trim().is_empty() || Self::is_xml_noise(p))
        {
            return true;
        }
        false
    }
}
