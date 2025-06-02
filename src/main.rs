use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;
use regex::bytes::Regex;

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    /// XPath transformation to apply to XML readed.
    #[arg(long, short, env)]
    xpath: Option<String>,
    /// File with XPath transformation to apply to XML readed.
    #[arg(long, env, conflicts_with = "xpath")]
    xpath_file: Option<PathBuf>,
    /// Regular expression to identify log entry start, should start with `\n`.
    #[arg(long, short, env)]
    regex: Option<Regex>,
    /// File with regular expression to identify log entry start, should start with `\n`.
    #[arg(long, env, conflicts_with = "regex")]
    regex_file: Option<Regex>,
    /// File input.
    #[arg(long, short)]
    input: Option<PathBuf>,
}

fn main() {
    let Cli {
        xpath,
        xpath_file,
        input,
        ..
    } = Cli::parse();

    let xpath = xpath_file
        .and_then(|file| std::fs::read_to_string(file).ok())
        .or(xpath);
    if let Some(input) = input {
        let file = BufReader::new(File::open(input).expect("existing file"));
        xml_log_scan::filter_xmls(file, xpath.as_deref());
    } else {
        let stdin = std::io::stdin().lock();
        xml_log_scan::filter_xmls(stdin, xpath.as_deref());
    }
}
