use std::error;
use std::fmt;
use std::result;

use i2cdev::core::*;
use i2cdev::linux::{LinuxI2CDevice, LinuxI2CError};

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

use num_integer::Integer;

type Result<T> = result::Result<T, HT16K33Error>;

#[derive(Debug)]
pub enum HT16K33Error {
    Device(LinuxI2CError),
}

impl fmt::Display for HT16K33Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            HT16K33Error::Device(ref err) => write!(f, "Device error: {}", err),
        }
    }
}

impl error::Error for HT16K33Error {
    fn description(&self) -> &str {
        match *self {
            HT16K33Error::Device(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            HT16K33Error::Device(ref err) => Some(err),
        }
    }
}

impl From<LinuxI2CError> for HT16K33Error {
    fn from(err: LinuxI2CError) -> HT16K33Error {
        HT16K33Error::Device(err)
    }
}

pub struct HT16K33 {
    device: LinuxI2CDevice,
    buffer: [u8; 16],
    logger: Logger,
}

const BLINK_CMD: u8 = 0x80;
const BLINK_DISPLAYON: u8 = 0x01;

pub const BLINK_OFF: u8 = 0x00;
pub const BLINK_2HZ: u8 = 0x02;
pub const BLINK_1HZ: u8 = 0x04;
pub const BLINK_HALFHZ: u8 = 0x06;

const SYSTEM_SETUP: u8 = 0x20;
const OSCILLATOR: u8 = 0x01;

const BRIGHTNESS_CMD: u8 = 0xE0;

// A bitmask value where the first bit is Green, and the second bit is
// Red.  If both bits are set the color is Yellow (Red + Green light).
pub const COLOR_OFF: u8 = 0;
pub const COLOR_GREEN: u8 = 1;
pub const COLOR_RED: u8 = 2;
pub const COLOR_YELLOW: u8 = 3;

/// Driver for interfacing with a Holtek HT16K33 16x8 LED driver,
/// which is used in the Adafruit Bi-Color 24-bar LED Bargraph I2C
/// backpack.
impl HT16K33 {
    /// Create an HT16K33 driver for the LED matrix device with the specified I2C device.
    ///
    /// `logger = None`, will log to the `slog-stdlog` drain. This makes the library
    /// effectively work the same as if it was just using `log` intead of `slog`.
    ///
    /// `Into` trick allows passing `Logger` directly, without the `Some` part.
    /// See http://xion.io/post/code/rust-optional-args.html
    pub fn new<L: Into<Option<Logger>>>(logger: L,
                                        device_i2c: LinuxI2CDevice)
                                        -> Result<HT16K33> {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing HT16K33 driver");

        let mut ht16k33 = HT16K33 {
            device: device_i2c,
            buffer: [0; 16],
            logger: logger,
        };

        ht16k33.init()?;

        Ok(ht16k33)
    }

    /// Initialize the HT16K33 driver.
    ///
    /// Sets the initial state:
    ///
    /// * System setup
    /// * Enable clock oscillator
    /// * Turn off any blinking
    /// * Maximum (15) brightness
    fn init(&mut self) -> Result<()> {
        // Turn on the oscillator.
        self.device
            .smbus_write_block_data(SYSTEM_SETUP | OSCILLATOR, &[0; 0])?;

        // Turn display on with no blinking.
        self.set_blink(BLINK_OFF)?;

        // Set display to full brightness.
        self.set_brightness(15)?;

        Ok(())
    }

    /// Blink the display at the specified frequency.
    ///
    /// Note that frequency must be a value allowed by the HT16K33, specifically one of:
    ///
    /// BLINK_OFF
    /// BLINK_2HZ
    /// BLINK_1HZ
    /// BLINK_HALFHZ
    pub fn set_blink(&mut self, frequency: u8) -> Result<()> {
        // TODO Validate 'frequency' parameter.
        self.device
            .smbus_write_block_data(BLINK_CMD | BLINK_DISPLAYON | frequency, &[0; 0])?;

        Ok(())
    }

    /// Set brightness of entire display to specified value (16 levels, from 0 to 15).
    pub fn set_brightness(&mut self, brightness: u8) -> Result<()> {
        // TODO Validate 'brightness' parameter.
        self.device
            .smbus_write_block_data(BRIGHTNESS_CMD | brightness, &[0; 0])?;

        Ok(())
    }

    /// Write display buffer to display hardware.
    pub fn write_display(&mut self) -> Result<()> {
        for value in 0..self.buffer.len() {
            self.device
                .smbus_write_byte_data(value as u8, self.buffer[value])?;
        }

        Ok(())
    }

    /// Sets specified LED (value of 0 to 127) to the specified value, False for off
    /// and True for on.
    pub fn set_led(&mut self, led: u8, enabled: bool) {
        // TODO Validate 'led' parameter.

        // Calculate position in byte buffer and get offset of desired LED.
        let (pos, offset) = led.div_mod_floor(&8);

        if enabled {
            // Turn on the specified LED (set bit to one).
            self.buffer[pos as usize] |= 1 << offset;
        } else {
            // Turn off the specified LED (set bit to zero).
            self.buffer[pos as usize] &= !(1 << offset);
        }
    }

    /// Clear contents of display buffer.
    pub fn clear(&mut self) {
        self.buffer = [0; 16];
    }

    /// Set bar to desired color. Bar should be a value of 0 to 23, and color should be
    /// OFF, GREEN, RED, or YELLOW.
    pub fn set_bar(&mut self, bar: u8, color: u8) {
        // TODO Validate 'bar' parameter.
        // TODO Validate 'color' parameter.
        // Compute cathode and anode values.
        let (c, mut a) = (if bar < 12 { bar } else { bar - 12 }).div_mod_floor(&4);
        if bar >= 12 {
            a += 4;
        }

        // Set green LED based on 1st bit in color.
        self.set_led(c * 16 + a + 8,
                     if color & COLOR_GREEN > 0 { true } else { false });

        // Set red LED based on 2nd bit in color.
        self.set_led(c * 16 + a, if color & COLOR_RED > 0 { true } else { false });
    }
}
