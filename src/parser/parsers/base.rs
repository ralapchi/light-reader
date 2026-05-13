/*!
解析器基础模块

定义解析器的通用接口和结果结构体，为所有具体解析器提供统一的标准。
*/

use crate::domain::book_metadata::BookMetadata;
use crate::domain::toc_item::TocItem;

/// 解析结果结构体
///
/// 存储解析后的书籍内容和章节标题
pub struct ParseResult {
    /// 书籍内容，按章节存储的文本内容
    pub content: Vec<String>,
    /// 章节标题列表
    pub chapter_titles: Vec<String>,
    /// spine 中每个章节的 href（与 content/chapter_titles 索引对应）
    pub spine_hrefs: Vec<String>,
    /// 结构化目录（可选，用于支持层级目录）
    pub toc: Option<Vec<TocItem>>,
    /// 书籍元信息（标题、作者等）
    pub metadata: Option<BookMetadata>,
    /// 解析过程中的警告信息
    pub warnings: Vec<String>,
    /// 封面图片原始字节（若可提取）
    pub cover_image: Option<Vec<u8>>,
    /// 封面媒体类型（如 "image/jpeg"）
    pub cover_media_type: Option<String>,
    /// 书籍内置图片资源索引
    pub image_assets: Vec<crate::domain::book_assets::BookImageAsset>,
    /// 每章节的图片块列表（与 content 索引对齐）
    pub chapter_image_blocks: Vec<Vec<(isize, crate::domain::chapter_block::InlineImageBlock)>>,
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
