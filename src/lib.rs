#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    variant_size_differences
)]

use std::io::{BufRead, Cursor, Read};

use quick_xml::{Reader, Writer, events::Event};
use regex::bytes::Regex;

/// Filter and print XMLs.
pub fn filter_xmls(mut input: impl BufRead, xpath: Option<&str>) {
    let log_entry_date_regex =
        Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").expect("valid regex");
    let log_entry_non_ws_regex = Regex::new(r"^\S").expect("valid regex");

    let mut buffer = [0u8; 1024];

    let mut first_entry = Vec::new();

    while let Ok(count) = input.read(&mut buffer) {
        if count == 0 {
            break;
        }
        first_entry.extend_from_slice(&buffer[..count]);
        if first_entry.len() > 19 {
            break;
        }
    }

    let log_entry_regex = if log_entry_date_regex.is_match(&first_entry) {
        Some(Regex::new(r"\n\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}").expect("valid regex"))
    } else if log_entry_non_ws_regex.is_match(&first_entry) {
        Some(Regex::new(r"\n\S").expect("valid regex"))
    } else {
        None
    };

    let mut head = first_entry.as_slice();

    loop {
        while let Some(pos) = head.iter().position(|&n| n == b'<') {
            head = &head[pos..];

            let cursor = Cursor::new(head);

            let xml_candidate = cursor.chain(input);

            let mut reader = Reader::from_reader(xml_candidate);

            let mut buf = Vec::new();

            let events = match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref b)) => {
                    let (start, end) = (b.clone().into_owned(), b.to_end().into_owned());

                    let end = end.name();

                    let mut depth = 0;
                    let mut events = vec![Event::Start(start)];

                    loop {
                        let evt = reader.read_event_into(&mut buf);

                        if let Ok(e) = &evt {
                            events.push(e.clone().into_owned());
                        }

                        match evt {
                            Ok(Event::Start(ref e)) if e.name() == end => depth += 1,
                            Ok(Event::End(ref e)) if e.name() == end => {
                                if depth == 0 {
                                    break Ok(events);
                                }
                                depth -= 1;
                            }
                            Ok(Event::Text(e)) => {
                                if let Some(log_entry_regex) = log_entry_regex.as_ref() {
                                    if log_entry_regex.is_match(&e) {
                                        break Err(());
                                    }
                                }
                            }
                            Ok(Event::Eof) | Err(_) => break Err(()),
                            _ => (),
                        }
                    }
                }
                Ok(e @ Event::Empty(_)) => Ok(vec![e.clone().into_owned()]),
                _ => Err(()),
            };

            if let Ok(events) = events {
                let mut writer = Writer::new(Cursor::new(Vec::new()));
                for event in events {
                    writer.write_event(event).expect("write event");
                }
                let buf = writer.into_inner().into_inner();
                let xml = String::from_utf8_lossy(&buf);
                if let Some(xpath) = xpath {
                    use amxml::dom::*;
                    let doc = new_document(&xml).expect("well formed XML");
                    let root = doc.root_element();
                    let result = root.eval_xpath(xpath).expect("XPath expression");

                    for item in (0..result.len()).filter_map(|i| result.get_item(i).as_nodeptr()) {
                        println!("{}", item.to_string());
                    }
                } else {
                    println!("{xml}");
                }
            }

            let (cursor, remaining_input) = reader.into_inner().into_inner();

            input = remaining_input;

            if (cursor.position() as usize) < head.len() {
                head = &head[cursor.position() as usize..];
            } else {
                break;
            }
        }

        if let Ok(count) = input.read(&mut buffer) {
            if count == 0 {
                break;
            }
            head = &buffer[..count];
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
