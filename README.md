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
    led-bargraph [options] clear
    led-bargraph [options] set <value> <range>
    led-bargraph [options] show

Commands:
    clear   Clear the display.
    set     Display the value against the range.
    show    Show on-screen the current bargraph display.

Arguments:
    value   The value to display.
    range   The range of the bar graph to display.

Options:
    --no-init               Do not initialize the device.
    --trace                 Enable verbose debug logging.
    -d, --debug             Enable debug logging.
    -v, --verbose           Enable verbose logging.
    -s, --show              Show on-screen the current bargraph display.
    --i2c-mock              Mock the I2C interface, useful when no device is available.
    --i2c-address=<N>       Address of the I2C device, in decimal [default: 112].
    --i2c-path=<path>       Path to the I2C device [default: /dev/i2c-1].
    -h, --help              Print this help.
```

## Supported Platforms

* Linux
    * 32 & 64bit
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

