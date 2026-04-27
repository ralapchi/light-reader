use quick_xml::events::Event;
use quick_xml::Reader;

fn main() {
    let html = "<h2 class=\"title\">第1章 内容简介</h2>";
    let title = extract_title(html);
    println!("Extracted title: '{}'", title);
}

fn extract_title(html: &str) -> String {
    let mut reader = Reader::from_str(html);
    reader.trim_text(true);

    let mut title = String::new();
    let mut depth = 0;

    let mut buffer = Vec::new();

    loop {
        match reader.read_event_into(&mut buffer) {
            Ok(Event::Start(ref e)) => {
                let tag_name = e.name().as_ref();
                if tag_name == b"h1" || tag_name == b"h2" || tag_name == b"h3" {
                    depth += 1;
                }
            }
            Ok(Event::End(ref e)) => {
                let tag_name = e.name().as_ref();
                if tag_name == b"h1" || tag_name == b"h2" || tag_name == b"h3" {
                    if depth > 0 {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                }
            }
            Ok(Event::Text(ref e)) => {
                if depth > 0 {
                    if let Ok(text) = e.unescape() {
                        title.push_str(&text);
                    }
                }
            }
            Ok(Event::Eof) => break,
            _ => {}
        }
        buffer.clear();
    }

    title.trim().to_string()
}