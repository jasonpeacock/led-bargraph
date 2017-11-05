use std::error;
use std::fmt;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

use ht16k33;

pub enum BargraphError {
    HT16K33(ht16k33::HT16K33Error),
    Error,
}

impl fmt::Debug for BargraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BargraphError: {:?}", self)
    }
}

impl fmt::Display for BargraphError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            BargraphError::HT16K33(ref err) => write!(f, "Device error: {}", err),
            BargraphError::Error => write!(f, "Bargraph Error"),
        }
    }
}

impl error::Error for BargraphError {
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

pub struct Bargraph {
    device: ht16k33::HT16K33,
    logger: Logger,
    size: u8,
}

impl Bargraph {
    /// `logger = None`, will log to the `slog-stdlog`
    /// drain. This makes the library effectively work the same
    /// as it was just using `log` intead of `slog`.
    ///
    /// `Into` trick allows passing `Logger` directly, without the `Some` part.
    /// See http://xion.io/post/code/rust-optional-args.html
    pub fn new<L: Into<Option<Logger>>>(logger: L,
                                        size: u8,
                                        device: ht16k33::HT16K33)
                                        -> Result<Bargraph, BargraphError> {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing Bargraph"; "size" => size);

        let bargraph = Bargraph {
            device: device,
            logger: logger,
            size: size,
        };

        Ok(bargraph)
    }

    pub fn clear(&mut self) -> Result<(), BargraphError> {
        self.device.clear();
        self.device.write_display().map_err(BargraphError::HT16K33);

        Ok(())
    }

    /// Update the display, showing up to `value` blocks filled of `range` total blocks.
    pub fn update(&mut self, value: &u8, range: &u8) -> Result<(), BargraphError> {
        self.device.clear();

        for block in 1..(*range + 1) {
            let mut fill = false;
            if block <= *value {
                fill = true;
            }
            self.set_block(&fill, &(block - 1), range);
        }

        self.device.write_display().map_err(BargraphError::HT16K33);

        Ok(())
    }

    pub fn set_blink(&mut self, enabled: &bool) -> Result<(), BargraphError> {
        if *enabled {
            self.device.set_blink(ht16k33::BLINK_2HZ).map_err(BargraphError::HT16K33);
        } else {
            self.device.set_blink(ht16k33::BLINK_OFF).map_err(BargraphError::HT16K33);
        }

        Ok(())
    }

    /// Fill in a "block" on the bargraph.
    ///
    /// Block 0 is at the bottom of the bar (lowest values).
    fn set_block(&mut self, fill: &bool, block: &u8, range: &u8) {
        let block_size = self.size / *range;

        let start_block = *block * block_size;
        let end_block = start_block + block_size - 1;

        // Fill in the bar.
        for bar in start_block..end_block {
            if *fill {
                // Make the fill yellow if it's ON.
                self.device.set_bar(bar, ht16k33::COLOR_YELLOW);
            } else {
                // Leave it empty if above ON blocks.
                self.device.set_bar(bar, ht16k33::COLOR_OFF);
            }
        }

        // Color the marker (end of block).
        if *fill {
            self.device.set_bar(end_block, ht16k33::COLOR_RED);
        } else {
            self.device.set_bar(end_block, ht16k33::COLOR_GREEN);
        }
    }
}
