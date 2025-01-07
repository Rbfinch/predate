<img src="src/predate-icon.svg" width="128" />

_Catch bugs and performance regressions through automated system testing_

[![Crates.io](https://img.shields.io/crates/v/predate.svg)](https://crates.io/crates/predate)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## What is this?

Many CLI applications take an input file and generate an output file and/or write some value(s) to standard out, with various options set by the user. `predate` is a simple example of how to test that the application is producing correct output given the arguments specified on the command line. Whilst there are libraries that can be used to test CLI applications, `predate` is a simple example of how to do this without the need for additional dependencies, and where the commands and expected output are conveniently specified in a YAML file.

`predate` is used to test the output of `grepq` (<https://crates.io/crates/grepq>), and record the execution time of `grepq` commands to avoid performance regression. Whilst `predate` is specific to testing of `grepq`, the code should be easily adapted for testing other CLI applications.

## Requirements

- `predate` has been tested on Linux and macOS. It might work on Windows, but it has not been tested
- Ensure that Rust is installed on your system (<https://www.rust-lang.org/tools/install>)
- If the build fails, make sure you have the latest version of the Rust compiler by running `rustup update`

## Installation

- From _crates.io_
  - `cargo install predate`

- From _source_
  - Clone the repository and `cd` into the `predate` directory
  - Run `cargo build --release`
  - Relative to the cloned parent directory, the executable will be located in `./target/release`
  - Make sure the executable is in your `PATH` or use the full path to the executable

## How to use

>[!NOTE]
`predate` is specifically used to test `grepq`

```bash
cd /path/to/grepq/examples
predate [OPTIONS] <PATH_TO_TESTS_YAML>
```

```bash
Usage: predate [OPTIONS] <PATH_TO_TESTS_YAML>

Arguments:
  <PATH_TO_TESTS_YAML>  Path to the tests YAML file

Options:
  -j, --json-out  Write test output to a JSON file
  -c, --control   Set the control
  -h, --help      Print help
```

## Update changes

see [CHANGELOG](https://github.com/Rbfinch/predate/blob/main/CHANGELOG.md)

## License

MIT
