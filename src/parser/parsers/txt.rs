/*!
TXT 解析器模块

实现 TXT 格式书籍的解析逻辑，支持章节检测和段落分割。
*/

use crate::parser::parsers::base::{BookParser, ParseResult};
use std::fs::File;
use std::io::Read;

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
        if trimmed.is_empty() || trimmed.len() > 100 {
            return None;
        }

        // 中文章节模式：第X章、第X回、第X节、第X卷
        let cn_patterns = ["第", "章", "回", "节", "卷"];
        if cn_patterns.iter().any(|p| trimmed.contains(p)) {
            // 检查是否匹配常见模式
            if trimmed.starts_with("第") && (trimmed.contains("章") || trimmed.contains("回") || trimmed.contains("节") || trimmed.contains("卷")) {
                return Some(trimmed.to_string());
            }
        }

        // 英文章节模式：Chapter X, CHAPTER X
        let upper = trimmed.to_uppercase();
        if upper.starts_with("CHAPTER ") {
            return Some(trimmed.to_string());
        }

        // 数字开头模式：1. 标题、一、标题
        if trimmed.len() >= 3 {
            let first_char = trimmed.chars().next().unwrap();
            if first_char.is_ascii_digit() {
                // 匹配 "1." 或 "1、" 等模式
                let rest = &trimmed[1..];
                if rest.starts_with('.') || rest.starts_with('、') || rest.starts_with(' ') {
                    return Some(trimmed.to_string());
                }
            }
        }

        // 中文数字开头：一、二、三、...
        let cn_numbers = ["一", "二", "三", "四", "五", "六", "七", "八", "九", "十",
                          "十一", "十二", "十三", "十四", "十五", "十六", "十七", "十八", "十九", "二十"];
        for num in cn_numbers {
            if trimmed.starts_with(num) {
                let rest = &trimmed[num.len()..];
                if rest.starts_with('、') || rest.starts_with('，') || rest.starts_with(',') {
                    return Some(trimmed.to_string());
                }
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
    /// * `Err(String)` - 解析失败，返回错误信息
    fn parse(&self, path: &str) -> Result<ParseResult, String> {
        let mut file = File::open(path).map_err(|e| format!("文件打开失败: {}", e))?;
        let mut content_str = String::new();
        file.read_to_string(&mut content_str).map_err(|e| format!("文件读取失败: {}", e))?;

        let lines: Vec<&str> = content_str.lines().collect();

        // 检测是否包含章节
        let has_chapters = lines.iter().any(|line| Self::is_chapter_line(line).is_some());

        let (content, chapter_titles) = if has_chapters {
            // 按章节分割
            Self::split_by_chapters(lines)
        } else {
            // 回退到按空行分割，全文作为单章
            let content: Vec<String> = content_str
                .split("\n\n")
                .map(|s| s.trim().to_string())
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
        })
    }
}
