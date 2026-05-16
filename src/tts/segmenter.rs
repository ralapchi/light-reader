use crate::domain::paragraph::Paragraph;

#[derive(Clone, Debug, PartialEq)]
pub struct Segment {
    pub chapter_index: usize,
    pub segment_index: usize,
    pub paragraph_indices: Vec<usize>,
    pub text: String,
    pub char_count: usize,
}

/// Split chapter text by paragraphs.
/// Each segment is one or more paragraphs, respecting max_chars limit.
/// Long paragraphs that exceed max_chars are NOT further split at the
/// sub-paragraph level; instead they become a single-segment overflow
/// (the provider will receive them as-is, though some providers may reject
/// text over their limit).
pub fn segment_chapter(
    chapter_index: usize,
    paragraphs: &[Paragraph],
    max_chars: usize,
) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut segment_index = 0;
    let mut current_text = String::new();
    let mut current_paragraphs: Vec<usize> = Vec::new();

    for para in paragraphs {
        let para_text = para.text.trim();
        if para_text.is_empty() {
            continue;
        }
        // If adding this paragraph would exceed max_chars, flush current segment
        if !current_text.is_empty() && current_text.len() + para_text.len() + 1 > max_chars {
            segments.push(Segment {
                chapter_index,
                segment_index,
                paragraph_indices: current_paragraphs.clone(),
                text: current_text.trim().to_string(),
                char_count: current_text.trim().chars().count(),
            });
            segment_index += 1;
            current_text.clear();
            current_paragraphs.clear();
        }
        if current_text.is_empty() {
            current_text = para_text.to_string();
        } else {
            current_text.push('\n');
            current_text.push_str(para_text);
        }
        current_paragraphs.push(para.index);
    }

    // Flush last segment
    if !current_text.is_empty() {
        segments.push(Segment {
            chapter_index,
            segment_index,
            paragraph_indices: current_paragraphs,
            text: current_text.trim().to_string(),
            char_count: current_text.trim().chars().count(),
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::paragraph::Paragraph;
    use crate::domain::paragraph_kind::ParagraphKind;

    #[test]
    fn empty_paragraphs_yields_no_segments() {
        let segments = segment_chapter(0, &[], 1000);
        assert!(segments.is_empty());
    }

    #[test]
    fn single_paragraph_yields_one_segment() {
        let paras = vec![Paragraph {
            index: 0,
            text: "Hello world".to_string(),
            kind: ParagraphKind::Body,
            indent_level: 0,
            source_line_hint: None,
        }];
        let segments = segment_chapter(0, &paras, 1000);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");
        assert_eq!(segments[0].paragraph_indices, vec![0]);
    }

    #[test]
    fn multiple_paragraphs_combined_within_limit() {
        let paras = vec![
            Paragraph {
                index: 0,
                text: "Short".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
            Paragraph {
                index: 1,
                text: "Para".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
        ];
        let segments = segment_chapter(0, &paras, 100);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].paragraph_indices, vec![0, 1]);
    }

    #[test]
    fn long_paragraph_triggers_split() {
        let paras = vec![
            Paragraph {
                index: 0,
                text: "A".repeat(30),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
            Paragraph {
                index: 1,
                text: "B".repeat(30),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
        ];
        let segments = segment_chapter(0, &paras, 40);
        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].paragraph_indices, vec![0]);
        assert_eq!(segments[1].paragraph_indices, vec![1]);
    }

    #[test]
    fn empty_paragraphs_are_skipped() {
        let paras = vec![
            Paragraph {
                index: 0,
                text: "First".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
            Paragraph {
                index: 1,
                text: "".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
            Paragraph {
                index: 2,
                text: "   ".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
            Paragraph {
                index: 3,
                text: "Last".to_string(),
                kind: ParagraphKind::Body,
                indent_level: 0,
                source_line_hint: None,
            },
        ];
        let segments = segment_chapter(0, &paras, 100);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "First\nLast");
    }

    #[test]
    fn char_count_is_accurate() {
        let paras = vec![Paragraph {
            index: 0,
            text: "你好世界".to_string(),
            kind: ParagraphKind::Body,
            indent_level: 0,
            source_line_hint: None,
        }];
        let segments = segment_chapter(0, &paras, 1000);
        assert_eq!(segments[0].char_count, 4);
    }
}
