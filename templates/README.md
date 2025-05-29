# xml-log-scan

## Legal

Dual-licensed under `MIT` or the [UNLICENSE](http://unlicense.org/).

## Features

- Parses and analyzes log files containing XMLs
- Outputs results in a structured and human-readable format
- Can handle large log files efficiently
- Maybe used to pipe output of other command

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
{{ exec "xml-log-scan --help" }}
```

Sample log:

```bash
{{ read_to_str "fixtures/example.log" }}
```

### Analyze log for XMLs

```bash
$ xml-log-scan --input fixtures/example.log
{{ exec "xml-log-scan --input fixtures/example.log" }}
```

### Pipe log to analyze for XMLs

```bash
$ cat fixtures/example.log | xml-log-scan
{{ exec "xml-log-scan --input fixtures/example.log" }}
```

### Apply XPath to found XMLs

```bash
$ xml-log-scan --input fixtures/example.log --xpath //world
{{ exec "xml-log-scan --input fixtures/example.log --xpath //world" }}
```
