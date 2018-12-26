//! # Bargraph
//!
//! A library for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).
#![deny(missing_docs)]
extern crate ansi_term;
extern crate embedded_hal as hal;
extern crate ht16k33;
extern crate num_integer;

#[macro_use]
extern crate slog;
extern crate slog_stdlog;

use ansi_term::Colour::{Fixed, Green, Red, White, Yellow};
use ansi_term::Style;

use hal::blocking::i2c::{Write, WriteRead};

use ht16k33::{Display, HT16K33};

use num_integer::Integer;

use slog::Drain;

#[derive(Clone, Copy, Debug, PartialEq)]
/// LED colors.
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

/// The bargraph state.
pub struct Bargraph<I2C> {
    device: HT16K33<I2C>,
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
    /// * `device` - A connected `HT16K33` device that drives the display.
    /// * `logger` - A logging instance.
    ///
    /// # Notes
    ///
    /// `logger = None` will log to the `slog-stdlog` drain. This makes the
    /// library effectively work the same as if it was just using `log` instead
    /// of `slog`.
    ///
    /// # Examples
    ///
    /// ```
    /// // NOTE: `None is used for the Logger in these examples for convenience,
    /// // in practice using an actual logger in preferred.
    ///
    /// extern crate ht16k33;
    /// extern crate led_bargraph;
    ///
    /// use ht16k33::i2c_mock::I2cMock;
    /// use led_bargraph::Bargraph;
    /// # fn main() {
    ///
    /// // Create an I2C device.
    /// let mut i2c = I2cMock::new(None);
    ///
    /// // The I2C device address.
    /// let address: u8 = 0;
    ///
    /// let mut bargraph = Bargraph::new(i2c, address, None);
    ///
    /// # }
    /// ```
    pub fn new<L>(i2c: I2C, i2c_address: u8, logger: L) -> Self
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
            logger,
        }
    }

    /// Initialize the Bargraph display & the connected `HT16K33` device.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # extern crate led_bargraph;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use led_bargraph::Bargraph;
    /// # fn main() {
    ///
    /// # let mut i2c = I2cMock::new(None);
    /// # let address: u8 = 0;
    ///
    /// let mut bargraph = Bargraph::new(i2c, address, None);
    /// bargraph.initialize().unwrap();
    ///
    /// # }
    /// ```
    pub fn initialize(&mut self) -> Result<(), E> {
        trace!(self.logger, "initialize");

        // Reset the display.
        self.device.initialize()?;

        Ok(())
    }

    /// Clear the Bargraph display.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # extern crate led_bargraph;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use led_bargraph::Bargraph;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new(None);
    /// # let address: u8 = 0;
    ///
    /// let mut bargraph = Bargraph::new(i2c, address, None);
    /// bargraph.clear().unwrap();
    ///
    /// # }
    /// ```
    pub fn clear(&mut self) -> Result<(), E> {
        trace!(self.logger, "clear");

        self.device.clear_display_buffer();
        self.device.write_display_buffer()
    }

    /// Update the Bargraph display, showing `range` total values with all values
    /// from `0` to `value` filled.
    ///
    /// If `value` is greater than `range`, then all bars are filled and will blink;
    /// automatic re-scaling of the range does *not* happen because:
    ///
    /// * The bargraph can only scale to a maximum resolution.
    /// * Users are already familiar with viewing the current range, and dynamically
    ///   changing the range makes it hard for users to see what's happening at a glance.
    ///
    /// # Arguments
    ///
    /// * `value` - How many values to fill, starting from `0`.
    /// * `range` - Total number of values to display.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # extern crate led_bargraph;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use led_bargraph::Bargraph;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new(None);
    /// # let address: u8 = 0;
    ///
    /// let mut bargraph = Bargraph::new(i2c, address, None);
    /// bargraph.update(5, 6, false).unwrap();
    ///
    /// # }
    /// ```
    pub fn update(&mut self, value: u8, range: u8, show: bool) -> Result<(), E> {
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
            self.update_value(current_value - 1, range, fill);
        }

        self.device.write_display_buffer()?;

        self.set_blink(blink)?;

        if show {
            self.show()?;
        }

        Ok(())
    }

    /// Enable/Disable continuous blinking of the Bargraph display.
    ///
    /// # Arguments
    ///
    /// * `enabled` - Whether to enabled blinking or not.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # extern crate led_bargraph;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use led_bargraph::Bargraph;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new(None);
    /// # let address: u8 = 0;
    ///
    /// let mut bargraph = Bargraph::new(i2c, address, None);
    /// bargraph.set_blink(true).unwrap();
    ///
    /// # }
    /// ```
    pub fn set_blink(&mut self, enabled: bool) -> Result<(), E> {
        // TODO Add support for different blink speeds.
        trace!(self.logger, "set_blink"; "enabled" => enabled);

        if enabled {
            self.device.set_display(Display::ONE_HZ)
        } else {
            self.device.set_display(Display::ON)
        }
    }

    /// Show the current bargraph display on-screen.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate ht16k33;
    /// # extern crate led_bargraph;
    /// # use ht16k33::i2c_mock::I2cMock;
    /// # use led_bargraph::Bargraph;
    /// # fn main() {
    /// # let mut i2c = I2cMock::new(None);
    /// # let address: u8 = 0;
    ///
    /// let mut bargraph = Bargraph::new(i2c, address, None);
    /// bargraph.show().unwrap();
    ///
    /// # }
    /// ```
    pub fn show(&mut self) -> Result<(), E> {
        trace!(self.logger, "show");

        // Read & retrieve the buffer values from the device.
        self.device.read_display_buffer()?;
        let &buffer = self.device.display_buffer();

        let display = self.device.display();

        // Convert the buffer values for display as LEDs.
        let mut leds = [LedColor::Off; BARGRAPH_RESOLUTION as usize];

        // The Adafruit bargraph only utilizes the first 6 rows:
        //
        // 6 rows x 8 commons == 48 LEDs == 24 bars * 2 colors
        //
        // As each row represents 8 of the 48 LEDs, many of the indexes will empty. Need to merge
        // each row together to get the complete display. When merging, if both red & green LEDs
        // are enabled, then update them to be yellow.
        for (row, common) in buffer.iter().enumerate().take(6) {
            if *display == Display::OFF {
                trace!(
                    self.logger,
                    "Display is off, don't attempt retrieve/merge the LED bars"
                );
                break;
            }

            let bars = self.row_common_to_bars(row as u8, common.bits());

            for index in 0..bars.len() {
                if let Some(color) = bars[index] {
                    match leds[index] {
                        LedColor::Green => {
                            if color == LedColor::Red {
                                leds[index] = LedColor::Yellow;
                            }
                        }
                        LedColor::Red => {
                            if color == LedColor::Green {
                                leds[index] = LedColor::Yellow;
                            }
                        }
                        LedColor::Off => {
                            leds[index] = color;
                        }
                        LedColor::Yellow => {
                            // Do nothing.
                        }
                    }
                }
            }
        }
        debug!(self.logger, "bars"; "colors" => format!("{:#?}", leds));

        // Display the LEDs.
        self.display_ascii_bargraph(&leds, *display);

        Ok(())
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
    fn update_value(&mut self, value: u8, range: u8, fill: bool) {
        trace!(self.logger, "update_value"; "value" => value, "range" => range, "fill" => fill);

        // Calculate the size of the value.
        let value_size = BARGRAPH_RESOLUTION / range;

        let start_bar = value * value_size;
        let end_bar = start_bar + value_size - 1;

        // Fill in the value.
        for current_bar in start_bar..end_bar {
            let fill_color = if fill {
                LedColor::Yellow
            } else {
                LedColor::Off
            };
            self.update_bar(current_bar, fill_color);
        }

        // Color the "top" bar of the value.
        let fill_color = if fill { LedColor::Red } else { LedColor::Green };
        self.update_bar(end_bar, fill_color);
    }

    // Set the bar to the desired color.
    //
    // The buffer must be later written using [write_display_buffer()](struct.HT16K33.html#method.write_display_buffer)
    // for the change to be displayed.
    //
    // # Arguments
    //
    // * `bar- A value from `0` to `23`.
    // * `color` - A valid color.
    #[allow(clippy::blacklisted_name)]
    fn update_bar(&mut self, bar: u8, color: LedColor) {
        trace!(self.logger, "update_bar"; "bar" => bar, "color" => format!("{:?}", color));

        let (row, common) = self.bar_to_row_common(bar);

        let red_led = ht16k33::LedLocation::new(row, common).unwrap();
        let green_led = ht16k33::LedLocation::new(row + 1, common).unwrap();

        let red_enabled = color == LedColor::Red || color == LedColor::Yellow;
        let green_enabled = color == LedColor::Green || color == LedColor::Yellow;

        self.device.update_display_buffer(red_led, red_enabled);
        self.device.update_display_buffer(green_led, green_enabled);
    }

    // This transform follows the layout of the Adafruit bargraph backpack.
    #[allow(clippy::blacklisted_name)]
    fn bar_to_row_common(&self, bar: u8) -> (u8, u8) {
        let (count, remainder) = bar.div_mod_floor(&12);
        let (mut row, mut common) = remainder.div_mod_floor(&4);
        row *= 2;
        common += count * 4;

        trace!(self.logger, "bar_to_row_common"; "bar" => bar, "row" => row, "common" => common);

        (row, common)
    }

    // For the given row & common determine the bar #'s and whether they're off, or enabled as red
    // or green. Each common "value" represents the state of 8 LEDs.
    //
    // The row determines if it's red (even) or green (odd).
    //
    // The bits of the common determine which commons are enabled.
    //
    // There are 2 LEDs per bar (1x red, 1x green), these bar #'s need to merged with the bar
    // #'s from other rows to determine if actual bar # is lit or not.
    //
    // This transform follows the layout of the Adafruit bargraph backpack.
    fn row_common_to_bars(
        &self,
        row_in: u8,
        common_in: u8,
    ) -> [Option<LedColor>; BARGRAPH_RESOLUTION as usize] {
        let mut bars = [None; BARGRAPH_RESOLUTION as usize];

        let (row, green) = row_in.div_mod_floor(&2);

        for position in 0..ht16k33::COMMONS_SIZE {
            let check = 1 << position;

            let (count, common) = (position as u8).div_mod_floor(&4);
            let remainder = row * 4 + common;
            #[allow(clippy::blacklisted_name)]
            let bar = count * 12 + remainder;
            let enabled = check == common_in & check;

            if enabled {
                bars[bar as usize] = if green == 1 {
                    Some(LedColor::Green)
                } else {
                    Some(LedColor::Red)
                };
            } else {
                bars[bar as usize] = Some(LedColor::Off);
            }
        }

        trace!(self.logger, "row_common_to_bars"; "row" => row_in, "common" => format!("{:#010b}", common_in), "bars" => format!("{:?}", bars));

        bars
    }

    // Unicode box-drawing characters: https://en.wikipedia.org/wiki/Box-drawing_character
    fn display_ascii_bargraph(&self, leds: &[LedColor], display: Display) {
        println!(
            "{corner_top_left}{line}{corner_top_right}",
            corner_top_left = White.paint("\u{2554}"),
            line = White.paint(
                std::iter::repeat("\u{2550}")
                    .take(leds.len() as usize)
                    .collect::<String>()
            ),
            corner_top_right = White.paint("\u{2557}")
        );

        print!("{side}", side = White.paint("\u{2551}"),);

        for led in leds.iter() {
            let mut style = Style::new();

            if display == Display::HALF_HZ
                || display == Display::ONE_HZ
                || display == Display::TWO_HZ
            {
                style = style.blink();
            }

            let mut color = match led {
                LedColor::Green => style.fg(Green),
                LedColor::Red => style.fg(Red),
                LedColor::Yellow => style.fg(Yellow),
                LedColor::Off => style.fg(Fixed(238)), // Dark grey.
            };

            print!("{}", color.paint(BARGRAPH_DISPLAY_CHAR));
        }

        println!("{side}", side = White.paint("\u{2551}"),);

        println!(
            "{corner_bottom_left}{line}{corner_bottom_right}",
            corner_bottom_left = White.paint("\u{255A}"),
            line = White.paint(
                std::iter::repeat("\u{2550}")
                    .take(leds.len() as usize)
                    .collect::<String>()
            ),
            corner_bottom_right = White.paint("\u{255D}")
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ht16k33::i2c_mock::I2cMock;

    const ADDRESS: u8 = 0;

    #[test]
    fn new() {
        let i2c = I2cMock::new(None);
        let _bargraph = Bargraph::new(i2c, ADDRESS, None);
    }

    #[test]
    fn initialize() {
        let i2c = I2cMock::new(None);
        let mut bargraph = Bargraph::new(i2c, ADDRESS, None);
        bargraph.initialize().unwrap();
    }

    #[test]
    fn clear() {
        let i2c = I2cMock::new(None);
        let mut bargraph = Bargraph::new(i2c, ADDRESS, None);
        bargraph.initialize().unwrap();

        bargraph.clear().unwrap();
    }

    #[test]
    fn update() {
        let i2c = I2cMock::new(None);
        let mut bargraph = Bargraph::new(i2c, ADDRESS, None);
        bargraph.initialize().unwrap();

        bargraph.update(5, 6, false).unwrap();
    }

    #[test]
    fn set_blink() {
        let i2c = I2cMock::new(None);
        let mut bargraph = Bargraph::new(i2c, ADDRESS, None);
        bargraph.initialize().unwrap();

        bargraph.set_blink(true).unwrap();
        bargraph.set_blink(false).unwrap();
    }

    #[test]
    fn show() {
        let i2c = I2cMock::new(None);
        let mut bargraph = Bargraph::new(i2c, ADDRESS, None);
        bargraph.initialize().unwrap();

        bargraph.show().unwrap();
    }
}
