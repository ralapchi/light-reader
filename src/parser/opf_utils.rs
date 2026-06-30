/*!
OPF 解析工具函数

提供轻量级的 OPF (Open Packaging Format) 解析功能，用于从 EPUB 文件中提取
封面信息和 OPF 路径。使用简单的字符串解析而非完整 XML 解析器，适用于
只需提取少量信息的场景（如封面缓存）。
*/

use std::collections::HashMap;

use crate::parser::epub_assets;

/// 从 container.xml 内容中提取 OPF 文件路径
///
/// 查找 `media-type="application/oebps-package+xml"` 的 rootfile 元素，
/// 返回其 `full-path` 属性值。
///
/// # 参数
/// * `container_xml` - container.xml 文件内容
///
/// # 返回值
/// * `Some(String)` - OPF 文件路径
/// * `None` - 解析失败
pub fn extract_opf_path(container_xml: &str) -> Option<String> {
    let needle = "application/oebps-package\x2bxml";
    let attr_key = "full-path=\"";
    for line in container_xml.lines() {
        if line.contains(needle) {
            if let Some(start) = line.find(attr_key) {
                let start = start + attr_key.len();
                if let Some(end) = line[start..].find('"') {
                    return Some(line[start..start + end].to_string());
                }
            }
        }
    }
    None
}

/// 解析 OPF 内容，提取 manifest 映射和封面图片的 href
///
/// 支持 EPUB 2（`<meta name="cover" content="..."/>`）和
/// EPUB 3（`properties="cover-image"`）两种封面声明方式，
/// 并包含多级回退策略。
///
/// # 参数
/// * `opf_content` - OPF 文件内容
///
/// # 返回值
/// * `Some((HashMap, String))` - (manifest id→href 映射, 封面图片 href)
/// * `None` - 未找到封面
pub fn parse_opf_cover(
    opf_content: &str,
) -> Option<(HashMap<String, String>, String)> {
    let mut manifest: HashMap<String, String> = HashMap::new();
    let mut cover_id: Option<String> = None;
    let mut cover_href: Option<String> = None;

    // Simple line-by-line parsing
    let mut in_manifest = false;
    for line in opf_content.lines() {
        let trimmed = line.trim();

        // Detect manifest section
        if trimmed.contains("<manifest") {
            in_manifest = true;
        }
        if trimmed.contains("</manifest") {
            in_manifest = false;
        }

        // Parse manifest items
        if in_manifest && trimmed.contains("<item") {
            let id = extract_attr(trimmed, "id");
            let href = extract_attr(trimmed, "href");
            let properties = extract_attr(trimmed, "properties");

            if let (Some(id), Some(href)) = (id, href) {
                let href = epub_assets::normalize_href(&href);
                manifest.insert(id.clone(), href.clone());

                // EPUB 3: properties="cover-image"
                if properties.as_deref().unwrap_or("").contains("cover-image") {
                    cover_href = Some(href);
                }
            }
        }

        // EPUB 2: <meta name="cover" content="cover-image-id"/>
        if trimmed.contains("<meta") {
            let name = extract_attr(trimmed, "name");
            let content = extract_attr(trimmed, "content");
            if name.as_deref() == Some("cover") {
                if let Some(content) = content {
                    cover_id = Some(content);
                }
            }
        }
    }

    // Resolve EPUB 2 cover ID to href
    if cover_href.is_none() {
        if let Some(ref id) = cover_id {
            // Try exact match first, then prefix match
            cover_href = manifest.get(id).cloned();
            if cover_href.is_none() {
                // Try __id__: prefixed marker
                if let Some(actual_id) = id.strip_prefix("__id__:") {
                    cover_href = manifest.get(actual_id).cloned();
                }
            }
            // Fallback: find any image manifest item with "cover" in id
            if cover_href.is_none() {
                for (mid, mhref) in &manifest {
                    if mid.to_lowercase().contains("cover") && epub_assets::is_image_href(mhref) {
                        cover_href = Some(mhref.clone());
                        break;
                    }
                }
            }
        }
    }

    // Last fallback: find any image with "cover" in path
    if cover_href.is_none() {
        for (_, href) in &manifest {
            if href.to_lowercase().contains("cover") && epub_assets::is_image_href(href) {
                cover_href = Some(href.clone());
                break;
            }
        }
    }

    cover_href.map(|h| (manifest, h))
}

/// 从 XML 标签字符串中提取指定属性的值
///
/// 支持双引号和单引号两种属性值格式。
///
/// # 参数
/// * `tag` - 包含属性的标签字符串
/// * `attr` - 要提取的属性名
///
/// # 返回值
/// * `Some(String)` - 属性值
/// * `None` - 未找到该属性
pub fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    // Try quoted: attr="value"
    let pattern = format!("{}=\"", attr);
    if let Some(start) = tag.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = tag[start..].find('"') {
            return Some(tag[start..start + end].to_string());
        }
    }
    // Try single-quoted: attr='value'
    let pattern = format!("{}='", attr);
    if let Some(start) = tag.find(&pattern) {
        let start = start + pattern.len();
        if let Some(end) = tag[start..].find('\'') {
            return Some(tag[start..start + end].to_string());
        }
    }
    None
}
