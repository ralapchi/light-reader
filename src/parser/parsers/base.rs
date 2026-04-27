// 解析结果结构体
pub struct ParseResult {
    pub content: Vec<String>,
    pub chapter_titles: Vec<String>,
}

// 解析器 trait
pub trait BookParser {
    fn parse(&self, path: &str) -> Result<ParseResult, String>;
}
