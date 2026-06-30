/*!
EPUB 目录解析与树构建

负责解析 EPUB3 nav.xhtml、EPUB2 toc.ncx 以及构建层级目录树。
*/

use crate::domain::toc_item::TocItem;
use quick_xml::Reader;
use quick_xml::events::Event;
use std::collections::HashMap;

use super::epub_parser::EpubParser;

impl EpubParser {
    /// 从扁平 TOC 列表构建 href → title 映射（用于优先使用 TOC 标题）
    pub(super) fn build_toc_title_map(toc: &[TocItem]) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for item in toc {
            if let Some(href) = &item.href {
                // 去掉 fragment（# 后面的部分），只保留文件路径
                let base_href = href.split('#').next().unwrap_or(href);
                if !item.title.is_empty() {
                    map.insert(base_href.to_string(), item.title.clone());
                }
            }
            // 递归处理子项
            let child_map = Self::build_toc_title_map(&item.children);
            map.extend(child_map);
        }
        map
    }

    /// 解析 EPUB3 nav.xhtml 文件（Navigation Document）
    ///
    /// # 参数
    /// * `content` - nav.xhtml 文件内容
    /// * `base_href` - 基础路径
    ///
    /// # 返回值
    /// * `Vec<TocItem>` - 解析得到的目录树
    pub(super) fn parse_nav_document(&self, content: &str, base_href: &str) -> Vec<TocItem> {
        // 尝试严格匹配（要求 type/epub:type="toc"）
        let items = self.parse_nav_with_filter(content, base_href, true);
        if !items.is_empty() {
            return items;
        }
        // T14 E-1: 回退模式 - 不限制 nav 类型属性
        self.parse_nav_with_filter(content, base_href, false)
    }

    /// 解析 EPUB3 nav.xhtml 目录。
    /// `strict_type`: 为 true 时仅匹配含 type/epub:type="toc" 的 nav 元素
    fn parse_nav_with_filter(
        &self,
        content: &str,
        base_href: &str,
        strict_type: bool,
    ) -> Vec<TocItem> {
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
                    if name.eq_ignore_ascii_case(b"nav") {
                        if strict_type {
                            for attr in e.attributes() {
                                if let Ok(attr) = attr {
                                    let attr_name = attr.key.as_ref();
                                    if attr_name.eq_ignore_ascii_case(b"type")
                                        || attr_name.eq_ignore_ascii_case(b"epub:type")
                                    {
                                        if let Ok(value) = std::str::from_utf8(attr.value.as_ref())
                                        {
                                            if value.contains("toc") {
                                                in_nav = true;
                                            }
                                        }
                                    }
                                }
                            }
                        } else {
                            in_nav = true;
                        }
                    } else if in_nav && name.eq_ignore_ascii_case(b"ol") {
                        in_ol = true;
                        depth += 1;
                    } else if in_nav && in_ol && name.eq_ignore_ascii_case(b"a") {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.eq_ignore_ascii_case(b"href") {
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
                    if name.eq_ignore_ascii_case(b"nav") {
                        in_nav = false;
                        if !strict_type && !items.is_empty() {
                            // 非严格模式下找到第一个有意义 nav 即停止
                        }
                    } else if in_nav && name.eq_ignore_ascii_case(b"ol") {
                        in_ol = false;
                        depth = depth.saturating_sub(1);
                    } else if in_nav && in_ol && name.eq_ignore_ascii_case(b"a") {
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
    pub(super) fn parse_ncx(&self, content: &str, base_href: &str) -> Vec<TocItem> {
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
                    if name.eq_ignore_ascii_case(b"navMap") {
                        in_nav_map = true;
                    } else if in_nav_map && name.eq_ignore_ascii_case(b"navPoint") {
                        in_nav_point = true;
                        depth += 1;
                    } else if in_nav_point && name.eq_ignore_ascii_case(b"content") {
                        for attr in e.attributes() {
                            if let Ok(attr) = attr {
                                let attr_name = attr.key.as_ref();
                                if attr_name.eq_ignore_ascii_case(b"src") {
                                    if let Ok(value) = std::str::from_utf8(attr.value.as_ref()) {
                                        current_src = value.to_string();
                                    }
                                }
                            }
                        }
                    } else if in_nav_point && name.eq_ignore_ascii_case(b"text") {
                        collecting_text = true;
                        current_text.clear();
                    }
                }
                Ok(Event::End(ref e)) => {
                    let qname = e.name();
                    let name = qname.as_ref();
                    if name.eq_ignore_ascii_case(b"navMap") {
                        in_nav_map = false;
                    } else if name.eq_ignore_ascii_case(b"navPoint") {
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
                    } else if name.eq_ignore_ascii_case(b"text") {
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
    pub(super) fn build_toc_tree(&self, items: Vec<TocItem>) -> Vec<TocItem> {
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
                let parent = self.find_parent_mut(&mut root, &stack);
                let child_index = parent.children.len();
                parent.children.push(item);
                stack.push((depth, child_index));
            }
        }

        root
    }

    /// 辅助方法：根据栈找到可变父节点
    fn find_parent_mut<'a>(
        &self,
        root: &'a mut Vec<TocItem>,
        stack: &[(u8, usize)],
    ) -> &'a mut TocItem {
        let mut current = &mut root[stack[0].1];
        for &(_, idx) in &stack[1..] {
            current = &mut current.children[idx];
        }
        current
    }
}
