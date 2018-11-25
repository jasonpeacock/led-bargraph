# LED Bargraph

[![Version info](https://img.shields.io/crates/v/led-bargraph.svg)](https://crates.io/crates/led-bargraph)
[![Documentation](https://docs.rs/led_bargraph/badge.svg)](https://docs.rs/led_bargraph)
[![Build Status](https://travis-ci.org/jasonpeacock/led-bargraph.svg?branch=master)](https://travis-ci.org/jasonpeacock/led-bargraph)
[![Average time to resolve an issue](http://isitmaintained.com/badge/resolution/jasonpeacock/led-bargraph.svg)](http://isitmaintained.com/project/jasonpeacock/led-bargraph "Average time to resolve an issue")
[![Percentage of issues still open](http://isitmaintained.com/badge/open/jasonpeacock/led-bargraph.svg)](http://isitmaintained.com/project/jasonpeacock/led-bargraph "Percentage of issues still open")

A Rust library & application for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).

# User Guide

```text
LED Bargraph.

Usage:
    led-bargraph clear
    led-bargraph set <value> <range>
    led-bargraph (-h | --help)

Commands:
    clear   Clear the display.
    set     Display the value against the range.

Arguments:
    value   The value to display.
    range   The range of the bar graph to display.

Options:
    -h --help               Show this screen.
    --i2c-path=<path>       Path to the I2C device [default: /dev/i2c-1].
    --i2c-address=<N>       Address of the I2C device, in decimal [default: 112].
    --bargraph-size=<N>     Size of the bargraph [default: 24].
```

# Installation

### With cargo (Linux, OSX, Windows)

`led-bargraph` can be installed via [cargo](https://doc.rust-lang.org/cargo/):

```
cargo install led-bargraph
```

Make sure that you use Rust 1.24 or higher.

### From binaries (Linux, OSX, Windows)

Download the corresponding archive from the [Releases](https://github.com/jasonpeacock/led-bargraph/releases) page.

# Todo

* Support configuring defaults & behavior via TOML.
* Built-in daemonizing (update itself using a given command, forever).
* Reverse bargraph direction on LEDs.
* Unit tests.
* Documentation.
* Useful logging in libraries.
* Review [API Guidelines Checklist](https://rust-lang-nursery.github.io/api-guidelines/checklist.html)
* Refactor `HT16K33` to use an interface.
* `HT16K33` should initialize itself before being passed to `Bargraph`, and `Bargraph` just verifies that it's usable.

# In Progress

* Add `--show` option for virtual display on the command-line.

## Supported Rust Versions

See the top of the [Travis configuration file](.travis.yml) for the oldest, and other, supported Rust versions.

## Supported Platforms

* Linux
    * 32 & 64bit
    * gnu & musl
* OSX
    * 64bit

# License

Licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

