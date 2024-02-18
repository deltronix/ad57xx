//! Dual channel implementation
use bitfield_struct::bitfield;
use embedded_hal::spi::{SpiDevice, Operation};

use crate::{
    Ad57xx, Ad57xxShared, Command, Data, Error, Config, Function
};

/// Dac Channel
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ChannelDual {
    /// DAC Channel A
    DacA = 0,
    /// DAC Channel B
    DacB = 2,
    /// All DAC Channels
    AllDacs = 4,
}
impl From<ChannelDual> for u8 {
    fn from(value: ChannelDual) -> Self {
        match value {
            ChannelDual::DacA => 0,
            ChannelDual::DacB => 2,
            ChannelDual::AllDacs => 4,
        }
    }
}
/// Definition of the power configuration register
#[bitfield(u16)]
pub struct PowerConfigDual {
    #[bits(1)]
    pu_a: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    pu_b: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    tsd: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    oc_a: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(1)]
    oc_b: bool,
    #[bits(1)]
    _unused: bool,
    #[bits(5)]
    _unused: u8,
}

impl<DEV, E> Ad57xxShared<DEV, crate::marker::Ad57x2>
where
    DEV: SpiDevice<Error = E>,
{
    /// Create a new dual channel AD57xx DAC SPI Device on a shared bus
    pub fn new_ad57x2(spi: DEV) -> Self {
        Self::create(spi)
    }
    /// Power up or down a single or all DAC channels
    /// After power up a timeout of 10us is required before loading the corresponding DAC register
    pub fn set_power(&mut self, chan: ChannelDual, pwr: bool) -> Result<(), Error<E>> {
        let pcfg = PowerConfigDual::from(self.pcfg);
        self.pcfg = match chan {
                ChannelDual::DacA => pcfg.with_pu_a(pwr),
                ChannelDual::DacB => pcfg.with_pu_b(pwr),
                ChannelDual::AllDacs => pcfg.with_pu_a(pwr).with_pu_b(pwr),
        }.into();
        self.write(Command::<ChannelDual>::PowerControlRegister, Data::PowerControl(self.pcfg.into()))
    }    

}
impl<DEV, E> Ad57xx<DEV, E> for Ad57xxShared<DEV, crate::marker::Ad57x2> where
DEV: SpiDevice<Error = E>,
{
    type CH = ChannelDual;
    type PCFG = PowerConfigDual;
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
    /// Set the device configuration
    fn set_config(&mut self, cfg: Config) -> Result<(), Error<E>> {
        self.cfg = cfg;
        self.write(
            Command::<Self::CH>::ControlRegister(Function::Config),
            Data::Control(cfg),
        )
    }

    /// Get the device configuration
    #[cfg(not(feature = "readback"))]
    fn get_config(&mut self) -> Result<Config, Error<E>> {
        Ok(self.cfg)
    }
    #[cfg(feature = "readback")]
    fn get_config(&mut self) -> Result<Config, Error<E>> {
        match self.read(Command::<Self::CH>::ControlRegister(Function::Config))? {
            Data::Control(cfg) => {self.cfg = cfg; Ok(cfg)},
            _ => Err(Error::ReadError),
        }
    }    
    /// Set the device power configuration
    fn set_power_config(&mut self, pcfg: Self::PCFG) -> Result<(), Error<E>> {
        self.pcfg = pcfg.into();
        self.write(Command::PowerControlRegister, Data::PowerControl(pcfg))
    }

    /// Get the device power configuration
    #[cfg(not(feature = "readback"))]
    fn get_power_config(&mut self) -> Result<Self::PCFG, Error<E>> {
        Ok(self.pcfg.into())
    }
    #[cfg(feature = "readback")]
    fn get_power_config(&mut self) -> Result<Self::PCFG, Error<E>> {
        if let Data::PowerControl(pcfg) = self.read(Command::PowerControlRegister)? {
            self.pcfg = pcfg.into();
            Ok(pcfg)
        } else {
            Err(Error::ReadError)
        }
    }
}
