_Catch bugs through automated system testing of a cli application_

[![Crates.io](https://img.shields.io/crates/v/predate.svg)](https://crates.io/crates/predate)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

## What is this?

Many CLI applications take an input file and generate an output file and/or write some value(s) to standard out, with various options set by the user.  `predate` is a simple example of how to test that the application is producing correct output given the arguments specified on the command line. This crate is used to test the output of the crate `grepq` (<https://crates.io/crates/grepq>), and record the execution time to avoid performance regression.  

## Installation

- From _crates.io_ (easiest method, but will not install the `examples` directory)
  - `cargo install predate`

- From _source_ (will install the `examples` directory)
  - Clone the repository and `cd` into the `predate` directory
  - Run `cargo build --release`
  - Relative to the cloned parent directory, the executable will be located in `./target/release`
  - Make sure the executable is in your `PATH` or use the full path to the executable

## How to use

```bash
Usage: predate [OPTIONS] <PATH_TO_TESTS_YAML>

Arguments:
  <PATH_TO_TESTS_YAML>  Path to the tests YAML file

Options:
  -j, --json-out  Write test output to a JSON file
  -c, --control   Set the control
  -h, --help      Print help
```

# So what?

Whilst `predate` is specific to testing of `grepq`, the code should be easily adapted for testing other CLI applications.
