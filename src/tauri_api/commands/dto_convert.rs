use crate::domain::book_format::BookFormat;
use crate::domain::paragraph::TextLink;
use crate::tts::config::TtsConfig;
use crate::tts::types::TtsProviderKind;

use super::super::dto::*;

fn format_str(format: &BookFormat) -> &'static str {
    match format {
        BookFormat::Epub => "epub",
        BookFormat::Txt => "txt",
    }
}

fn links_to_dto(links: &[TextLink]) -> Vec<ReaderTextLinkDto> {
    links
        .iter()
        .map(|l| ReaderTextLinkDto {
            start: l.start,
            end: l.end,
            href: l.href.clone(),
            title: l.title.clone(),
        })
        .collect()
}

pub fn item_to_dto(item: &crate::domain::library_item::LibraryItem) -> LibraryBookCardDto {
    use crate::services::asset_service::AssetService;
    let svc = crate::services::asset_service_impl::AssetServiceImpl::new();
    let cover_url = item
        .cover_cache_key
        .as_deref()
        .and_then(|ext| {
            let p = crate::storage::paths::cover_cache_path(&item.book_id, ext);
            p.exists().then(|| p.to_str().map(|s| s.to_string())).flatten()
        })
        .or_else(|| svc.cover_path(&item.book_id).and_then(|p| p.to_str().map(|s| s.to_string())));
    LibraryBookCardDto {
        book_id: item.book_id.clone(),
        title: item.title.clone(),
        author: item.author.clone(),
        format: format_str(&item.format).to_string(),
        cover_url,
        progress_percent: item.progress_percent,
        chapter_count: item.chapter_count,
        last_opened_at: item.last_opened_at.clone(),
        imported_at: item.imported_at.clone(),
        file_ok: item.file_health == crate::domain::library_item::FileHealth::Ok,
    }
}

pub fn toc_to_dto(toc: &crate::domain::toc_item::TocItem) -> TocItemDto {
    TocItemDto {
        id: toc.id.clone(),
        title: toc.title.clone(),
        chapter_index: toc.chapter_index,
        href: toc.href.clone(),
        depth: toc.depth as usize,
        children: toc.children.iter().map(toc_to_dto).collect(),
    }
}

pub fn block_to_dto(
    block: &crate::domain::chapter_block::ChapterBlock,
    block_index: usize,
) -> ReaderBlockDto {
    match block {
        crate::domain::chapter_block::ChapterBlock::Paragraph(p) => ReaderBlockDto::Paragraph {
            index: p.index,
            block_id: format!("p-{}", p.index),
            text: p.text.clone(),
            kind: format!("{:?}", p.kind).to_lowercase(),
            links: links_to_dto(&p.links),
        },
        crate::domain::chapter_block::ChapterBlock::Heading(p) => ReaderBlockDto::Heading {
            index: p.index,
            block_id: format!("h-{}", p.index),
            text: p.text.clone(),
            kind: format!("{:?}", p.kind).to_lowercase(),
            links: links_to_dto(&p.links),
        },
        crate::domain::chapter_block::ChapterBlock::Quote(p) => ReaderBlockDto::Quote {
            index: p.index,
            block_id: format!("q-{}", p.index),
            text: p.text.clone(),
            links: links_to_dto(&p.links),
        },
        crate::domain::chapter_block::ChapterBlock::Image(img) => ReaderBlockDto::Image {
            index: img.index,
            block_id: format!("img-{}", img.index),
            asset_id: img.asset_id.clone(),
            alt_text: img.alt_text.clone(),
            caption: img.caption.clone(),
        },
        crate::domain::chapter_block::ChapterBlock::Separator => ReaderBlockDto::Separator {
            block_id: format!("sep-{}", block_index),
        },
    }
}

pub fn build_reader_book_dto(book: &crate::domain::book::Book) -> ReaderBookDto {
    ReaderBookDto {
        book_id: book.id.clone(),
        title: book.metadata.title.clone(),
        author: book.metadata.author.clone(),
        format: format_str(&book.format).to_string(),
        chapter_count: book.chapters.len(),
        toc: book.toc.iter().map(toc_to_dto).collect(),
    }
}

pub fn tts_config_to_dto(config: &TtsConfig) -> TtsConfigDto {
    TtsConfigDto {
        enabled: config.enabled,
        provider: format!("{:?}", config.provider).to_lowercase(),
        has_api_key: config.api_key.is_some(),
        api_key: None,
        base_url: config.base_url.clone(),
        model: config.model.clone(),
        voice_id: config.voice_id.clone(),
    }
}

pub fn dto_to_tts_config(dto: &TtsConfigDto, api_key: Option<String>) -> TtsConfig {
    let provider = match dto.provider.as_str() {
        #[cfg(feature = "tts-aliyun")]
        "aliyun" => TtsProviderKind::Aliyun,
        _ => TtsProviderKind::Xiaomi,
    };
    TtsConfig {
        enabled: dto.enabled,
        provider,
        api_key: dto.api_key.clone().or(api_key),
        base_url: dto.base_url.clone(),
        model: dto.model.clone(),
        voice_id: dto.voice_id.clone(),
    }
}

/// Round `idx` down to the nearest valid UTF-8 char boundary in `s`.
pub fn snap_to_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

/// Read a file by absolute path and return a base64 data URI.
pub fn read_file_to_data_uri(path: &str) -> Result<Option<String>, String> {
    use base64::Engine;
    let p = std::path::Path::new(path);
    if !p.exists() {
        return Ok(None);
    }
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "application/octet-stream",
    };
    let bytes = std::fs::read(p).map_err(|e| e.to_string())?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(Some(format!("data:{};base64,{}", mime, b64)))
}
