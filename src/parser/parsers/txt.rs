/*!
TXT 解析器模块

实现 TXT 格式书籍的解析逻辑，支持章节检测和段落分割。
*/

use crate::domain::app_error::{AppError, AppResult};
use crate::domain::error_codes;
use crate::parser::parsers::base::{BookParser, ParseResult};
use std::fs::File;
use std::io::Read;

/// 检测编码并解码为 UTF-8 字符串
///
/// 检测顺序：BOM (UTF-8/UTF-16) → UTF-8 有效性 → GBK 回退
fn decode_text(raw: &[u8]) -> String {
    // UTF-8 BOM
    if raw.starts_with(&[0xEF, 0xBB, 0xBF]) {
        return String::from_utf8_lossy(&raw[3..]).into_owned();
    }
    // UTF-16 LE/BE BOM
    if raw.starts_with(&[0xFF, 0xFE]) || raw.starts_with(&[0xFE, 0xFF]) {
        let (cow, _, _) = encoding_rs::UTF_16LE.decode(raw);
        return cow.into_owned();
    }

    // 尝试 UTF-8
    if let Ok(s) = std::str::from_utf8(raw) {
        return s.to_string();
    }

    // 回退到 GBK（中文 TXT 最常见的非 UTF-8 编码）
    log::info!("文件非 UTF-8 编码，尝试 GBK 解码");
    let (cow, _, had_errors) = encoding_rs::GBK.decode(raw);
    if had_errors {
        log::warn!("GBK 解码存在不可识别字符");
    }
    cow.into_owned()
}

/// Skip a number prefix (Arabic digits or Chinese numerals), returning the remainder.
fn skip_number(s: &str) -> &str {
    // Arabic digits
    if let Some(rest) = s.strip_prefix(|c: char| c.is_ascii_digit()) {
        return rest.trim_start_matches(|c: char| c.is_ascii_digit());
    }
    // Chinese numerals
    strip_cn_number_prefix(s).unwrap_or(s)
}

/// Strip a Chinese number prefix (一~九十九) from the input, returning the remainder.
pub(crate) fn strip_cn_number_prefix(s: &str) -> Option<&str> {
    let cn_units = [
        "二十", "三十", "四十", "五十", "六十", "七十", "八十", "九十", "十",
    ];
    let cn_digits = ["一", "二", "三", "四", "五", "六", "七", "八", "九"];

    // Try unit + optional digit (e.g. "二十一", "十")
    for unit in cn_units {
        if let Some(rest) = s.strip_prefix(unit) {
            for digit in cn_digits {
                if let Some(r) = rest.strip_prefix(digit) {
                    return Some(r);
                }
            }
            return Some(rest);
        }
    }

    // Single digit (e.g. "一")
    for digit in cn_digits {
        if let Some(rest) = s.strip_prefix(digit) {
            return Some(rest);
        }
    }

    None
}

/// TXT 解析器
///
/// 负责解析 TXT 格式的书籍文件，支持自动章节检测
pub struct TxtParser;

impl TxtParser {
    /// 创建 TXT 解析器实例
    pub fn new() -> Self {
        Self
    }

    /// 检测行是否为章节标题
    fn is_chapter_line(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.len() > 80 {
            return None;
        }

        // 中文章节模式：第X章、第X回、第X节、第X卷（支持阿拉伯数字和中文数字）
        if trimmed.starts_with("第") {
            let after_first = &trimmed["第".len()..];
            // Skip digits (Arabic or Chinese)
            let rest = skip_number(after_first);
            if !rest.is_empty() && rest == after_first {
                // No number found after 第
            } else if rest.starts_with("章")
                || rest.starts_with("回")
                || rest.starts_with("节")
                || rest.starts_with("卷")
                || rest.starts_with("部分")
                || rest.starts_with("篇")
            {
                // After marker, remaining should be short title or empty (not body text)
                let marker_len = if rest.starts_with("部分") {
                    "部分".len()
                } else if rest.starts_with("篇") {
                    "篇".len()
                } else {
                    "章".len()
                };
                let after_marker = &rest[marker_len..];
                let after_trimmed = after_marker.trim();
                if after_trimmed.is_empty() || after_trimmed.chars().count() <= 20 {
                    return Some(trimmed.to_string());
                }
            }
        }

        // 特殊中文章节词：序章、终章、番外、楔子、尾声、引子、后记、前言
        let special_cn = [
            "序章", "终章", "番外", "楔子", "尾声", "引子", "后记", "前言", "序言", "序节",
        ];
        for word in special_cn {
            if trimmed.starts_with(word) {
                return Some(trimmed.to_string());
            }
        }

        // 英文章节模式：Chapter X, CHAPTER X, Part X, PART X（含冒号标题）
        let upper = trimmed.to_uppercase();
        if upper.starts_with("CHAPTER ") || upper.starts_with("PART ") {
            return Some(trimmed.to_string());
        }

        // 数字开头模式：1. 标题、12、标题（阿拉伯数字 + 分隔符）
        if trimmed.len() >= 3 {
            if let Some(first_char) = trimmed.chars().next() {
                if first_char.is_ascii_digit() {
                    let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
                    if rest.starts_with('.') || rest.starts_with('、') || rest.starts_with(' ') {
                        return Some(trimmed.to_string());
                    }
                }
            }
        }

        // 中文数字开头：一、二、...、二十、三十...
        if let Some(rest) = strip_cn_number_prefix(trimmed) {
            if rest.starts_with('、') || rest.starts_with('，') || rest.starts_with(',') {
                return Some(trimmed.to_string());
            }
        }

        None
    }

    /// 按章节分割文本
    fn split_by_chapters(lines: Vec<&str>) -> (Vec<String>, Vec<String>) {
        let mut chapters: Vec<String> = Vec::new();
        let mut titles: Vec<String> = Vec::new();
        let mut current_lines: Vec<String> = Vec::new();
        let mut current_title = String::from("前言");

        for line in lines {
            if let Some(title) = Self::is_chapter_line(line) {
                // 保存当前章节（如果有内容）
                if !current_lines.is_empty() {
                    let content = current_lines.join("\n").trim().to_string();
                    if !content.is_empty() {
                        chapters.push(content);
                        titles.push(current_title.clone());
                    }
                }
                // 开始新章节
                current_title = title;
                current_lines = Vec::new();
            } else {
                current_lines.push(line.to_string());
            }
        }

        // 保存最后一个章节
        if !current_lines.is_empty() {
            let content = current_lines.join("\n").trim().to_string();
            if !content.is_empty() {
                chapters.push(content);
                titles.push(current_title);
            }
        }

        (chapters, titles)
    }
}

impl BookParser for TxtParser {
    /// 解析 TXT 文件
    ///
    /// # 参数
    /// * `path` - TXT 文件路径
    ///
    /// # 返回值
    /// * `Ok(ParseResult)` - 解析成功，返回解析结果
    /// * `Err(AppError)` - 解析失败，返回结构化错误
    fn parse(&self, path: &str) -> AppResult<ParseResult> {
        let mut file = File::open(path).map_err(|e| {
            let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件打开失败", e.to_string());
            err.recoverable = true;
            err
        })?;

        // 读取原始字节，检测编码后解码为 UTF-8
        let mut raw_bytes = Vec::new();
        file.read_to_end(&mut raw_bytes).map_err(|e| {
            let mut err = AppError::with_detail(error_codes::FILE_OPEN_FAILED, "文件读取失败", e.to_string());
            err.recoverable = true;
            err
        })?;

        let content_str = decode_text(&raw_bytes);

        let lines: Vec<&str> = content_str.lines().collect();

        // 检测是否包含章节
        let has_chapters = lines
            .iter()
            .any(|line| Self::is_chapter_line(line).is_some());

        let (content, chapter_titles) = if has_chapters {
            // 按章节分割
            Self::split_by_chapters(lines)
        } else {
            // 回退到按空行分割，全文作为单章
            let content = vec![content_str.trim().to_string()]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect();
            let chapter_titles = vec!["文本文件".to_string()];
            (content, chapter_titles)
        };

        Ok(ParseResult {
            content,
            chapter_titles,
            spine_hrefs: Vec::new(),
            toc: None,
            metadata: None,
            warnings: Vec::new(),
            cover_image: None,
            cover_media_type: None,
            image_assets: Vec::new(),
            chapter_image_blocks: Vec::new(),
            chapter_links: Vec::new(),
            chapter_anchors: Vec::new(),
            chapter_heading_flags: Vec::new(),
        })
    }
}
