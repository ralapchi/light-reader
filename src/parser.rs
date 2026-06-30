/*!
解析器模块入口

作为解析器模块的根入口，导出所有解析器相关的类型和函数。

解析器模块采用策略模式设计，支持多种书籍格式的解析，包括 EPUB和TXT。
*/

// 解析器模块导出
pub mod epub_assets;
pub mod opf_utils;
pub mod parsers;

// 重新导出所有解析器相关的类型和函数
pub use parsers::*;
