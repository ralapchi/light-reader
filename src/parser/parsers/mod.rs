/*!
解析器模块集合

包含所有解析器相关的子模块，统一导出解析器接口和实现。

子模块包括：
- base: 解析器基础接口和结果结构体
- epub: EPUB 格式解析器
- txt: TXT 格式解析器
- factory: 解析器工厂，用于根据文件扩展名选择解析器
*/

pub mod base;
pub mod epub;
pub mod factory;
pub mod txt;

pub use factory::ParserFactory;

#[cfg(test)]
mod tests;
