/// Chapter building utilities: paragraph parsing, kind inference, TOC mapping.
/// Extracted from `app/compat.rs` to keep the adapter focused on coordination.
use crate::domain::chapter::Chapter;
use crate::domain::chapter_block::ChapterBlock;
use crate::domain::chapter_block::InlineImageBlock;
use crate::domain::paragraph::Paragraph;
use crate::domain::paragraph_kind::ParagraphKind;
use crate::domain::toc_item::TocItem;

pub(crate) const INDENT_MARKER: &str = "\x01INDENT\x01";

pub(crate) fn build_chapter(
    index: usize,
    title: &str,
    text: &str,
    img_blocks: &[(isize, InlineImageBlock)],
    source_href: Option<&str>,
    paragraph_links: &[Vec<crate::domain::paragraph::TextLink>],
    anchors: Vec<(String, usize)>,
) -> Chapter {
    let mut line_number = 0usize;
    let paragraphs = text
        .split("\n\n")
        .enumerate()
        .filter_map(|(paragraph_index, raw)| {
            line_number += 1;
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (indent_level, clean_text) = if trimmed.starts_with(INDENT_MARKER) {
                (1u8, &trimmed[INDENT_MARKER.len()..])
            } else {
                let indent = detect_indent(trimmed);
                (indent, trimmed)
            };
            let clean_text = clean_text.trim();
            if clean_text.is_empty() {
                return None;
            }
            // Adjust link positions: INDENT_MARKER stripping and trim() shift indices
            let raw_links = paragraph_links
                .get(paragraph_index)
                .cloned()
                .unwrap_or_default();
            let leading_offset = raw.chars().count() - raw.trim_start().chars().count();
            let total_offset = if trimmed.starts_with(INDENT_MARKER) {
                let after_marker = &trimmed[INDENT_MARKER.len()..];
                leading_offset
                    + INDENT_MARKER.chars().count()
                    + (after_marker.chars().count() - after_marker.trim_start().chars().count())
            } else {
                leading_offset
            };
            let clean_char_count = clean_text.chars().count();
            let links: Vec<_> = raw_links
                .iter()
                .map(|l| crate::domain::paragraph::TextLink {
                    start: l.start.saturating_sub(total_offset),
                    end: l.end.saturating_sub(total_offset).min(clean_char_count),
                    href: l.href.clone(),
                    title: l.title.clone(),
                })
                .collect();
            Some(Paragraph {
                index: paragraph_index,
                text: clean_text.to_string(),
                kind: infer_paragraph_kind(clean_text),
                indent_level,
                source_line_hint: Some(line_number),
                links,
            })
        })
        .collect::<Vec<_>>();

    let mut blocks: Vec<ChapterBlock> = Vec::new();

    if paragraphs.is_empty() {
        // No paragraphs: render all images
        for (_pos, img) in img_blocks {
            blocks.push(ChapterBlock::Image(img.clone()));
        }
    } else {
        // pos == -1 的图片放最前面
        for (pos, img) in img_blocks.iter().filter(|(pos, _)| *pos == -1) {
            let _ = pos;
            blocks.push(ChapterBlock::Image(img.clone()));
        }

        for para in &paragraphs {
            match para.kind {
                ParagraphKind::Title | ParagraphKind::Subtitle => {
                    blocks.push(ChapterBlock::Heading(para.clone()));
                }
                ParagraphKind::Quote => {
                    blocks.push(ChapterBlock::Quote(para.clone()));
                }
                _ => {
                    blocks.push(ChapterBlock::Paragraph(para.clone()));
                }
            }
            // 在该段落之后插入 pos == para.index 的图片
            for (_pos, img) in img_blocks.iter().filter(|(p, _)| *p == para.index as isize) {
                blocks.push(ChapterBlock::Image(img.clone()));
            }
        }
        // pos >= paragraphs.len() 的图片放最后
        for (_pos, img) in img_blocks
            .iter()
            .filter(|(p, _)| *p >= paragraphs.len() as isize)
        {
            blocks.push(ChapterBlock::Image(img.clone()));
        }
    }

    let (word_count, char_count) = blocks.iter().fold((0usize, 0usize), |(wc, cc), b| {
        let text = match b {
            ChapterBlock::Paragraph(p) | ChapterBlock::Heading(p) | ChapterBlock::Quote(p) => &p.text,
            _ => return (wc, cc),
        };
        (wc + text.split_whitespace().count(), cc + text.chars().count())
    });

    Chapter {
        id: format!("ch-{}", index),
        index,
        title: title.to_string(),
        raw_title: Some(title.to_string()),
        word_count,
        char_count,
        blocks,
        source_href: source_href.map(|s| s.to_string()),
        anchor: None,
        anchors,
        warnings: Vec::new(),
    }
}

fn detect_indent(text: &str) -> u8 {
    let mut indent: u8 = 0;
    for ch in text.chars() {
        match ch {
            '\u{3000}' | ' ' | '\t' => indent += 1,
            _ => break,
        }
        if indent >= 4 {
            break;
        }
    }
    indent.min(4)
}

pub(crate) fn infer_paragraph_kind(text: &str) -> ParagraphKind {
    let char_count = text.chars().count();
    if is_separator_line(text) {
        return ParagraphKind::Separator;
    }
    if text.starts_with('>') || is_quote_wrapped(text) {
        return ParagraphKind::Quote;
    }
    if is_title_line(text, char_count) {
        return ParagraphKind::Title;
    }
    if char_count < 30 && char_count >= 2 && is_subtitle_like(text) {
        return ParagraphKind::Subtitle;
    }
    ParagraphKind::Body
}

fn is_separator_line(text: &str) -> bool {
    let separators = [
        "***",
        "---",
        "＊＊＊",
        "* * *",
        "————",
        "====",
        "~~~~",
        "___",
    ];
    let trimmed = text.trim();
    if separators.contains(&trimmed) {
        return true;
    }
    let chars: Vec<char> = trimmed.chars().collect();
    if chars.len() >= 3 && chars.iter().all(|&c| c == chars[0]) {
        let c = chars[0];
        if c == '*' || c == '-' || c == '—' || c == '＊' || c == '~' || c == '=' || c == '_' {
            return true;
        }
    }
    if chars.len() >= 5 && chars.iter().all(|&c| c == '.') {
        return true;
    }
    let non_space: Vec<char> = chars
        .iter()
        .copied()
        .filter(|c| !c.is_whitespace())
        .collect();
    if non_space.len() >= 3
        && non_space
            .iter()
            .all(|&c| c == '─' || c == '━' || c == '·' || c == '•')
        && non_space.windows(2).all(|w| w[0] == w[1])
    {
        return true;
    }
    false
}

fn is_quote_wrapped(text: &str) -> bool {
    let pairs = [('「', '」'), ('『', '』'), ('"', '"'), ('\'', '\'')];
    let trimmed = text.trim();
    for (open, close) in pairs {
        if trimmed.starts_with(open) && trimmed.ends_with(close) && trimmed.len() > 2 {
            return true;
        }
    }
    trimmed.starts_with('《') && trimmed.ends_with('》') && trimmed.len() > 2
}

fn is_title_line(text: &str, char_count: usize) -> bool {
    if char_count >= 50 {
        return false;
    }
    if text.starts_with('第')
        && (text.contains('章')
            || text.contains('回')
            || text.contains('节')
            || text.contains('卷')
            || text.contains("部分")
            || text.contains('篇'))
    {
        return true;
    }
    let upper = text.to_uppercase();
    if upper.starts_with("CHAPTER ") || upper.starts_with("PART ") {
        return true;
    }
    let special = [
        "序章", "终章", "番外", "楔子", "尾声", "引子", "后记", "前言", "序言",
    ];
    for word in special {
        if text.starts_with(word) {
            return true;
        }
    }
    false
}

fn is_subtitle_like(text: &str) -> bool {
    let trimmed = text.trim();
    if let Some(first) = trimmed.chars().next() {
        if first.is_ascii_digit() {
            let rest = trimmed.trim_start_matches(|c: char| c.is_ascii_digit());
            if rest.starts_with('.') || rest.starts_with('、') {
                return true;
            }
        }
    }
    let cn_numbers = [
        "一", "二", "三", "四", "五", "六", "七", "八", "九", "十", "十一", "十二", "十三", "十四",
        "十五", "十六", "十七", "十八", "十九", "二十", "三十", "四十", "五十", "六十", "七十",
        "八十", "九十",
    ];
    for num in cn_numbers {
        if trimmed.starts_with(num) {
            let rest = &trimmed[num.len()..];
            if rest.starts_with('、') || rest.starts_with('，') || rest.starts_with(',') {
                return true;
            }
        }
    }
    false
}

pub(crate) fn strip_href_fragment(href: &str) -> &str {
    href.split('#').next().unwrap_or(href)
}

pub(crate) fn href_filename(href: &str) -> &str {
    strip_href_fragment(href)
        .rsplit('/')
        .next()
        .unwrap_or(strip_href_fragment(href))
}

pub(crate) fn build_href_index(spine_hrefs: &[String]) -> std::collections::HashMap<String, usize> {
    let mut map = std::collections::HashMap::new();
    for (index, href) in spine_hrefs.iter().enumerate() {
        let key = href_filename(href).to_string();
        map.entry(key).or_insert(index);
    }
    map
}

pub(crate) fn map_toc_chapter_indices(
    items: Vec<TocItem>,
    href_to_index: &std::collections::HashMap<String, usize>,
) -> Vec<TocItem> {
    items
        .into_iter()
        .map(|mut item| {
            if item.chapter_index.is_none() {
                if let Some(ref href) = item.href {
                    let key = href_filename(href).to_string();
                    if let Some(&idx) = href_to_index.get(&key) {
                        item.chapter_index = Some(idx);
                    }
                }
            }
            item.children = map_toc_chapter_indices(item.children, href_to_index);
            item
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn img_block(pos: isize, idx: usize) -> (isize, InlineImageBlock) {
        (
            pos,
            InlineImageBlock {
                index: idx,
                asset_id: format!("img-{}", idx),
                alt_text: None,
                caption: None,
                source_href: None,
            },
        )
    }

    #[test]
    fn images_interleaved_at_correct_positions() {
        let text = "第一段\n\n第二段\n\n第三段";
        let imgs = vec![img_block(0, 0), img_block(2, 1)];
        let ch = build_chapter(0, "test", text, &imgs, None, &[], Vec::new());
        // Expected: Para(0), Img(0), Para(1), Para(2), Img(1)
        let kinds: Vec<&str> = ch
            .blocks
            .iter()
            .map(|b| match b {
                ChapterBlock::Paragraph(_) => "P",
                ChapterBlock::Heading(_) => "H",
                ChapterBlock::Quote(_) => "Q",
                ChapterBlock::Image(_) => "I",
                ChapterBlock::Separator => "S",
            })
            .collect();
        assert_eq!(kinds, vec!["P", "I", "P", "P", "I"]);
    }

    #[test]
    fn pos_neg1_images_before_first_paragraph() {
        let text = "第一段\n\n第二段";
        let imgs = vec![img_block(-1, 0), img_block(-1, 1)];
        let ch = build_chapter(0, "test", text, &imgs, None, &[], Vec::new());
        let kinds: Vec<&str> = ch
            .blocks
            .iter()
            .map(|b| match b {
                ChapterBlock::Paragraph(_) => "P",
                ChapterBlock::Heading(_) => "H",
                ChapterBlock::Quote(_) => "Q",
                ChapterBlock::Image(_) => "I",
                ChapterBlock::Separator => "S",
            })
            .collect();
        assert_eq!(kinds, vec!["I", "I", "P", "P"]);
    }

    #[test]
    fn no_images_produces_only_paragraphs() {
        let text = "第一段\n\n第二段";
        let ch = build_chapter(0, "test", text, &[], None, &[], Vec::new());
        assert_eq!(ch.blocks.len(), 2);
        for block in &ch.blocks {
            assert!(matches!(block, ChapterBlock::Paragraph(_)));
        }
    }
}
