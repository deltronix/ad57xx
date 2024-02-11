#![doc = include_str!("../README.md")]
#![deny(unsafe_code, missing_docs)]
#![no_std]

use bitfield_struct::bitfield;
use core::include_str;
use core::marker::PhantomData;
use embedded_hal::spi::{Operation, SpiDevice};

/// AD57xx DAC with shared SPI bus access
pub struct Ad57xxShared<DEV, IC, CH, PCFG> {
    spi: DEV,
    cfg: Config,
    _pcfg: PhantomData<PCFG>,
    _ic: PhantomData<IC>,
    _ch: PhantomData<CH>,
}
impl<DEV, IC, CH, PCFG> Ad57xxShared<DEV, IC, CH, PCFG> {
    pub(crate) fn create(spi: DEV) -> Self {
        Ad57xxShared {
            spi,
            cfg: Config::default(),
            _pcfg: PhantomData,
            _ic: PhantomData,
            _ch: PhantomData,
        }
    }
    /// Return spi bus instance and SYNC pin
    pub fn destroy(self) -> DEV {
        self.spi
    }
}

/// Common functionality among the Ad57xx range
pub trait Ad57xx<DEV, E, CH, PCFG> {
    /// Write a 16 bit value to the selected DAC register.
    /// > Note that the devices with a bit depth smaller than 16 use a left-aligned data format.
    ///
    /// To push the value to the output it has to be loaded through the ~LDAC
    /// and ~SYNC pins or through the load function in the control register.
    /// The actual output voltage will depend on the reference voltage, output
    /// range and for bipolar ranges on the state of the BIN/~2sCOMPLEMENT pin.
    /// ```
    /// ad5754.set_dac_output(Channel::DacA, 0x8000);
    /// ```
    ///
    fn set_dac_output(&mut self, chan: CH, val: u16) -> Result<(), Error<E>>;

    /// Set the device configuration
    fn set_config(&mut self, cfg: Config) -> Result<(), Error<E>>;
    /// Get the device configuration
    fn get_config(&mut self) -> Result<Config, Error<E>>;

    /// Set the device power configuration
    fn set_power_config(&mut self, pcfg: PCFG) -> Result<(), Error<E>>;
    /// Get the device power configuration
    fn get_power_config(&mut self) -> Result<PCFG, Error<E>>;
    /// Set the output range of the selected DAC channel
    fn set_output_range(&mut self, chan: CH, range: OutputRange) -> Result<(), Error<E>>;
    /// This function sets the DAC registers to the clear code and updates the outputs.
    fn clear_dacs(&mut self) -> Result<(), Error<E>>;
    /// This function updates the DAC registers and, consequently, the DAC outputs.
    fn load_dacs(&mut self) -> Result<(), Error<E>>;
}
impl<DEV, E, IC, CH, PCFG> Ad57xx<DEV, E, CH, PCFG> for Ad57xxShared<DEV, IC, CH, PCFG>
where
    DEV: SpiDevice<Error = E>,
    u8: From<CH>,
    u8: From<Command<CH>>,
    u16: From<PCFG>,
    PCFG: Into<u16> + From<u16> + Copy,
    CH: Copy,
{
    fn set_dac_output(&mut self, chan: CH, val: u16) -> Result<(), Error<E>> {
        self.write(Command::DacRegister(chan), Data::DacValue(val))
    }

    fn set_config(&mut self, cfg: Config) -> Result<(), Error<E>> {
        self.cfg = cfg;
        self.write(
            Command::ControlRegister(Function::Config),
            Data::Control(self.cfg),
        )
    }
    fn get_config(&mut self) -> Result<Config, Error<E>> {
        match self.read(Command::ControlRegister(Function::Config))? {
            Data::Control(cfg) => Ok(cfg),
            _ => Err(Error::ReadError),
        }
    }
    fn set_power_config(&mut self, pcfg: PCFG) -> Result<(), Error<E>> {
        self.write(Command::PowerControlRegister, Data::PowerControl(pcfg))
    }
    fn get_power_config(&mut self) -> Result<PCFG, Error<E>> {
        if let Data::PowerControl(pcfg) = self.read(Command::PowerControlRegister)? {
            Ok(pcfg)
        } else {
            Err(Error::ReadError)
        }
    }

    fn set_output_range(&mut self, chan: CH, range: OutputRange) -> Result<(), Error<E>> {
        self.write(Command::RangeSelectRegister(chan), Data::OutputRange(range))
    }
    fn clear_dacs(&mut self) -> Result<(), Error<E>> {
        self.write(Command::ControlRegister(Function::Clear), Data::None)
    }
    fn load_dacs(&mut self) -> Result<(), Error<E>> {
        self.write(Command::ControlRegister(Function::Load), Data::None)
    }
}
trait Ad57xxPrivate<DEV, E, CH, PCFG> {
    fn spi_write(&mut self, payload: &[u8; 3]) -> Result<(), Error<E>>;
    fn spi_read(&mut self, cmd: u8) -> Result<u16, Error<E>>;
    fn write(&mut self, cmd: Command<CH>, data: Data<PCFG>) -> Result<(), Error<E>>;
    fn read(&mut self, cmd: Command<CH>) -> Result<Data<PCFG>, Error<E>>;
}
impl<DEV, E, IC, CH, PCFG> Ad57xxPrivate<DEV, E, CH, PCFG> for Ad57xxShared<DEV, IC, CH, PCFG>
where
    DEV: SpiDevice<Error = E>,
    u8: From<CH>,
    u8: From<Command<CH>>,
    u16: From<PCFG>,
    PCFG: Into<u16> + From<u16> + Copy,
    CH: Copy,
{
    fn spi_write(&mut self, payload: &[u8; 3]) -> Result<(), Error<E>> {
        self.spi
            .transaction(&mut [Operation::Write(payload)])
            .map_err(Error::Spi)
    }
    fn spi_read(&mut self, cmd: u8) -> Result<u16, Error<E>> {
        self.spi.write(&[cmd, 0, 0]).map_err(Error::Spi)?;
        let mut rx: [u8; 3] = [0x00; 3];
        // Send a NOP instruction (0x18) while reading
        self.spi
            .transfer(&mut rx, &mut [0x18, 0, 0])
            .map_err(Error::Spi)?;
        Ok(((rx[1] as u16) << 8) | rx[0] as u16)
    }
    fn write(&mut self, cmd: Command<CH>, data: Data<PCFG>) -> Result<(), Error<E>> {
        let payload: [u8; 3] = match cmd {
            Command::DacRegister(addr) => {
                if let Data::DacValue(val) = data {
                    [
                        CommandByte::new()
                            .with_addr(u8::from(addr))
                            .with_reg(u8::from(cmd))
                            .into(),
                        (val >> 8) as u8,
                        val as u8,
                    ]
                } else {
                    return Err(Error::InvalidArgument);
                }
            }
            Command::RangeSelectRegister(addr) => {
                if let Data::OutputRange(val) = data {
                    [
                        CommandByte::new()
                            .with_addr(u8::from(addr))
                            .with_reg(u8::from(cmd))
                            .into(),
                        0x00,
                        val as u8,
                    ]
                } else {
                    return Err(Error::InvalidArgument);
                }
            }
            Command::PowerControlRegister => {
                if let Data::PowerControl(pcfg) = data {
                    [
                        CommandByte::new().with_reg(u8::from(cmd)).into(),
                        (u16::from(pcfg) >> 8) as u8,
                        (u16::from(pcfg)) as u8,
                    ]
                } else {
                    return Err(Error::InvalidArgument);
                }
            }
            Command::ControlRegister(func) if func == Function::Config => {
                if let Data::Control(cfg) = data {
                    [
                        CommandByte::new()
                            .with_reg(u8::from(cmd))
                            .with_addr(func as u8)
                            .into(),
                        0x00,
                        u8::from(cfg) as u8,
                    ]
                } else {
                    return Err(Error::InvalidArgument);
                }
            }
            Command::ControlRegister(func) => {
                if let Data::None = data {
                    [
                        CommandByte::new()
                            .with_reg(u8::from(cmd))
                            .with_addr(func as u8)
                            .into(),
                        0x00,
                        0x00,
                    ]
                } else {
                    return Err(Error::InvalidArgument);
                }
            }
        };
        self.spi_write(&payload)
    }
    fn read(&mut self, cmd: Command<CH>) -> Result<Data<PCFG>, Error<E>> {
        let addr = match cmd {
            Command::DacRegister(addr) => u8::from(addr),
            Command::RangeSelectRegister(addr) => u8::from(addr),
            Command::PowerControlRegister => 0,
            Command::ControlRegister(function) => function as u8,
        };
        // The register to be read with the read bit set
        let cmd_byte = CommandByte::new()
            .with_rw(true)
            .with_reg(u8::from(cmd))
            .with_addr(addr);
        let data = self.spi_read(u8::from(cmd_byte))?;
        match cmd {
            Command::DacRegister(_) => Ok(Data::DacValue(data)),
            Command::RangeSelectRegister(_) => Ok(Data::OutputRange(OutputRange::from(data))),
            Command::PowerControlRegister => Ok(Data::PowerControl(data.try_into().unwrap())),
            Command::ControlRegister(func) if func == Function::Config => {
                Ok(Data::Control(Config::from(data as u8)))
            }
            Command::ControlRegister(_) => Err(Error::ReadError),
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
#[bitfield(u8)]
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
    #[bits(4)]
    _unused: u16,
}

/// Markers
#[doc(hidden)]
pub mod marker {
    pub struct Ad57x4 {}
    pub struct Ad57x2 {}
}

pub mod ad57x2;
pub mod ad57x4;

mod private {
    use super::marker;
    pub trait Sealed {}

    impl Sealed for marker::Ad57x4 {}
    impl Sealed for marker::Ad57x2 {}
}
