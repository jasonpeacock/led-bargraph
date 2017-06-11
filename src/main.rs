extern crate docopt;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate slog;
extern crate slog_async;
extern crate slog_term;

extern crate led_bargraph;

use docopt::Docopt;

use slog::Drain;

use led_bargraph::bargraph::Bargraph;

// Docopts: https://github.com/docopt/docopt.rs
const USAGE: &'static str = "
LED Bargraph.

Usage:
    led-bargraph clear
    led-bargraph set <value> <range>
    led-bargraph (-h | --help)

Commands:
    clear   Clear the display.
    set     Display the value against the range.

Arguments:
    value   The value to display.
    range   The range of the bar graph to display.

Options:
    -h --help               Show this screen.
    --i2c-path=<path>       Path to the I2C device [default: /dev/i2c-1].
    --i2c-address=<N>       Address of the I2C device, in decimal [default: 112].
    --bargraph-size=<N>     Size of the bargraph [default: 24].
";

#[derive(Debug, Deserialize)]
struct Args {
    cmd_set: bool,
    cmd_clear: bool,
    arg_value: u8,
    arg_range: u8,
    flag_i2c_path: String,
    flag_i2c_address: u16,
    flag_bargraph_size: u8,
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

    let bargraph_logger = logger.new(o!("mod" => "bargraph"));

    let mut bargraph = Bargraph::new(bargraph_logger,
                                     args.flag_bargraph_size,
                                     args.flag_i2c_path,
                                     args.flag_i2c_address)
            .expect("Could not create bargraph");

    if args.cmd_clear {
        info!(logger, "Clearing the display");
        bargraph.clear().expect("Could not clear the display");
    }

    if args.cmd_set {
        info!(logger, "Setting a value in the range on the display";
              "value" => args.arg_value, "range" => args.arg_range);

        let mut value = args.arg_value;
        let range = args.arg_range;
        let mut blink = false;

        // Limit `value` to be no greater than `range`, and set the display to blinking.
        if value > range {
            value = range;
            blink = true;
        }

        bargraph
            .update(&value, &range)
            .expect("Could not update the display");
        bargraph
            .set_blink(&blink)
            .expect("Could not start/stop blinking the display");
    }

    debug!(logger, "Success");
}
