/*
 * Copyright (C) 2018 Kubos Corporation
 * Copyright (C) 2022 CUAVA
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 * July 2022:
 * - moved Stream to common hal trait
 *
 */

#![deny(missing_docs)]
#![deny(warnings)]

//! I2C device connection abstractions

use i2c_linux::I2c;
use std::io::Result;
use std::thread;
use std::time::Duration;
use hal_stream::Stream;

/// An implementation of `i2c_hal::Stream` which uses the `i2c_linux` crate
/// for communication with actual I2C hardware.
pub struct I2CStream {
    path: String,
    slave: u16,
}

impl I2CStream {
    /// Creates new I2CStream instance
    ///
    /// # Arguments
    ///
    /// `path` - File system path to I2C device handle
    /// `slave` - Address of slave I2C device
    pub fn new(path: &str, slave: u16) -> Self {
        Self {
            path: path.to_string(),
            slave,
        }
    }
}

impl Stream for I2CStream {
    type StreamError = std::io::Error;

    /// Writing
    fn write(&self, command: Vec<u8>) -> Result<()> {
        let mut i2c = I2c::from_path(self.path.clone())?;
        i2c.smbus_set_slave_address(self.slave, false)?;
        i2c.i2c_write_block_data(command[0], &command[1..])
    }

    fn write_bytes(&self, command: Vec<u8>) -> Result<()> {
        let mut i2c = I2c::from_path(self.path.clone())?;
        i2c.smbus_set_slave_address(self.slave, false)?;
        i2c.i2c_write_block_data(command[0], &command[1..])
    }

    /// Reading
    fn read(&self, command: &mut Vec<u8>, rx_len: usize) -> Result<Vec<u8>> {
        let mut i2c = I2c::from_path(self.path.clone())?;
        i2c.smbus_set_slave_address(self.slave, false)?;
        let mut data = vec![0; rx_len];
        i2c.i2c_read_block_data(command[0], &mut data)?;
        Ok(data)
    }

    /// Reads command result with Timeout
    fn read_timeout(&self, command: Vec<u8>, rx_len: usize, timeout: Duration) -> Result<Vec<u8>> {
        let mut i2c = I2c::from_path(self.path.clone())?;
        i2c.smbus_set_slave_address(self.slave, false)?;
        i2c.i2c_set_timeout(timeout)?;
        let mut data = vec![0; rx_len];
        i2c.i2c_read_block_data(command[0], &mut data)?;
        Ok(data)
    }

    /// Read/Write transaction
    fn transfer(&self, command: Vec<u8>, rx_len: usize, delay: Duration) -> Result<Vec<u8>> {
        let mut i2c = I2c::from_path(self.path.clone())?;
        i2c.smbus_set_slave_address(self.slave, false)?;
        let mut data = vec![0; rx_len];
        let mut msgs = [
            Message::Write {
                address: self.slave,
                data: &command,
                flags: if i2c.address_10bit() {
                    WriteFlags::TENBIT_ADDR
                } else {
                    WriteFlags::default()
                },
            },
            Message::Read {
                address: self.slave,
                data: &data,
                flags: if i2c.address_10bit() {
                    ReadFlags::TENBIT_ADDR
                } else {
                    ReadFlags::default()
                },
            }
            return i2c.i2c_transfer(&mut msgs).map(|_| msgs[1].len());
        ]            
        // Ok(data)
    }
}

/// Struct for abstracting I2C command/data structure
#[derive(Debug, Eq, PartialEq)]
pub struct Command {
    /// I2C command or registry
    pub cmd: u8,
    /// Data to write to registry
    pub data: Vec<u8>,
}

/// Struct for communicating with an I2C device
pub struct Connection {
    stream: Box<dyn Stream<StreamError = std::io::Error> + Send>,
}

impl Connection {
    /// I2C connection constructor
    ///
    /// # Arguments
    ///
    /// `path` - Path to I2C device
    /// `slave` - I2C slave address to read/write to
    pub fn new(stream: Box<dyn Stream<StreamError = std::io::Error> + Send>) -> Self {
        Self { stream }
    }

    /// Convenience constructor for creating a Connection with an I2CStream.
    ///
    /// # Arguments
    ///
    /// `path` - Path to I2C device
    /// `slave` - I2C slave address
    pub fn from_path(path: &str, slave: u16) -> Self {
        Self {
            stream: Box::new(I2CStream::new(path, slave)),
        }
    }

    /// Writes an I2C command
    ///
    /// # Arguments
    ///
    /// `command` - Command to write
    pub fn write(&self, command: Command) -> Result<()> {
        let mut buf = command.data;
        buf.insert(0,command.cmd);
        self.stream.write(buf)
    }

    /// Reads command result
    ///
    /// # Arguments
    ///
    /// `command` - Command to read result from
    /// `rx_len`  - Amount of data to read
    pub fn read(&self, command: Command, rx_len: usize) -> Result<Vec<u8>> {
        let mut buf = command.data;
        buf.insert(0,command.cmd);
        self.stream.read(&mut buf,rx_len)
    }    

    /// Writes I2C command and reads result
    ///
    /// # Arguments
    ///
    /// `command` - Command to write and read from
    /// `rx_len`  - Amount of data to read
    /// `delay` - Delay between writing and reading
    pub fn transfer(&self, command: Command, rx_len: usize, delay: Duration) -> Result<Vec<u8>> {
        let mut buf = command.data;
        buf.insert(0,command.cmd);
        self.stream.transfer(buf, rx_len, delay)
    }
}
