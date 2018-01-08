//! # Bargraph
//!
//! A library for the [Adafruit Bi-Color (Red/Green) 24-Bar Bargraph w/I2C Backpack Kit](https://www.adafruit.com/product/1721).

use std::error;
use std::fmt;

use i2cdev::core::I2CDevice;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

use ht16k33;

pub enum BargraphError<D>
where
    D: I2CDevice,
{
    /// Error from the connected `HT16K33` device.
    HT16K33(ht16k33::HT16K33Error<D>),
    /// Error from `bargraph`.
    Error,
}

impl<D> fmt::Debug for BargraphError<D>
where
    D: I2CDevice,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BargraphError: {:?}", self)
    }
}

impl<D> fmt::Display for BargraphError<D>
where
    D: I2CDevice,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BargraphError::HT16K33(ref err) => write!(f, "HT16K33 error: {}", err),
            BargraphError::Error => write!(f, "Bargraph Error"),
        }
    }
}

impl<D> error::Error for BargraphError<D>
where
    D: I2CDevice + fmt::Debug,
{
    fn description(&self) -> &str {
        match *self {
            BargraphError::HT16K33(ref err) => err.description(),
            BargraphError::Error => "Bargraph Error",
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            BargraphError::HT16K33(ref err) => Some(err),
            BargraphError::Error => None,
        }
    }
}

pub struct Bargraph<D>
where
    D: I2CDevice,
{
    device: ht16k33::HT16K33<D>,
    steps: u8,
    logger: Logger,
}

impl<D> Bargraph<D>
where
    D: I2CDevice,
{
    /// Create a Bargraph for display.
    ///
    /// # Arguments
    ///
    /// * `device` - A connected `HTK1633` device that drives the display.
    /// * `steps` - A resolution of the display.
    /// * `logger` - A logging instance.
    ///
    /// # Notes
    ///
    /// `logger = None` will log to the `slog-stdlog` drain. This makes the
    /// library effectively work the same as if it was just using `log` instead
    /// of `slog`.
    pub fn new<L>(
        device: ht16k33::HT16K33<D>,
        steps: u8,
        logger: L,
    ) -> Bargraph<D>
    where
        L: Into<Option<Logger>>,
    {
        // The `Into` [trick](http://xion.io/post/code/rust-optional-args.html) allows
        // passing `Logger` directly, without the `Some` part.
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing Bargraph"; "steps" => steps);

        Bargraph {
            device: device,
            steps: steps,
            logger: logger,
        }
    }

    /// Initialize the Bargraph display & the connected `HT16K33` device.
    ///
    /// # Errors
    ///
    /// * `BargraphError` - Either the Bargraph display or connected `HT16K33`
    /// device could not be initialized.
    pub fn initialize(&mut self) -> Result<(), BargraphError<D>> {
        debug!(self.logger, "Initializing Bargraph");

        // TODO move this outside to main, it should be initialized before being given
        // to Bargraph. Just verify that it's usable here.
        self.device.initialize().map_err(BargraphError::HT16K33)
    }

    /// Clear the Bargraph display.
    ///
    /// # Errors
    ///
    /// * `BargraphError` - The display could not be updated.
    pub fn clear(&mut self) -> Result<(), BargraphError<D>> {
        self.device.clear();
        self.device.write_display().map_err(BargraphError::HT16K33)
    }

    /// Update the Bargraph display, showing `range` total bars with all bars
    /// from `0` to `bar` filled.
    ///
    /// # Arguments
    ///
    /// * `bar` - How many bars to fill, starting from `0`.
    /// * `range` - Total number of bars to display.
    ///
    /// # Errors
    ///
    /// * `BargraphError` - The display could not be updated.
    pub fn update(&mut self, bar: &u8, range: &u8) -> Result<(), BargraphError<D>> {
        self.device.clear();

        for current_bar in 1..(*range + 1) {
            let mut fill = false;
            if current_bar <= *bar {
                fill = true;
            }
            self.set_bar_fill(&(current_bar - 1), range, &fill);
        }

        self.device.write_display().map_err(BargraphError::HT16K33)
    }

    /// Enable/Disable continuous blinking of the Bargraph display.
    ///
    /// # Errors
    ///
    /// * `BargraphError` - The display could not be updated.
    pub fn set_blink(&mut self, enabled: &bool) -> Result<(), BargraphError<D>> {
        if *enabled {
            self.device
                .set_blink(ht16k33::BLINK_2HZ)
                .map_err(BargraphError::HT16K33)
        } else {
            self.device
                .set_blink(ht16k33::BLINK_OFF)
                .map_err(BargraphError::HT16K33)
        }
    }

    /// Enable/disable the fill for a `bar` on the Bargraph display.
    ///
    /// Bar `0` is at the bottom of the display (lowest value).
    fn set_bar_fill(&mut self, bar: &u8, range: &u8, fill: &bool) {
        let bar_size = self.steps / *range;

        let start_bar = *bar * bar_size;
        let end_bar = start_bar + bar_size - 1;

        // Fill in the bar.
        for bar in start_bar..end_bar {
            if *fill {
                // Make the fill yellow if it's ON.
                self.device.set_bar(bar, ht16k33::COLOR_YELLOW);
            } else {
                // Leave it empty if above an ON bar.
                self.device.set_bar(bar, ht16k33::COLOR_OFF);
            }
        }

        // Color the marker (end of bar).
        if *fill {
            self.device.set_bar(end_bar, ht16k33::COLOR_RED);
        } else {
            self.device.set_bar(end_bar, ht16k33::COLOR_GREEN);
        }
    }
}
