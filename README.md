# xml-log-scan

## Legal

Dual-licensed under `MIT` or the [UNLICENSE](http://unlicense.org/).

## Features

- Extracts XMLs from log files or standard input
- Outputs results in a structured and human-readable format
- Supports [XPath 3.1](https://www.w3.org/TR/xpath-31/) transformation
- Can handle large log files efficiently

## Support:

You can contribute to the ongoing development and maintenance of **xml-log-scan** application in various ways:

### Sponsorship

Your support, no matter how big or small, helps sustain the project and ensures its continued improvement. Reach out to explore sponsorship opportunities.

### Feedback

Whether you are a developer, user, or enthusiast, your feedback is invaluable. Share your thoughts, suggestions, and ideas to help shape the future of the library.

### Contribution

If you're passionate about open-source and have skills to share, consider contributing to the project. Every contribution counts!

Thank you for being part of **xml-log-scan** community. Together, we are making authentication processes more accessible, reliable, and efficient for everyone.


## Install

Install `xml-log-scan` using Cargo with the command `cargo install xml-log-scan`. This command downloads and installs the package from `crates.io`. Once installed, you can use the tool from the command line to analyze logs containing XMLs.

```sh
cargo install xml-log-scan
```

## Usage

**xml-log-scan** is a command-line utility, usage info can be access with --help argument:

```bash
$ xml-log-scan --help
Usage: xml-log-scan [OPTIONS]

Options:
  -x, --xpath <XPATH>  XPath transformation to apply to XML readed [env: XPATH=]
  -i, --input <INPUT>  File input
  -h, --help           Print help
  -V, --version        Print version

```

Sample log:

```bash
123414214231234 <hello>
  <world/>
</hello>
sdfafsdasfd

sdfafsdasfdsdf
zed <simple qqq="aaa"/>
sdfafsdasfd
qqq <another></another>
```

### Analyze log for XMLs

```bash
$ xml-log-scan --input fixtures/example.log
<hello>
  <world/>
</hello>
<simple qqq="aaa"/>
<another></another>

```

### Pipe log to analyze for XMLs

```bash
$ cat fixtures/example.log | xml-log-scan
<hello>
  <world/>
</hello>
<simple qqq="aaa"/>
<another></another>

```

### Apply XPath to XMLs found

```bash
$ xml-log-scan --input fixtures/example.log --xpath //world
<world/>

```

## References

- [XPath 3.1](https://www.w3.org/TR/xpath-31/)