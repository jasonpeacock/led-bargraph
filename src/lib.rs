//! # Bargraph
//!
//! A library for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).
//!
//! The HT16K33 has 16 rows by 8 commons for controlling 128 LEDs (#0-127). This is represented internally
//! by an array of size 16 of type u8, where each lower bit of the u8 value denotes a common:
//!
//! __   0   2   4   8  16  32  64 128
//! 00   0   1   2   3   4   5   6   7
//! 01   8   9  10  11  12  13  14  15
//! ...
//! 14 112 113 114 115 116 117 118 119
//! 15 120 121 122 123 124 125 126 127
//!
//! The LED address (N) can be converted to a buffer location by modulo `8` to calculate the row,
//! then left-shifted by the remainder to calculate the common:
//!
//! LED #11 -> row 1, common 8 (e.g. 1 << 3)
//!
extern crate ansi_term;
extern crate embedded_hal as hal;
extern crate ht16k33;
extern crate num_integer;

#[macro_use]
extern crate slog;
extern crate slog_stdlog;

use ansi_term::Colour::{Fixed, Green, Red, White, Yellow};

use hal::blocking::i2c::{Write, WriteRead};

use ht16k33::HT16K33;

use num_integer::Integer;

use slog::Drain;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LedColor {
    /// Turn off both the Red & Green LEDs.
    Off,
    /// Turn on only the Green LED.
    Green,
    /// Turn on only the Red LED.
    Red,
    /// Turn on both the Red  & Green LEDs.
    Yellow,
}

const BARGRAPH_DISPLAY_CHAR: &str = "\u{258A}";
const BARGRAPH_RESOLUTION: u8 = 24;

pub struct Bargraph<I2C> {
    device: HT16K33<I2C>,
    show: bool,
    logger: slog::Logger,
}

impl<I2C, E> Bargraph<I2C>
where
    I2C: Write<Error = E> + WriteRead<Error = E>,
{
    /// Create a Bargraph for display.
    ///
    /// # Arguments
    ///
    /// * `device` - A connected `HTK1633` device that drives the display.
    /// * `logger` - A logging instance.
    /// * `show` - Show the bargraph state on-screen.
    ///
    /// # Notes
    ///
    /// `logger = None` will log to the `slog-stdlog` drain. This makes the
    /// library effectively work the same as if it was just using `log` instead
    /// of `slog`.
    ///
    /// The `Into` [trick](http://xion.io/post/code/rust-optional-args.html) allows
    /// passing `Logger` directly, without the `Some` part.
    pub fn new<L>(i2c: I2C, i2c_address: u8, show: bool, logger: L) -> Self
    where
        L: Into<Option<slog::Logger>>,
    {
        let logger = logger
            .into()
            .unwrap_or_else(|| slog::Logger::root(slog_stdlog::StdLog.fuse(), o!()));

        trace!(logger, "Constructing Bargraph");

        let ht16k33_logger = logger.new(o!("mod" => "HT16K33"));
        let ht16k33 = HT16K33::new(i2c, i2c_address, ht16k33_logger);

        Bargraph {
            device: ht16k33,
            show,
            logger,
        }
    }

    /// Initialize the Bargraph display & the connected `HT16K33` device.
    pub fn initialize(&mut self) -> Result<(), E> {
        trace!(self.logger, "initialize");

        // Reset the display.
        self.device.initialize()?;

        Ok(())
    }

    /// Clear the Bargraph display.
    pub fn clear(&mut self) -> Result<(), E> {
        trace!(self.logger, "clear");

        self.device.clear_display_buffer();
        self.device.write_display_buffer()
    }

    /// Update the Bargraph display, showing `range` total bars with all bars
    /// from `0` to `value` filled.
    ///
    /// If `value` is greater than `range`, then all bars are filled and will blink;
    /// automatic re-scaling of the range does *not* happen because:
    ///
    /// * The bargraph can only scale to a maximum resolution.
    /// * Users are already familiar with viewing the current range, and dynamically
    ///   changing the range makes it hard for users to see what's happening at a glance.
    ///
    /// **Idea** Support a "low fuel" mode, where the display flashes when below some threshold.
    ///
    /// # Arguments
    ///
    /// * `value` - How many values to fill, starting from `0`.
    /// * `range` - Total number of values to display.
    // TODO accept more user-friendly input values?
    pub fn update(&mut self, value: u8, range: u8) -> Result<(), E> {
        trace!(self.logger, "update");

        // Reset the display in preparation for the update.
        self.device.clear_display_buffer();

        let mut blink = false;
        let mut clamped_value = value;

        if value > range {
            warn!(self.logger, "Value is greater than range, setting display to blink";
                  "value" => value, "range" => range);
            clamped_value = range;
            blink = true;
        }

        for current_value in 1..=range {
            let fill = current_value <= clamped_value;
            self.update_value_fill(current_value - 1, range, fill);
        }

        self.device.write_display_buffer()?;

        self.set_blink(blink)?;

        if self.show {
            self.show()?;
        }

        Ok(())
    }

    /// Show on-screen the current bargraph display.
    pub fn show(&mut self) -> Result<(), E> {
        trace!(self.logger, "show");

        // Read current values from the device.
        self.device.read_display_buffer()?;

        // Convert values for display.

        // Display the values.
        // Unicode box-drawing characters: https://en.wikipedia.org/wiki/Box-drawing_character
        println!(
            "{corner_top_left}{line}{corner_top_right}",
            corner_top_left = White.paint("\u{2554}"),
            line = White.paint(std::iter::repeat("\u{2550}").take(9).collect::<String>()),
            corner_top_right = White.paint("\u{2557}")
        );

        println!(
            "{side}{yellow}{yellow}{red}{black}{black}{green}{black}{black}{green}{side}",
            side = White.paint("\u{2551}"),
            green = Green.paint(BARGRAPH_DISPLAY_CHAR),
            yellow = Yellow.paint(BARGRAPH_DISPLAY_CHAR),
            red = Red.paint(BARGRAPH_DISPLAY_CHAR),
            black = Fixed(238).paint(BARGRAPH_DISPLAY_CHAR)
        );

        println!(
            "{corner_bottom_left}{line}{corner_bottom_right}",
            corner_bottom_left = White.paint("\u{255A}"),
            line = White.paint(std::iter::repeat("\u{2550}").take(9).collect::<String>()),
            corner_bottom_right = White.paint("\u{255D}")
        );

        Ok(())
    }

    /// Enable/Disable continuous blinking of the Bargraph display.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enabled blinking or not.
    pub fn set_blink(&mut self, enabled: bool) -> Result<(), E> {
        trace!(self.logger, "set_blink"; "enabled" => enabled);

        if enabled {
            self.device
                .set_display(ht16k33::Display::On, ht16k33::Blink::TwoHz)
        } else {
            self.device
                .set_display(ht16k33::Display::On, ht16k33::Blink::Off)
        }
    }

    // Enable/disable the fill for a `value` on the Bargraph display.
    //
    // # Arguments
    //
    // * `value` - Which value to fill.
    // * `range` - The total range of the display (for calculating the value size).
    // * `fill` - Whether to fill (true) the value or only display its header.
    //
    // # Notes
    //
    // Value `0` is at the bottom of the display (lowest value).
    fn update_value_fill(&mut self, value: u8, range: u8, fill: bool) {
        trace!(self.logger, "update_value_fill"; "value" => value, "range" => range, "fill" => fill);

        // Calculate the size of the value.
        let value_size = BARGRAPH_RESOLUTION / range;

        let start_value = value * value_size;
        let end_value = start_value + value_size - 1;

        // Fill in the value.
        for current_value in start_value..end_value {
            if fill {
                // Make the fill yellow if it's ON.
                let _ = self.update_value_(current_value, LedColor::Yellow);
            } else {
                // Leave it empty if above an ON value.
                let _ = self.update_value_(current_value, LedColor::Off);
            }
        }

        // Color the value header (end of value).
        if fill {
            let _ = self.update_value_(end_value, LedColor::Red);
        } else {
            let _ = self.update_value_(end_value, LedColor::Green);
        }
    }

    // Set value_to desired color. Value should be a value of 0 to 23, and color should be
    // OFF, GREEN, RED, or YELLOW.
    //
    // The buffer must be written using [write_display_buffer()](struct.HT16K33.html#method.write_display_buffer)
    // for the change to be displayed.
    //
    // # Arguments
    //
    // * `value_- A value from `0` to `23`.
    // * `color` - A valid color value.
    fn update_value_(&mut self, value: u8, color: LedColor) -> Result<(), E> {
        // TODO use Option to return only errors for these void functions
        // TODO Validate `value` parameter.
        trace!(self.logger, "update_value"; "value" => value, "color" => format!("{:?}", color));

        let (count, remainder) = value.div_mod_floor(&12);
        let (row, mut common) = remainder.div_mod_floor(&4);
        let red_row = row * 2;
        let green_row = red_row + 1;
        common += count * 4;

        if color == LedColor::Green || color == LedColor::Yellow {
            let _ = self
                .device
                .update_display_buffer(ht16k33::LedLocation::new(green_row, common).unwrap(), true);
        } else {
            let _ = self.device.update_display_buffer(
                ht16k33::LedLocation::new(green_row, common).unwrap(),
                false,
            );
        }

        if color == LedColor::Red || color == LedColor::Yellow {
            let _ = self
                .device
                .update_display_buffer(ht16k33::LedLocation::new(red_row, common).unwrap(), true);
        } else {
            let _ = self
                .device
                .update_display_buffer(ht16k33::LedLocation::new(red_row, common).unwrap(), false);
        }

        Ok(())
    }
}
