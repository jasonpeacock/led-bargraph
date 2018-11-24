extern crate docopt;

extern crate ht16k33;
extern crate i2cdev;
extern crate led_bargraph;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

use docopt::Docopt;
use ht16k33::HT16K33;
use led_bargraph::Bargraph;
use slog::Drain;

// LinuxI2CDevice only works on linux, use a mock
// object to support compilation & testing on other
// platforms (e.g. OSX).
//
// Linux
#[cfg(target_os = "linux")]
use i2cdev::linux::LinuxI2CDevice;
//
// Not Linux
//
// The `MockI2CDevice` from `i2cdev` is only available
// for test builds, and is very basic. Use the more
// capable and available `MockI2CDevice` from `ht16k33`.
#[cfg(not(target_os = "linux"))]
use ht16k33::i2c_mock::MockI2CDevice;

// Docopts: https://github.com/docopt/docopt.rs
const USAGE: &'static str = "
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
    -h --help               Print this help.
    --show                  Show on-screen the current bargraph display.
    --steps=<N>             Resolution of the bargraph [default: 24].
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
    flag_show: bool,
    flag_steps: u8,
    flag_i2c_path: String,
    flag_i2c_address: u16,
}

fn main() {
    // Setup logging for the terminal (STDERR).
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();

    let logger = slog::Logger::root(drain, o!());

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    debug!(logger, "{:?}", args);

    let device_logger = logger.new(o!("mod" => "HT16K33"));

    // LinuxI2CDevice only works on linux, otherwise use a mock
    // object to support compilation & testing.
    //
    // Linux
    #[cfg(target_os = "linux")]
    let i2c_device = LinuxI2CDevice::new(args.flag_i2c_path, args.flag_i2c_address).unwrap();
    //
    // Not Linux
    #[cfg(not(target_os = "linux"))]
    let mock_logger = logger.new(o!("mod" => "HT16K33::i2c_mock"));
    #[cfg(not(target_os = "linux"))]
    let i2c_device = MockI2CDevice::new(mock_logger);

    let mut device = HT16K33::new(i2c_device, args.flag_steps, device_logger).unwrap();
    device.initialize().unwrap();

    let bargraph_logger = logger.new(o!("mod" => "bargraph"));
    let mut bargraph = Bargraph::new(device, args.flag_show, bargraph_logger);

    bargraph
        .initialize()
        .expect("Could not initialize bargraph");

    if args.cmd_clear {
        info!(logger, "Clearing the display");
        bargraph.clear().expect("Could not clear the display");
    }

    if args.cmd_set {
        info!(logger, "Setting a value in the range on the display";
              "value" => args.arg_value, "range" => args.arg_range);

        bargraph
            .update(args.arg_value, args.arg_range)
            .expect("Could not update the display");
    }

    if args.cmd_show {
        info!(logger, "Showing on-screen the current bargraph display");

        bargraph
            .show()
            .expect("Could not show on-screen the current bargraph display");
    }

    debug!(logger, "Success");
}