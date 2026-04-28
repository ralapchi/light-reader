/*!
解析器基础模块

定义解析器的通用接口和结果结构体，为所有具体解析器提供统一的标准。
*/

/// 解析结果结构体
/// 
/// 存储解析后的书籍内容和章节标题
pub struct ParseResult {
    /// 书籍内容，按章节存储的文本内容
    pub content: Vec<String>,
    /// 章节标题列表
    pub chapter_titles: Vec<String>,
}

/// 解析器 trait
/// 
/// 所有书籍解析器都需要实现的接口
pub trait BookParser {
    /// 解析书籍文件
    /// 
    /// # 参数
    /// * `path` - 书籍文件路径
    /// 
    /// # 返回值
    /// * `Ok(ParseResult)` - 解析成功，返回解析结果
    /// * `Err(String)` - 解析失败，返回错误信息
    fn parse(&self, path: &str) -> Result<ParseResult, String>;
}
