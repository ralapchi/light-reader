use crate::parser::parsers::base::BookParser;
use crate::parser::parsers::epub::EpubParser;
use crate::parser::parsers::txt::TxtParser;

// 解析器工厂
pub struct ParserFactory;

impl ParserFactory {
    pub fn get_parser(path: &str) -> Option<Box<dyn BookParser>> {
        if path.ends_with(".epub") {
            Some(Box::new(EpubParser::new()))
        } else if path.ends_with(".txt") {
            Some(Box::new(TxtParser::new()))
        } else {
            // 可以添加更多格式的解析器
            None
        }
    }
}
