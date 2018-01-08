// Copied and lightly modified from:
// https://github.com/rust-embedded/rust-i2cdev/blob/master/src/mock.rs
//
// Original License:
//
// Copyright 2015, Paul Osborne <osbpau@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/license/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option.  This file may not be copied, modified, or distributed
// except according to those terms.
use std::error;
use std::fmt;
use std::result;

use i2cdev::core::I2CDevice;

use slog::Drain;
use slog::Logger;
use slog_stdlog::StdLog;

pub type I2CResult<T> = result::Result<T, MockI2CDeviceError>;

pub struct I2CRegisterMap {
    registers: [u8; 0xFF],
    offset: usize,
    logger: Logger,
}

impl I2CRegisterMap {
    pub fn new<L>(logger: L) -> I2CRegisterMap
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        trace!(logger, "Constructing I2CRegisterMap");

        I2CRegisterMap {
            registers: [0x00; 0xFF],
            offset: 0,
            logger: logger,
        }
    }

    pub fn write_regs(&mut self, offset: usize, data: &[u8]) {
        trace!(self.logger, "WRITE";
               "register" => format!("0x{:X}", offset),
               "data" => format!("{:?}", data));
        for i in 0..data.len() {
            self.registers[offset + i] = data[i];
        }
    }

    /// Read data from the device to fill the provided buffer
    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        for i in 0..data.len() {
            data[i] = self.registers[self.offset];
            self.offset += 1;
        }
        trace!(self.logger, "READ";
               "register" => format!("0x{:X}", self.offset - data.len()),
               "data" => format!("{:?}", data));
        Ok(())
    }

    /// Write the provided buffer to the device
    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        // ASSUMPTION: first byte sets the offset
        // ASSUMPTION: write has length of at least one (will panic)
        let offset = data[0] as usize;
        let remdata = &data[1..];
        self.write_regs(offset, remdata);
        self.offset = offset + remdata.len();
        Ok(())
    }
}

pub struct MockI2CDevice {
    pub regmap: I2CRegisterMap,
    logger: Logger,
}

#[derive(Debug)]
pub struct MockI2CDeviceError;

impl fmt::Display for MockI2CDeviceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "MockI2CDeviceError!")
    }
}

impl error::Error for MockI2CDeviceError {
    fn description(&self) -> &str {
        "MockI2CDeviceError!"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}

impl MockI2CDevice {
    pub fn new<L>(logger: L) -> MockI2CDevice
    where
        L: Into<Option<Logger>>,
    {
        let logger = logger.into().unwrap_or(Logger::root(StdLog.fuse(), o!()));

        debug!(logger, "Constructing MockI2CDevice");

        let regmap_logger = logger.new(o!("mod" => "HT16K33::i2c_mock::I2CRegisterMap"));

        MockI2CDevice {
            regmap: I2CRegisterMap::new(regmap_logger),
            logger: logger,
        }
    }
}

impl I2CDevice for MockI2CDevice {
    type Error = MockI2CDeviceError;

    fn read(&mut self, data: &mut [u8]) -> I2CResult<()> {
        debug!(self.logger, "read");
        self.regmap.read(data)
    }

    fn write(&mut self, data: &[u8]) -> I2CResult<()> {
        debug!(self.logger, "write";
               "data" => format!("{:?}", data));
        self.regmap.write(data)
    }

    fn smbus_write_quick(&mut self, _bit: bool) -> I2CResult<()> {
        debug!(self.logger, "smbus_write_quick";
               "bit" => _bit);
        Ok(())
    }

    fn smbus_read_block_data(&mut self, _register: u8) -> I2CResult<Vec<u8>> {
        debug!(self.logger, "smbus_read_block_data";
               "register" => format!("0x{:X}", _register));
        Ok(Vec::new())
    }

    fn smbus_write_block_data(&mut self, _register: u8, _values: &[u8]) -> I2CResult<()> {
        debug!(self.logger, "smbus_write_block_data";
               "register" => format!("0x{:X}", _register),
               "values" => format!("{:?}", _values));
        Ok(())
    }

    fn smbus_process_block(&mut self, _register: u8, _values: &[u8]) -> I2CResult<()> {
        debug!(self.logger, "smbus_process_block";
               "register" => format!("0x{:X}", _register),
               "values" => format!("{:?}", _values));
        Ok(())
    }

    fn smbus_read_i2c_block_data(&mut self, _register: u8, _len: u8) -> I2CResult<Vec<u8>> {
        debug!(self.logger, "smbus_read_i2c_block_data";
               "register" => format!("0x{:X}", _register),
               "length" => _len);
        Ok(Vec::new())
    }
}
