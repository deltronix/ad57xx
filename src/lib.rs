//! # Embedded-hal driver for Analog Devices AD57xx series of dual and quad channel 16/14/12bit DACs
//!
//! For now only the AD57x4 quad channel chips are supported. Readback operation is currently
//! untested as my hardware does not support it. If you are in the opportunity to do so please
//! let me know.
//!
//! Any contribution to this crate is welcome, as it's my first published crate any 
//! feedback is appreciated.
//!
//! ## Shared bus example on stm32f4:
//! ```
//! // Creating shared DAC SPI Device
//! use ad57xx::Ad57xxShared;
//! let mut dac = Ad57xxShared::new(RefCellDevice::new(&spi_bus, spi3_dac_sync, NoDelay));
//!
//! // Setup the DAC as desired.
//! dac.set_power(ad57xx::Channel::AllDacs, true).unwrap();
//! dac.set_output_range(ad57xx::Channel::AllDacs, ad57xx::OutputRange::Bipolar5V)
//!     .unwrap();
//! // Output a value (left-aligned 16 bit)
//! dac.set_dac_output(ad57xx::Channel::DacA, 0x9000).unwrap();
//! ```

#![deny(unsafe_code, missing_docs)]
#![no_std]

use bitfield_struct::bitfield;
pub use crate::common::Ad57xx;

/// AD57xx DAC with shared SPI bus access
pub struct Ad57xxShared<DEV> {
    spi: DEV,
    pcfg: PowerConfig,
    cfg: Config,
}


/// Errors for this crate
#[derive(Debug)]
pub enum Error<E> {
    /// SPI communication error
    Spi(E),
    /// Invalid argument
    InvalidArgument,
    /// Read Error
    ReadError,
}

#[derive(Debug)]
enum Data {
    DacValue(u16),
    OutputRange(OutputRange),
    Control(Config),
    PowerControl(PowerConfig),
    None,
}

/// Address of a function in the control register
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum Function {
    /// No-operation function used for readback operations
    Nop = 0b000,
    /// Access the configuration register
    Config = 0b001,
    /// Set the DACs to the clear code defined by CLR_SEL
    Clear = 0b100,
    /// Load the DAC register values
    Load = 0b101,
}

/// Available output ranges for the DAC channels.
/// These values are valid with a reference input of 2.5V, if the reference
/// voltage is different, consult the datasheet for the gains associated with
/// these settings.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub enum OutputRange {
    /// Gain = 2, 0V to +5V when Vref = 2.5V
    Unipolar5V = 0b000,
    /// Gain = 4, 0V to +10V when Vref = 2.5V
    Unipolar10V = 0b001,
    /// Gain = 4.32, 0V to +10.8V when Vref = 2.5V
    Unipolar10_8V = 0b010,
    /// Gain = 4, -5V to +5V when Vref = 2.5V
    Bipolar5V = 0b011,
    /// Gain = 8, -10V to +10V when Vref = 2.5V
    Bipolar10V = 0b100,
    /// Gain = 8.64, -10.8 to +10.8V when Vref = 2.5V
    Bipolar10_8V = 0b101,
    /// Invalid readback result
    InvalidReadback,
}
impl From<u16> for OutputRange {
    fn from(value: u16) -> Self {
        match value {
            0b000 => Self::Unipolar5V,
            0b001 => Self::Unipolar10V,
            0b010 => Self::Unipolar10_8V,
            0b011 => Self::Bipolar5V,
            0b100 => Self::Bipolar10V,
            0b101 => Self::Bipolar10_8V,
            _ => Self::InvalidReadback,
        }
    }
}



#[bitfield(u8)]
struct CommandByte {
    #[bits(3)]
    addr: u8,

    #[bits(3)]
    reg: u8,

    #[bits(1)]
    _zero: bool,

    #[bits(1)]
    rw: bool,
}

/// Definition of the configuration in the Control Register
#[bitfield(u16)]
pub struct Config {
    /// Set by the user to disable the SDO output. Cleared by the user to
    /// enable the SDO output (default).
    #[bits(default = false)]
    pub sdo_disable: bool,

    /// Sets the output voltage after a clear operation.
    /// ```
    /// | CLR_Select | Unipolar | Bipolar Operation   |
    /// |------------|----------|---------------------|
    /// | 0          | 0V       | 0V                  |
    /// | 1          | Midscale | Negative Full Scale |
    /// ```
    #[bits(default = false)]
    pub clr_select: bool,
    /// Set by the user to enable the current-limit clamp. The channel does not
    /// power down upon detection of an overcurrent; the current is clamped at
    /// 20 mA (default).  
    #[bits(default = true)]
    pub clamp_enable: bool,
    /// Set by the user to enable the thermal shutdown feature. Cleared by the
    /// user to disable the thermal shutdown feature (default).
    #[bits(default = false)]
    pub(crate) tsd_enable: bool,
    /// Rest of the bits are unused during config operation
    #[bits(12)]
    _unused: u16,
}
/// Definition of the power configuration register
#[bitfield(u16)]
pub struct PowerConfig {
    #[bits(1)]
    pu_a: bool,
    #[bits(1)]
    pu_b: bool,
    #[bits(1)]
    pu_c: bool,
    #[bits(1)]
    pu_d: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    tsd: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    oc_a: bool,
    #[bits(1)]
    oc_b: bool,
    #[bits(1)]
    oc_c: bool,
    #[bits(1)]
    oc_d: bool,
    #[bits(5)]
    _unused: u8,
}

/// Markers
#[doc(hidden)]
pub mod marker {
    pub enum Ad57x4 {}
}
pub mod common;
pub mod ad57x4;

mod private {
    use super::marker;
    pub trait Sealed {}

    impl Sealed for marker::Ad57x4 {}
}
