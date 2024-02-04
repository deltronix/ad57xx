//! Driver for Analog Devices AD57xx series of dual and quad channel 16/14/12bit DACs
//!
//! For now only AD57x4 series are supported. However
//! 
//!


#![deny(unsafe_code, missing_docs)]
#![no_std]

use bitfield_struct::bitfield;

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

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Command {
    DacRegister(Channel),
    RangeSelectRegister(Channel),
    PowerControlRegister,
    ControlRegister(Function),
}

impl From<Command> for u8 {
    fn from(cmd: Command) -> Self {
        match cmd {
            Command::DacRegister(_) => 0b000,
            Command::RangeSelectRegister(_) => 0b001,
            Command::PowerControlRegister => 0b010,
            Command::ControlRegister(_) => 0b011,
        }
    }
}


#[derive(Debug)]
enum Data {
    DacValue(u16),
    OutputRange(OutputRange),
    Control(Config),
    PowerControl(PowerConfig),
    None,
}
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
enum Function {
    Nop = 0b000,
    Config = 0b001,
    Clear = 0b100,
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

/// Dac Channel
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Channel {
    /// DAC Channel A
    DacA = 0,
    /// DAC Channel B
    DacB = 1,
    /// DAC Channel C
    DacC = 2,
    /// DAC Channel D
    DacD = 3,
    /// All DAC Channels
    AllDacs = 4,
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
    /// | CLR_Select | Unipolar | Bipolar Operation   |
    /// |------------|----------|---------------------|
    /// | 0          | 0V       | 0V                  |
    /// | 1          | Midscale | Negative Full Scale |
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
#[bitfield(u16)]
struct PowerConfig {
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
    pub enum Ad5754 {}
}

mod common;

mod private {
    use super::marker;
    pub trait Sealed {}

    impl Sealed for marker::Ad5754 {}
}
