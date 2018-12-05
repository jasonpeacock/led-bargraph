extern crate docopt;

extern crate ht16k33;
extern crate led_bargraph;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use docopt::Docopt;
use led_bargraph::Bargraph;
use slog::Drain;

use std::result;
use std::sync::atomic::Ordering;
use std::sync::{atomic, Arc};

// Custom Drain logic to support enabling different log levels.
struct RuntimeLevelFilter<D> {
    drain: D,
    debug: Arc<atomic::AtomicBool>,
    trace: Arc<atomic::AtomicBool>,
}

impl<D> Drain for RuntimeLevelFilter<D>
where
    D: Drain,
{
    type Ok = Option<D::Ok>;
    type Err = Option<D::Err>;

    fn log(
        &self,
        record: &slog::Record,
        values: &slog::OwnedKVList,
    ) -> result::Result<Self::Ok, Self::Err> {
        let current_level = if self.trace.load(Ordering::Relaxed) {
            slog::Level::Trace
        } else if self.debug.load(Ordering::Relaxed) {
            slog::Level::Debug
        } else {
            slog::Level::Info
        };

        if record.level().is_at_least(current_level) {
            self.drain.log(record, values).map(Some).map_err(Some)
        } else {
            Ok(None)
        }
    }
}

// The Linux I2cdevice only works on linux, use a mock
// object to support compilation & testing on other
// platforms (e.g. OSX).
//
// Linux
#[cfg(target_os = "linux")]
extern crate linux_embedded_hal;
#[cfg(target_os = "linux")]
use linux_embedded_hal::I2cdev;
//
// Not Linux
//
// Use the `I2cMock` provided by `ht16k33`.
#[cfg(not(target_os = "linux"))]
use ht16k33::i2c_mock::I2cMock;

// Docopts: https://github.com/docopt/docopt.rs
const USAGE: &str = "
LED Bargraph.

Usage:
    led-bargraph [options] clear
    led-bargraph [options] set <value> <range>
    led-bargraph [options] show
    led-bargraph (-h | --help)

Commands:
    clear   Clear the display.
    set     Display the value against the range.
    show    Show on-screen the current bargraph display.

Arguments:
    value   The value to display.
    range   The range of the bar graph to display.

Options:
    -h, --help              Print this help.
    -d, --debug             Enable verbose debug logging.
    --trace                 Enable extra-verbose trace logging.
    --as-is                 Assume device is already initialized.
    --show                  Show on-screen the current bargraph display.
    --i2c-path=<path>       Path to the I2C device [default: /dev/i2c-1].
    --i2c-address=<N>       Address of the I2C device, in decimal [default: 112].
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_clear: bool,
    cmd_set: bool,
    cmd_show: bool,
    arg_value: u8,
    arg_range: u8,
    flag_debug: bool,
    flag_trace: bool,
    flag_as_is: bool,
    flag_show: bool,
    flag_i2c_path: String,
    flag_i2c_address: u8,
}

fn main() {
    let debug = Arc::new(atomic::AtomicBool::new(false));
    let trace = Arc::new(atomic::AtomicBool::new(false));

    // Setup logging for the terminal (e.g. STDERR).
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = RuntimeLevelFilter {
        drain,
        debug: debug.clone(),
        trace: trace.clone(),
    }
    .fuse();
    let drain = slog_async::Async::new(drain)
        // It's OK to block on logging if we log too fast (e.g. `trace`).
        .overflow_strategy(slog_async::OverflowStrategy::Block)
        .build()
        .fuse();

    let logger = slog::Logger::root(drain, o!());

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    // Enable debug logging if requested. If both `--debug` and `--trace` are enabled,
    // then log level will be trace.
    debug.store(args.flag_debug, Ordering::Relaxed);
    trace.store(args.flag_trace, Ordering::Relaxed);

    debug!(logger, "{:?}", args);

    // The Linux I2cdevice only works on linux, use a mock object to support compilation & testing.
    //
    // Linux
    #[cfg(target_os = "linux")]
    let mut i2c_device = I2cdev::new(args.flag_i2c_path).unwrap();
    #[cfg(target_os = "linux")]
    i2c_device
        .set_slave_address(args.flag_i2c_address as u16)
        .unwrap();
    //
    // Not Linux
    #[cfg(not(target_os = "linux"))]
    let mock_logger = logger.new(o!("mod" => "HT16K33::i2c_mock"));
    #[cfg(not(target_os = "linux"))]
    let i2c_device = I2cMock::new(mock_logger);

    let bargraph_logger = logger.new(o!("mod" => "bargraph"));
    let mut bargraph = Bargraph::new(
        i2c_device,
        args.flag_i2c_address,
        args.flag_show,
        bargraph_logger,
    );

    if args.flag_as_is {
        info!(logger, "Not initializing the display");
        bargraph
            .initialize()
            .expect("Failed to initialize the display");
    }

    if args.cmd_clear {
        info!(logger, "Clearing the display");
        bargraph.clear().expect("Failed to clear the display");
    }

    if args.cmd_set {
        info!(logger, "Setting a value within a range on the display";
              "value" => args.arg_value, "range" => args.arg_range);

        bargraph
            .update(args.arg_value, args.arg_range)
            .expect("Failed to set a value within a range on the display");
    }

    if args.cmd_show {
        info!(logger, "Showing the current display on-screen");

        bargraph
            .show()
            .expect("Failed to show the current display on-screen");
    }

    debug!(logger, "Success");
}
