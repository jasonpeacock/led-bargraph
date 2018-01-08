# LED Bargraph

A Rust library & application for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).

# Features

## Configurable commandline application

## Direct-write library

# Requirements

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

## Completed

* <del>Support OSX compilation.</del> Fixed in Release 0.2

# Releases

## 0.2

- Refactored to use custom error types.
- Support OSX compilation through traits/generics to inject the I2CDevice.

## 0.1

- It works!
