/*!
解析器工厂模块

根据文件扩展名创建相应的解析器实例，实现解析器的动态选择。
*/

use crate::domain::book_format::BookFormat;
use crate::parser::parsers::base::BookParser;
use crate::parser::parsers::epub::EpubParser;
use crate::parser::parsers::txt::TxtParser;

/// 解析器工厂
///
/// 根据文件扩展名选择合适的解析器
pub struct ParserFactory;

impl ParserFactory {
    /// 根据文件路径获取对应的解析器
    ///
    /// # 参数
    /// * `path` - 文件路径
    ///
    /// # 返回值
    /// * `Some(Box<dyn BookParser>)` - 找到对应格式的解析器
    /// * `None` - 不支持的文件格式
    pub fn get_parser(path: &str) -> Option<Box<dyn BookParser>> {
        match BookFormat::from_path(path)? {
            BookFormat::Epub => Some(Box::new(EpubParser::new())),
            BookFormat::Txt => Some(Box::new(TxtParser::new())),
        }
    }
}
