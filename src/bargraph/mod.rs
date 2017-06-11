use i2cdev::linux::LinuxI2CError;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

mod ht16k33;

pub struct Bargraph {
    pub device: ht16k33::HT16K33,
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
                                        path: String,
                                        address: u16)
                                        -> Result<Bargraph, LinuxI2CError> {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing Bargraph";
               "size" => size, "path" => &path, "address" => address);

        let device_logger = logger.new(o!("mod" => "HT16K33"));

        let device = ht16k33::HT16K33::new(device_logger, path, address)
            .expect("Could not create HT16K33 device");

        let bargraph = Bargraph {
            device: device,
            logger: logger,
            size: size,
        };

        Ok(bargraph)
    }

    pub fn clear(&mut self) -> Result<(), LinuxI2CError> {
        self.device.clear();
        self.device.write_display()?;

        Ok(())
    }

    /// Update the display, showing up to `value` blocks filled of `range` total blocks.
    pub fn update(&mut self, value: &u8, range: &u8) -> Result<(), LinuxI2CError> {
        self.device.clear();

        for block in 1..(*range + 1) {
            let mut fill = false;
            if block <= *value {
                fill = true;
            }
            self.set_block(&fill, &(block - 1), range);
        }

        self.device.write_display()?;

        Ok(())
    }

    pub fn set_blink(&mut self, enabled: &bool) -> Result<(), LinuxI2CError> {
        if *enabled {
            self.device.set_blink(ht16k33::BLINK_2HZ)
        } else {
            self.device.set_blink(ht16k33::BLINK_OFF)
        }
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
