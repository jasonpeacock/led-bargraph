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
    logger: Logger,
    is_ready: bool,
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
    /// * `logger` - A logging instance.
    ///
    /// # Notes
    ///
    /// `logger = None` will log to the `slog-stdlog` drain. This makes the
    /// library effectively work the same as if it was just using `log` instead
    /// of `slog`.
    ///
    /// The `Into` [trick](http://xion.io/post/code/rust-optional-args.html) allows
    /// passing `Logger` directly, without the `Some` part.
    ///
    /// # Examples
    ///
    /// ```
    /// // NOTE: `None` is used for the Logger in these examples for convenience,
    /// // in practice using an actual logger is preferred.
    ///
    /// // Create a mock I2C device.
    /// use led_bargraph::ht16k33::i2c_mock::MockI2CDevice;
    /// let i2c_device = MockI2CDevice::new(None);
    ///
    /// // Create a connected display.
    /// use led_bargraph::ht16k33::HT16K33;
    /// let mut device = HT16K33::new(i2c_device, 24, None).unwrap();
    /// device.initialize().unwrap();
    ///
    /// // Create a Bargraph instance.
    /// use led_bargraph::bargraph::Bargraph;
    /// let mut bargraph = Bargraph::new(device, None);
    /// ```
    pub fn new<L>(
        device: ht16k33::HT16K33<D>,
        logger: L,
    ) -> Bargraph<D>
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing Bargraph");

        Bargraph {
            device: device,
            logger: logger,
            is_ready: false,
        }
    }

    /// Initialize the Bargraph display & the connected `HT16K33` device.
    ///
    /// # Errors
    ///
    /// * `BargraphError` - Either the Bargraph display or connected `HT16K33`
    /// device could not be initialized.
    ///
    /// # Examples
    ///
    /// ```
    /// # use led_bargraph::ht16k33::HT16K33;
    /// # use led_bargraph::ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # use led_bargraph::bargraph::Bargraph;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// # let mut device = HT16K33::new(i2c_device, 24, None).unwrap();
    /// # device.initialize().unwrap();
    /// #
    /// // Create a Bargraph instance.
    /// let mut bargraph = Bargraph::new(device, None);
    ///
    /// // Initialize the bargraph.
    /// bargraph.initialize();
    /// ```
    pub fn initialize(&mut self) -> Result<(), BargraphError<D>> {
        debug!(self.logger, "Initializing Bargraph");

        // All intializations finished, ready to use.
        self.is_ready = true;

        Ok(())
    }

    /// Check if the Bargraph display is ready to be used.
    ///
    /// The Bargraph must be initialized to be ready to be used, as well
    /// as the connected `HT16K33` device.
    ///
    /// # Examples
    ///
    /// ```
    /// # use led_bargraph::ht16k33::HT16K33;
    /// # use led_bargraph::ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # use led_bargraph::bargraph::Bargraph;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// # let mut device = HT16K33::new(i2c_device, 24, None).unwrap();
    /// # device.initialize().unwrap();
    /// #
    /// // Create a Bargraph instance.
    /// let mut bargraph = Bargraph::new(device, None);
    ///
    /// // Not ready to use yet.
    /// assert_eq!(false, bargraph.is_ready());
    ///
    /// // Initialize the bargraph.
    /// bargraph.initialize();
    ///
    /// // Ready to use.
    /// assert_eq!(true, bargraph.is_ready());
    /// ```
    pub fn is_ready(&mut self) -> bool {
        self.device.is_ready() && self.is_ready
    }

    /// Clear the Bargraph display.
    ///
    /// # Errors
    ///
    /// * `BargraphError` - The display could not be updated.
    ///
    /// # Examples
    ///
    /// ```
    /// # use led_bargraph::ht16k33::HT16K33;
    /// # use led_bargraph::ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # use led_bargraph::bargraph::Bargraph;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// # let mut device = HT16K33::new(i2c_device, 24, None).unwrap();
    /// # device.initialize().unwrap();
    /// #
    /// // Create a Bargraph instance.
    /// let mut bargraph = Bargraph::new(device, None);
    /// bargraph.initialize();
    ///
    /// bargraph.clear();
    /// ```
    pub fn clear(&mut self) -> Result<(), BargraphError<D>> {
        if ! self.is_ready() {
            return Err(BargraphError::Error);
        }

        self.device.clear().map_err(BargraphError::HT16K33)?;
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
    ///
    /// # Examples
    ///
    /// ```
    /// # use led_bargraph::ht16k33::HT16K33;
    /// # use led_bargraph::ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # use led_bargraph::bargraph::Bargraph;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// # let mut device = HT16K33::new(i2c_device, 24, None).unwrap();
    /// # device.initialize().unwrap();
    /// #
    /// // Create a Bargraph instance & initialize it.
    /// let mut bargraph = Bargraph::new(device, None);
    /// bargraph.initialize();
    ///
    /// // Display a bargraph with 3 of 12 bars filled.
    /// bargraph.update(&3u8, &12u8);
    /// ```
    // TODO accept more user-friendly input values?
    pub fn update(&mut self, bar: &u8, range: &u8) -> Result<(), BargraphError<D>> {
        if ! self.is_ready() {
            return Err(BargraphError::Error);
        }

        // Reset the display in preparation for the update.
        self.device.clear().map_err(BargraphError::HT16K33)?;

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
    ///
    /// # Examples
    ///
    /// ```
    /// # use led_bargraph::ht16k33::HT16K33;
    /// # use led_bargraph::ht16k33::i2c_mock::MockI2CDevice;
    /// #
    /// # use led_bargraph::bargraph::Bargraph;
    /// #
    /// # let i2c_device = MockI2CDevice::new(None);
    /// # let mut device = HT16K33::new(i2c_device, 24, None).unwrap();
    /// # device.initialize().unwrap();
    /// #
    /// // Create a Bargraph instance & initialize it.
    /// let mut bargraph = Bargraph::new(device, None);
    /// bargraph.initialize();
    ///
    /// // Make the bargraph blink continuously.
    /// bargraph.set_blink(&true);
    // TODO accept more user-friendly input values?
    pub fn set_blink(&mut self, enabled: &bool) -> Result<(), BargraphError<D>> {
        if ! self.is_ready() {
            return Err(BargraphError::Error);
        }

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

    // Enable/disable the fill for a `bar` on the Bargraph display.
    //
    // # Arguments
    //
    // * `bar` - Which bar to fill.
    // * `range` - The total range of the display (for calculating the bar size).
    // * `fill` - Whether to fill (true) the bar or only display its header.
    //
    // # Notes
    //
    // Bar `0` is at the bottom of the display (lowest value).
    fn set_bar_fill(&mut self, bar: &u8, range: &u8, fill: &bool) {
        // Calculate the size of the bar.
        let bar_size = self.device.get_resolution() / *range;

        let start_bar = *bar * bar_size;
        let end_bar = start_bar + bar_size - 1;

        // Fill in the bar.
        for bar in start_bar..end_bar {
            if *fill {
                // Make the fill yellow if it's ON.
                let _ = self.device.set_bar(bar, ht16k33::COLOR_YELLOW).map_err(BargraphError::HT16K33);
            } else {
                // Leave it empty if above an ON bar.
                let _ = self.device.set_bar(bar, ht16k33::COLOR_OFF).map_err(BargraphError::HT16K33);
            }
        }

        // Color the bar header (end of bar).
        if *fill {
            let _ = self.device.set_bar(end_bar, ht16k33::COLOR_RED);
        } else {
            let _ = self.device.set_bar(end_bar, ht16k33::COLOR_GREEN);
        }
    }
}
