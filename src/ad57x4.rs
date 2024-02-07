//! Quad channel implementation
use bitfield_struct::bitfield;
use embedded_hal::spi::SpiDevice;

use crate::{
    Ad57xxShared, Command, CommandByte, Config, Data, Error, Function, OutputRange
};

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


#[doc(hidden)]
impl<DEV, E> Ad57xxShared<DEV, crate::marker::Ad57x4>
where
    DEV: SpiDevice<Error = E>,
{
    /// Create a new quad channel AD57xx DAC SPI Device on a shared bus
    pub fn new_ad57x4(spi: DEV) -> Self {
        Self::create(spi)
    }
    fn spi_write(&mut self, payload: &[u8]) -> Result<(), Error<E>> {
        self.spi.write(&payload).map_err(Error::Spi)
    }
    fn read(&mut self, cmd: Command<Channel>) -> Result<Data<PowerConfig>, Error<E>> {
        let addr = match cmd {
            Command::DacRegister(addr) => addr as u8,
            Command::RangeSelectRegister(addr) => addr as u8,
            Command::PowerControlRegister => 0,
            Command::ControlRegister(function) => function as u8,
        };
        // The register to be read with the read bit set
        let cmd_byte = CommandByte::new()
            .with_rw(true)
            .with_reg(u8::from(cmd))
            .with_addr(addr);
        self.spi
            .write(&[u8::from(cmd_byte), 0, 0])
            .map_err(Error::Spi)?;
        let mut rx: [u8; 3] = [0x00; 3];
        // Send a NOP instruction while reading
        let nop = CommandByte::new()
            .with_reg(u8::from(Command::<Channel>::ControlRegister(Function::Nop)))
            .with_addr(Function::Nop as u8);
        self.spi
            .transfer(&mut [u8::from(nop), 0, 0], &mut rx)
            .map_err(Error::Spi)?;
        let data: u16 = (rx[1] as u16) << 8 + rx[0] as u16;
        match cmd {
            Command::DacRegister(_) => Ok(Data::DacValue(data)),
            Command::RangeSelectRegister(_) => Ok(Data::OutputRange(OutputRange::from(data))),
            Command::PowerControlRegister => Ok(Data::PowerControl(PowerConfig::from(data))),
            Command::ControlRegister(func) if func == Function::Config => {
                Ok(Data::Control(Config::from(data)))
            }
            Command::ControlRegister(_) => Err(Error::ReadError),
        }
    }
    fn write(&mut self, cmd: Command<Channel>, data: Data<PowerConfig>) -> Result<(), Error<E>> {
        let payload: [u8; 3] = match cmd {
            Command::DacRegister(addr) => {
                if let Data::DacValue(val) = data {
                    [
                        CommandByte::new()
                            .with_addr(addr as u8)
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
                            .with_addr(addr as u8)
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
                if let Data::PowerControl(pc) = data {
                    [
                        CommandByte::new().with_reg(u8::from(cmd)).into(),
                        (u16::from(pc) >> 8) as u8,
                        (u16::from(pc)) as u8,
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
                        u16::from(cfg) as u8,
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
    /// Power up or down a single or all DAC channels
    /// After power up a timeout of 10us is required before loading the corresponding DAC register
    pub fn set_power(&mut self, chan: Channel, pwr: bool) -> Result<(), Error<E>> {
        if let Data::PowerControl(pcfg) = self.read(Command::PowerControlRegister)? {
            let pcfg = match chan {
                Channel::DacA => pcfg.with_pu_a(pwr),
                Channel::DacB => pcfg.with_pu_b(pwr),
                Channel::DacC => pcfg.with_pu_c(pwr),
                Channel::DacD => pcfg.with_pu_d(pwr),
                Channel::AllDacs => {
                    pcfg
                        .with_pu_a(pwr)
                        .with_pu_b(pwr)
                        .with_pu_c(pwr)
                        .with_pu_d(pwr)
                }
            };
            self.write(Command::PowerControlRegister, Data::PowerControl(pcfg))?
        }
        Err(Error::ReadError)
    }
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
    pub fn set_dac_output(&mut self, chan: Channel, val: u16) -> Result<(), Error<E>> {
        self.write(Command::DacRegister(chan), Data::DacValue(val))
    }
    /// Set the device configuration
    pub fn set_config(&mut self, cfg: Config) -> Result<(), Error<E>> {
        self.cfg = cfg;
        self.write(
            Command::ControlRegister(Function::Config),
            Data::Control(self.cfg),
        )
    }
    /// Get the device configuration
    pub fn get_config(&mut self) -> Result<Config, Error<E>> {
        match self.read(Command::ControlRegister(Function::Config))? {
            Data::Control(cfg) => Ok(cfg),
            _ => Err(Error::ReadError),
        }
    }

    /// Set the output range of the selected DAC channel
    pub fn set_output_range(&mut self, chan: Channel, range: OutputRange) -> Result<(), Error<E>> {
        self.write(Command::RangeSelectRegister(chan), Data::OutputRange(range))
    }

    /// This function sets the DAC registers to the clear code and updates the outputs.
    pub fn clear_dacs(&mut self) -> Result<(), Error<E>> {
        self.write(Command::ControlRegister(Function::Clear), Data::None)
    }
    /// This function updates the DAC registers and, consequently, the DAC outputs.
    pub fn load_dacs(&mut self) -> Result<(), Error<E>> {
        self.write(Command::ControlRegister(Function::Load), Data::None)
    }
}
