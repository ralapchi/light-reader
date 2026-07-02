/*!
EPUB 解析器模块

实现 EPUB 格式书籍的解析逻辑，包括解析 container.xml、OPF 文件和 HTML 内容。
*/

use super::epub_content::HtmlExtractionResult;
use crate::domain::app_error::{AppError, AppResult};
use crate::domain::error_codes;
use crate::domain::paragraph::TextLink;
use crate::domain::toc_item::TocItem;
use crate::parser::epub_assets;
use crate::parser::parsers::base::{BookParser, ParseResult};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use zip::ZipArchive;

/// 判断标签是否为图片元素
pub(super) fn is_img_tag(name: &[u8]) -> bool {
    name.len() >= 3 && name[..3].eq_ignore_ascii_case(b"img")
        || name.len() >= 4 && name[name.len()-4..].eq_ignore_ascii_case(b":img")
        || name.len() >= 5 && name[name.len()-5..].eq_ignore_ascii_case(b"image")
}

/// 从标签属性中读取图片的 src、alt 和 style
pub(super) fn read_img_attrs(attrs: quick_xml::events::attributes::Attributes) -> (String, Option<String>, Option<String>) {
    let mut src = String::new();
    let mut alt = None;
    let mut style = None;
    for attr in attrs {
        if let Ok(attr) = attr {
            let an = attr.key.as_ref();
            let an_lower = an.to_ascii_lowercase();
            if an_lower.starts_with(b"src") || an_lower.starts_with(b"xlink:href") {
                if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                    src = v.to_string();
                }
            } else if an_lower.starts_with(b"alt") {
                if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                    if !v.trim().is_empty() {
                        alt = Some(v.to_string());
                    }
                }
            } else if an_lower.starts_with(b"style") {
                if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                    style = Some(v.to_string());
                }
            }
        }
    }
    (src, alt, style)
}

/// 从元素属性中提取 id/name 锚点
pub(super) fn record_anchor(
    attrs: quick_xml::events::attributes::Attributes,
    anchors: &mut Vec<(String, usize)>,
    para_index: usize,
) {
    for attr in attrs {
        if let Ok(attr) = attr {
            let an = attr.key.as_ref();
            let an_lower = an.to_ascii_lowercase();
            if an_lower.starts_with(b"id") || an_lower == b"name" {
                if let Ok(v) = std::str::from_utf8(attr.value.as_ref()) {
                    let v = v.trim().to_string();
                    if !v.is_empty() {
                        anchors.push((v, para_index));
                    }
                }
            }
        }
    }
}

/// EPUB 解析器
///
/// 负责解析 EPUB 格式的书籍文件
pub struct EpubParser;

/// 章节构建结果
struct ChapterBuildResult {
    content: Vec<String>,
    chapter_titles: Vec<String>,
    spine_hrefs: Vec<String>,
    image_assets: Vec<crate::domain::book_assets::BookImageAsset>,
    chapter_image_blocks: Vec<Vec<(isize, crate::domain::chapter_block::InlineImageBlock)>>,
    chapter_links: Vec<Vec<Vec<TextLink>>>,
    chapter_anchors: Vec<Vec<(String, usize)>>,
    chapter_heading_flags: Vec<Vec<bool>>,
}

impl EpubParser {
    /// 创建 EPUB 解析器实例
    pub fn new() -> Self {
        Self
    }

    /// 判断 spine 项是否应被过滤（非正文内容）
    pub(super) fn is_non_body_spine_item(href: &str) -> bool {
        let lower = href.to_lowercase();
        let filename = lower.rsplit('/').next().unwrap_or(&lower);
        let non_body_patterns = [
            "nav.xhtml",
            "nav.html",
            "toc.xhtml",
            "toc.html",
            "toc.ncx",
            "cover.xhtml",
            "cover.html",
            "coverpage.xhtml",
            "cover-page.xhtml",
            "titlepage.xhtml",
            "title-page.xhtml",
            "titlepage.html",
            "title-page.html",
            "copyright.xhtml",
            "copyright-page.xhtml",
            "imprint.xhtml",
        ];
        non_body_patterns.iter().any(|p| filename == *p)
    }

    /// 获取或创建图片资源，返回 (asset_id, img_full_path)
    fn get_or_create_image_asset(
        &self,
        img_src: &str,
        chapter_dir: &str,
        alt_text: Option<String>,
        image_asset_ids_by_path: &mut HashMap<String, String>,
        image_assets: &mut Vec<crate::domain::book_assets::BookImageAsset>,
    ) -> (String, String) {
        let img_full_path = epub_assets::resolve_path(chapter_dir, img_src);

        let asset_id = image_asset_ids_by_path
            .entry(img_full_path.clone())
            .or_insert_with(|| {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                img_full_path.hash(&mut hasher);
                format!("img-{:016x}", hasher.finish())
            })
            .clone();

        if !image_assets.iter().any(|a| a.asset_path == img_full_path) {
            let mime = epub_assets::media_type_from_href(img_src);
            let cache_key = Some(format!(
                "{}.{}",
                asset_id,
                epub_assets::ext_from_href(img_src)
            ));
            image_assets.push(crate::domain::book_assets::BookImageAsset {
                asset_id: asset_id.clone(),
                source_href: epub_assets::normalize_href(img_src),
                asset_path: img_full_path.clone(),
                media_type: Some(mime.to_string()),
                cache_key,
                width_hint: None,
                height_hint: None,
                alt_text,
            });
        }

        (asset_id, img_full_path)
    }

    /// 解析 container.xml 和 OPF 文件，返回 (opf_base_path, manifest, spine_ids, opf_cover_href, metadata)
    fn parse_container_and_opf(
        &self,
        archive: &mut ZipArchive<BufReader<File>>,
        warnings: &mut Vec<String>,
    ) -> AppResult<(
        String,
        HashMap<String, String>,
        Vec<String>,
        Option<String>,
        Option<crate::domain::book_metadata::BookMetadata>,
    )> {
        let opf_path = {
            let mut container_content = String::new();
            if let Ok(mut container_file) = archive.by_name("META-INF/container.xml") {
                container_file.read_to_string(&mut container_content).ok();
            } else {
                warnings.push("缺少 META-INF/container.xml，使用默认路径".to_string());
            }
            self.parse_container_xml(&container_content)
                .unwrap_or_else(|| "content.opf".to_string())
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

        let (manifest, spine_ids, opf_cover_href) =
            self.parse_opf_file(&opf_content, &opf_base_path);
        let metadata = self.parse_opf_metadata(&opf_content);
        if metadata.is_none() {
            warnings.push("EPUB 缺少元信息，使用文件名作为标题".to_string());
        }

        Ok((opf_base_path, manifest, spine_ids, opf_cover_href, metadata))
    }

    /// 解析结构化目录（优先 nav.xhtml，回退 toc.ncx）
    fn parse_toc(
        &self,
        archive: &mut ZipArchive<BufReader<File>>,
        opf_base_path: &str,
        manifest: &HashMap<String, String>,
        warnings: &mut Vec<String>,
    ) -> Option<Vec<TocItem>> {
        let mut toc_items: Option<Vec<TocItem>> = None;

        // 1. 尝试 EPUB3 nav.xhtml
        if let Some(nav_href) = manifest
            .values()
            .find(|href| href.ends_with("nav.xhtml") || href.contains("nav"))
        {
            let nav_path = self.get_full_path(opf_base_path, nav_href);
            if let Ok(mut nav_file) = archive.by_name(&nav_path) {
                let mut nav_content = String::new();
                nav_file.read_to_string(&mut nav_content).ok();
                let items = self.parse_nav_document(&nav_content, opf_base_path);
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
                let ncx_path = self.get_full_path(opf_base_path, ncx_href);
                if let Ok(mut ncx_file) = archive.by_name(&ncx_path) {
                    let mut ncx_content = String::new();
                    ncx_file.read_to_string(&mut ncx_content).ok();
                    let items = self.parse_ncx(&ncx_content, opf_base_path);
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

        toc_items
    }

    /// 遍历 spine 项，提取 HTML 内容并构建章节数据
    fn build_chapters_from_spine(
        &self,
        archive: &mut ZipArchive<BufReader<File>>,
        opf_base_path: &str,
        manifest: &HashMap<String, String>,
        spine_ids: &[String],
        toc_title_map: &HashMap<String, String>,
        warnings: &mut Vec<String>,
    ) -> ChapterBuildResult {
        let mut content = Vec::new();
        let mut chapter_titles = Vec::new();
        let mut spine_hrefs = Vec::new();
        let mut image_assets: Vec<crate::domain::book_assets::BookImageAsset> = Vec::new();
        let mut chapter_image_blocks: Vec<
            Vec<(isize, crate::domain::chapter_block::InlineImageBlock)>,
        > = Vec::new();
        let mut chapter_links: Vec<Vec<Vec<TextLink>>> = Vec::new();
        let mut chapter_anchors: Vec<Vec<(String, usize)>> = Vec::new();
        let mut chapter_heading_flags: Vec<Vec<bool>> = Vec::new();
        let mut image_asset_ids_by_path: HashMap<String, String> = HashMap::new();

        for idref in spine_ids {
            if let Some(href) = manifest.get(idref.as_str()) {
                // 过滤非正文 spine 项（nav, cover, titlepage 等）
                if Self::is_non_body_spine_item(href) {
                    warnings.push(format!("跳过非正文项: {}", href));
                    continue;
                }
                let full_path = self.get_full_path(opf_base_path, href);
                // Read HTML content (release borrow on archive before image extraction)
                let html_content = {
                    let mut content = String::new();
                    if let Ok(mut f) = archive.by_name(&full_path) {
                        f.read_to_string(&mut content).ok();
                    }
                    content
                };
                if !html_content.is_empty() {
                    let HtmlExtractionResult {
                        paragraphs,
                        images: images_with_pos,
                        paragraph_links,
                        anchors: raw_anchors,
                        heading_flags: para_heading_flags,
                        inline_images: inline_images_from_html,
                    } = self.extract_html_with_positions(&html_content);
                    // 过滤 XML/SVG 噪声段落，同步更新链接和锚点索引
                    let keep: Vec<bool> =
                        paragraphs.iter().map(|p| !Self::is_xml_noise(p)).collect();
                    let mut reindex: Vec<Option<usize>> = vec![None; paragraphs.len()];
                    let mut next = 0usize;
                    for (i, &k) in keep.iter().enumerate() {
                        if k {
                            reindex[i] = Some(next);
                            next += 1;
                        }
                    }
                    let mut filtered_paragraphs: Vec<String> = paragraphs
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| keep[*i])
                        .map(|(_, p)| p)
                        .collect();
                    let mut filtered_links: Vec<Vec<TextLink>> = paragraph_links
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| keep[*i])
                        .map(|(_, links)| links)
                        .collect();
                    let filtered_heading_flags: Vec<bool> = para_heading_flags
                        .into_iter()
                        .enumerate()
                        .filter(|(i, _)| keep[*i])
                        .map(|(_, f)| f)
                        .collect();
                    let filtered_anchors: Vec<(String, usize)> = raw_anchors
                        .into_iter()
                        .filter_map(|(frag, old_idx)| {
                            reindex
                                .get(old_idx)
                                .and_then(|&new_idx| new_idx.map(|n| (frag, n)))
                        })
                        .collect();
                    let mut text_content = filtered_paragraphs.join("\n\n");
                    if !text_content.is_empty() && !Self::is_cover_only_text(&text_content) {
                        // 处理内联图片（生僻字替代），替换 PUA 占位符为 asset_id
                        // 必须在 push 到 chapter_links 之前完成，因为替换会改变链接位置
                        {
                            let chapter_full = self.get_full_path(opf_base_path, href);
                            let chapter_dir = std::path::Path::new(&chapter_full)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();
                            for (idx, img_src, _img_alt) in &inline_images_from_html {
                                let (asset_id, _path) = self.get_or_create_image_asset(
                                    img_src,
                                    &chapter_dir,
                                    _img_alt.clone(),
                                    &mut image_asset_ids_by_path,
                                    &mut image_assets,
                                );
                                let placeholder = format!("\u{E000}{}\u{E001}", idx);
                                let replacement = format!("\u{E000}{}\u{E001}", asset_id);
                                let placeholder_chars = placeholder.chars().count();
                                let len_diff = replacement.chars().count() as isize - placeholder_chars as isize;
                                for (para_text, para_links) in filtered_paragraphs.iter_mut().zip(filtered_links.iter_mut()) {
                                    if let Some(byte_pos) = para_text.find(&placeholder) {
                                        let char_pos = para_text[..byte_pos].chars().count();
                                        *para_text = para_text.replacen(&placeholder, &replacement, 1);
                                        if len_diff != 0 {
                                            for link in para_links.iter_mut() {
                                                if link.start > char_pos + placeholder_chars {
                                                    link.start = (link.start as isize + len_diff) as usize;
                                                    link.end = (link.end as isize + len_diff) as usize;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            text_content = filtered_paragraphs.join("\n\n");
                        }

                        spine_hrefs.push(href.clone());
                        chapter_links.push(filtered_links);
                        chapter_anchors.push(filtered_anchors);
                        chapter_heading_flags.push(filtered_heading_flags);

                        // Build image blocks from single-pass extraction
                        let mut img_blocks: Vec<(
                            isize,
                            crate::domain::chapter_block::InlineImageBlock,
                        )> = Vec::new();
                        for (img_idx, (pos, img_src, img_alt)) in
                            images_with_pos.into_iter().enumerate()
                        {
                            let chapter_full = self.get_full_path(opf_base_path, href);
                            let chapter_dir = std::path::Path::new(&chapter_full)
                                .parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_default();
                            let (asset_id, _path) = self.get_or_create_image_asset(
                                &img_src,
                                &chapter_dir,
                                img_alt,
                                &mut image_asset_ids_by_path,
                                &mut image_assets,
                            );
                            img_blocks.push((
                                pos,
                                crate::domain::chapter_block::InlineImageBlock {
                                    index: img_idx,
                                    asset_id,
                                    alt_text: None,
                                    caption: None,
                                    source_href: None,
                                    is_inline: false,
                                },
                            ));
                        }
                        chapter_image_blocks.push(img_blocks);
                        content.push(text_content);

                        // 优先使用 TOC 标题
                        let chapter_title =
                            if let Some(toc_title) = toc_title_map.get(href.as_str()) {
                                toc_title.clone()
                            } else {
                                let title = self.extract_title(&html_content);
                                if title.is_empty() {
                                    format!("章节 {}", content.len())
                                } else {
                                    title
                                }
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

        ChapterBuildResult {
            content,
            chapter_titles,
            spine_hrefs,
            image_assets,
            chapter_image_blocks,
            chapter_links,
            chapter_anchors,
            chapter_heading_flags,
        }
    }

    /// 提取封面图片和媒体类型
    fn extract_cover(
        &self,
        archive: &mut ZipArchive<BufReader<File>>,
        opf_base_path: &str,
        manifest: &HashMap<String, String>,
        opf_cover_href: &Option<String>,
    ) -> (Option<Vec<u8>>, Option<String>) {
        fn read_from_archive(
            archive: &mut zip::ZipArchive<std::io::BufReader<std::fs::File>>,
            opf_base_path: &str,
            href: &str,
        ) -> Option<(Vec<u8>, &'static str)> {
            let cover_path = epub_assets::resolve_path(opf_base_path, href);
            epub_assets::read_zip_entry(archive, &cover_path)
                .map(|buf| (buf, epub_assets::media_type_from_href(href)))
        }
        // Priority 1: OPF meta properties="cover-image" or <meta name="cover">
        let result = opf_cover_href
            .as_ref()
            .filter(|href| epub_assets::is_image_href(href))
            .and_then(|href| read_from_archive(archive, opf_base_path, href))
            .or_else(|| {
                // Priority 2: manifest item with "cover" in id/href (image files only)
                manifest
                    .iter()
                    .find_map(|(id, href)| {
                        let lower = format!("{}|{}", id.to_lowercase(), href.to_lowercase());
                        if lower.contains("cover") && epub_assets::is_image_href(href) {
                            Some(href.clone())
                        } else {
                            None
                        }
                    })
                    .and_then(|href| read_from_archive(archive, opf_base_path, &href))
            })
            .or_else(|| {
                // Priority 3: any image file with "cover" in its path
                manifest
                    .iter()
                    .find_map(|(_, href)| {
                        let h = href.to_lowercase();
                        if h.contains("cover") && epub_assets::is_image_href(href) {
                            Some(href.clone())
                        } else {
                            None
                        }
                    })
                    .and_then(|href| read_from_archive(archive, opf_base_path, &href))
            });
        match result {
            Some((buf, mime)) => (Some(buf), Some(mime.to_string())),
            _ => (None, None),
        }
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
    /// * `Err(AppError)` - 解析失败，返回结构化错误
    fn parse(&self, path: &str) -> AppResult<ParseResult> {
        let file = File::open(path).map_err(|e| {
            let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件打开失败", e.to_string());
            err.recoverable = true;
            err
        })?;
        let mut archive =
            ZipArchive::new(BufReader::new(file)).map_err(|e| AppError::with_detail(error_codes::FILE_OPEN_FAILED, "ZIP 解析失败", e.to_string()))?;
        let mut warnings = Vec::new();

        let (opf_base_path, manifest, spine_ids, opf_cover_href, metadata) =
            self.parse_container_and_opf(&mut archive, &mut warnings)?;
        let toc_items = self.parse_toc(&mut archive, &opf_base_path, &manifest, &mut warnings);
        let toc_title_map = toc_items
            .as_ref()
            .map(|toc| Self::build_toc_title_map(toc))
            .unwrap_or_default();
        let ch = self.build_chapters_from_spine(
            &mut archive,
            &opf_base_path,
            &manifest,
            &spine_ids,
            &toc_title_map,
            &mut warnings,
        );
        let (cover_image, cover_media_type) = self.extract_cover(
            &mut archive,
            &opf_base_path,
            &manifest,
            &opf_cover_href,
        );

        Ok(ParseResult {
            content: ch.content,
            chapter_titles: ch.chapter_titles,
            spine_hrefs: ch.spine_hrefs,
            toc: toc_items,
            metadata,
            warnings,
            cover_image,
            cover_media_type,
            image_assets: ch.image_assets,
            chapter_image_blocks: ch.chapter_image_blocks,
            chapter_links: ch.chapter_links,
            chapter_anchors: ch.chapter_anchors,
            chapter_heading_flags: ch.chapter_heading_flags,
        })
    }
}
