# LED Bargraph

A Rust library & application for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).

# Features

## Configurable cmdline application

## Direct-write library

# Requirements

# User Guide

# Todo

* Configure defaults & behavior via YAML.
* Built-in daemonizing (update itself using a given command, forever).
* Reverse bargraph direction on LEDs.
* Unit tests.
* Documentation.
* Support OSX compilation.

# Releases

## 0.2

- Refactored to use custom error types.
- Support OSX compilation through traits/generics to inject the I2CDevice.

## 0.1

- It works!
