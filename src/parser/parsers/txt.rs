/*!
TXT 解析器模块

实现 TXT 格式书籍的解析逻辑，将文本按空行分割为段落。
*/

use crate::parser::parsers::base::{BookParser, ParseResult};
use std::fs::File;
use std::io::Read;

/// TXT 解析器
/// 
/// 负责解析 TXT 格式的书籍文件
pub struct TxtParser;

impl TxtParser {
    /// 创建 TXT 解析器实例
    pub fn new() -> Self {
        Self
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

        // 简单处理：按空行分割段落
        let content: Vec<String> = content_str.split("\n\n").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
        let chapter_titles = vec!["文本文件".to_string()];

        Ok(ParseResult {
            content,
            chapter_titles,
        })
    }
}
