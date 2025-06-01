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

use std::{
    io::{BufRead, Cursor, Read},
    ops::Range,
};

use quick_xml::{Reader, Writer, events::Event};
use regex::bytes::Regex;
use thiserror::Error;

/// Filter and print XMLs.
pub fn filter_xmls(input: impl BufRead, xpath: Option<&str>) {
    let xml_extractor = XmlExtractor::<_, 1024>::new(input, None).into_iter();
    if let Some(xpath) = xpath {
        for entry in xml_extractor.map(|xml| {
            xml.map(|xml| {
                use amxml::dom::*;
                let doc = new_document(&xml).expect("well formed XML");
                let root = doc.root_element();
                let result = root.eval_xpath(xpath).expect("XPath expression");

                (0..result.len())
                    .filter_map(|i| result.get_item(i).as_nodeptr())
                    .map(|item| item.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }) {
            println!("{}", entry.expect("next xml"));
        }
    } else {
        for entry in xml_extractor {
            println!("{}", entry.expect("next xml"));
        }
    }
}

/// XmlExtractor from input stream.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct XmlExtractor<I: BufRead, const B: usize = 1024> {
    input: Option<I>,
    buffer: [u8; B],
    log_entry_regex: Option<Regex>,
}

impl<I: BufRead, const B: usize> XmlExtractor<I, B> {
    /// Create a new XmlExtractor from input stream.
    pub fn new(input: I, log_entry_regex: Option<Regex>) -> XmlExtractor<I, B> {
        XmlExtractor {
            input: Some(input),
            buffer: [0; B],
            log_entry_regex,
        }
    }
}

impl<I: BufRead, const B: usize> IntoIterator for XmlExtractor<I, B> {
    type Item = Result<String, XmlExtractorError>;
    type IntoIter = XmlExtractorIter<I, B>;

    fn into_iter(self) -> Self::IntoIter {
        const LOG_ENTRY_DATE_MIN_LEN: usize = 19;
        let Self {
            mut input,
            mut buffer,
            log_entry_regex,
        } = self;
        let Some(mut input) = input.take() else {
            panic!("Input stream is empty");
        };
        let mut total_count = 0;
        let mut eof = false;
        while let Ok(count) = input.read(&mut buffer[total_count..]) {
            if count == 0 {
                eof = true;
                break;
            }
            total_count += count;
            if total_count > LOG_ENTRY_DATE_MIN_LEN {
                break;
            }
        }
        let log_entry_regex = log_entry_regex.or_else(|| {
            const LOG_ENTRY_DATE_REGEX: &str = r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}";
            const LOG_NEW_ENTRY_DATE_REGEX: &str = r"\n\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}";
            const LOG_ENTRY_NON_WS_REGEX: &str = r"^\S";
            const LOG_NEW_ENTRY_NON_WS_REGEX: &str = r"\n\S";

            if total_count > LOG_ENTRY_DATE_MIN_LEN
                && Regex::new(LOG_ENTRY_DATE_REGEX)
                    .expect("valid regex")
                    .is_match(&buffer[..total_count])
            {
                Some(Regex::new(LOG_NEW_ENTRY_DATE_REGEX).expect("valid regex"))
            } else if Regex::new(LOG_ENTRY_NON_WS_REGEX)
                .expect("valid regex")
                .is_match(&buffer[..total_count])
            {
                Some(Regex::new(LOG_NEW_ENTRY_NON_WS_REGEX).expect("valid regex"))
            } else {
                None
            }
        });

        Self::IntoIter {
            input: if eof { None } else { Some(input) },
            buffer,
            total_pos: 0,
            head_range: 0..total_count,
            log_entry_regex,
        }
    }
}

/// Error type for XML extraction.
#[derive(Debug, Error)]
pub enum XmlExtractorError {
    /// I/O error
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    /// Regex error
    #[error("Regex error")]
    Regex(#[from] regex::Error),
}

/// Iterator for extracting XML elements from a BufRead input.
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct XmlExtractorIter<I: BufRead, const B: usize> {
    input: Option<I>,
    buffer: [u8; B],
    total_pos: usize,
    log_entry_regex: Option<Regex>,
    head_range: Range<usize>,
}

impl<I: BufRead, const B: usize> Iterator for XmlExtractorIter<I, B> {
    type Item = Result<String, XmlExtractorError>;

    fn next(&mut self) -> Option<Self::Item> {
        let &mut Self {
            ref mut log_entry_regex,
            ref mut head_range,
            ref mut total_pos,
            ref mut input,
            ref mut buffer,
        } = self;

        let mut stream = input.take()?;

        let mut head = &buffer[head_range.clone()];

        let mut result = None;

        loop {
            while let Some(pos) = head.iter().position(|&n| n == b'<') {
                head = &head[pos..];
                *total_pos += pos;
                let cursor = Cursor::new(head);

                let xml_candidate = cursor.chain(stream);

                let (xml_candidate, events) = read_xmls(log_entry_regex, xml_candidate);

                if let Ok(events) = events {
                    let mut writer = Writer::new(Cursor::new(Vec::new()));
                    for event in events {
                        if let Err(err) = writer.write_event(event) {
                            return Some(Err(err.into()));
                        }
                    }
                    let buf = writer.into_inner().into_inner();
                    let xml = String::from_utf8_lossy(&buf);
                    result = Some(Ok(format!("{xml}")));
                }

                let (cursor, remaining_input) = xml_candidate.into_inner();

                stream = remaining_input;

                let cursor_position = cursor.position() as usize;

                if cursor_position < head.len() {
                    head = &head[cursor_position..];
                    if result.is_some() {
                        *total_pos += cursor_position;
                        if *total_pos > head_range.end {
                            *total_pos = head_range.end;
                            // could be we should skip (head_range.end - *total_pos) bytes in stream.read
                        }
                        *head_range = *total_pos..head_range.end;
                        *input = Some(stream);
                        return result;
                    }
                } else {
                    break;
                }
            }
            if let Ok(count) = stream.read(buffer.as_mut()) {
                if count == 0 {
                    break;
                }
                if result.is_none() {
                    head = &buffer[..count];
                    *total_pos = 0;
                } else {
                    *head_range = 0..count;
                    *total_pos = 0;
                    *input = Some(stream);
                    break;
                }
            } else {
                *input = Some(stream);
                break;
            }
        }
        result
    }
}

fn read_xmls<I: BufRead>(
    log_entry_regex: &mut Option<Regex>,
    xml_candidate: I,
) -> (I, Result<Vec<Event<'static>>, ()>) {
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
    (reader.into_inner(), events)
}

#[cfg(test)]
mod tests {
    use std::io::BufReader;

    use crate::XmlExtractor;

    #[test]
    fn test_xml_extractor_iter() {
        let xml = include_bytes!("../fixtures/example.log").as_slice();
        let extractor = XmlExtractor::<_, 1024>::new(BufReader::new(xml), None);
        let mut iter = extractor.into_iter();

        assert_eq!(
            iter.next().transpose().unwrap(),
            Some("<hello>\n  <world/>\n</hello>".to_string())
        );
        assert_eq!(
            iter.next().transpose().unwrap(),
            Some(r#"<simple qqq="aaa"/>"#.to_string())
        );
        assert_eq!(
            iter.next().transpose().unwrap(),
            Some("<another></another>".to_string())
        );
        assert_eq!(iter.next().transpose().unwrap(), None);
    }

    #[test]
    fn test_xml_extractor_iter2() {
        let xml = b"qqq <hello/> <world/>\nqqq <next><child/></next>".as_slice();
        let extractor = XmlExtractor::<_, 1024>::new(BufReader::new(xml), None);
        let mut iter = extractor.into_iter().flatten();

        assert_eq!(iter.next(), Some("<hello/>".to_string()));
        assert_eq!(iter.next(), Some("<world/>".to_string()));
        assert_eq!(iter.next(), Some("<next><child/></next>".to_string()));
        assert_eq!(iter.next(), None);
    }
}
