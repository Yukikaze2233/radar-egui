//! Serial port data domain.
//!
//! Serial protocol parsing, data format definitions, and client transport.

#![allow(dead_code)]

pub mod data_format;
pub mod serial;
pub mod serial_package;
pub mod serial_parser;
pub mod serialconfig;
pub mod serial_crc;