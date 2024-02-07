#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs)]
#![no_std]

use core::include_str;
use core::marker::PhantomData;

use bitfield_struct::bitfield;

/// AD57xx DAC with shared SPI bus access
pub struct Ad57xxShared<DEV, IC> {
    spi: DEV,
    cfg: Config,
    _ic: PhantomData<IC>
}
impl<DEV, IC> Ad57xxShared <DEV, IC>{
    pub(crate) fn create(spi:DEV) -> Self {
        Ad57xxShared {
            spi,
            cfg: Config::default(),
            _ic: PhantomData,
        }
    }
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
enum Data<PCFG> {
    DacValue(u16),
    OutputRange(OutputRange),
    Control(Config),
    PowerControl(PCFG),
    None,
}
/// Enum determining the contents of the Register and Address bits
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Command<C> {
    /// Access the DAC register of the channel(s)
    DacRegister(C),
    /// Access the range select register of the channel(s)
    RangeSelectRegister(C),
    /// Access the power control register
    PowerControlRegister,
    /// Access the control register
    ControlRegister(Function),
}
impl<C> From<Command<C>> for u8 {
    fn from(cmd: Command<C>) -> Self {
        match cmd {
            Command::DacRegister(_) => 0b000,
            Command::RangeSelectRegister(_) => 0b001,
            Command::PowerControlRegister => 0b010,
            Command::ControlRegister(_) => 0b011,
        }
    }
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

/// Markers
#[doc(hidden)]
pub mod marker {
    pub struct Ad57x4 {}
    pub struct Ad57x2 {}
}

#[doc(hidden)]
pub mod ad57x4;
#[doc(hidden)]
pub mod ad57x2;

mod private {
    use super::marker;
    pub trait Sealed {}

    impl Sealed for marker::Ad57x4 {}
    impl Sealed for marker::Ad57x2 {}
}
