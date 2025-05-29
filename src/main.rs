use std::{fs::File, io::BufReader, path::PathBuf};

use clap::Parser;

#[derive(Parser)]
#[clap(author, version)]
struct Cli {
    /// XPath transformation to apply to XML readed.
    #[arg(long, short, env)]
    xpath: Option<String>,
    /// File input.
    #[arg(long, short)]
    input: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let xpath = cli.xpath.as_deref();
    if let Some(input) = cli.input {
        let file = BufReader::new(File::open(input).expect("existing file"));
        xml_log_scan::filter_xmls(file, xpath);
    } else {
        let stdin = std::io::stdin().lock();
        xml_log_scan::filter_xmls(stdin, xpath);
    }
}
