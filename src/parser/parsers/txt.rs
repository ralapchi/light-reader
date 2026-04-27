use crate::parser::parsers::base::{BookParser, ParseResult};
use std::fs::File;
use std::io::Read;

// TXT 解析器
pub struct TxtParser;

impl TxtParser {
    pub fn new() -> Self {
        Self
    }
}

impl BookParser for TxtParser {
    fn parse(&self, path: &str) -> Result<ParseResult, String> {
        let mut file = File::open(path).map_err(|e| format!("文件打开失败: {}", e))?;
        let mut content_str = String::new();
        file.read_to_string(&mut content_str).map_err(|e| format!("文件读取失败: {}", e))?;

        // 简单处理：按空行分割段落
        let content: Vec<String> = content_str.split("\n\n").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let chapter_titles = vec!["文本文件".to_string()];

        Ok(ParseResult {
            content,
            chapter_titles,
        })
    }
}
