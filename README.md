# LED Bargraph

A Rust library & application for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).

You can integrate `led-bargraph` into your project through the [releases on crates.io](https://crates.io/crates/ht16k33):

```toml
# Cargo.toml
[dependencies]
led-bargraph = "0.1.0"
```

# User Guide

```
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

## Completed

* <del>Refactor `ht16k33` library into a separate crate.</del>
* <del>Support OSX compilation.</del>

# License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

