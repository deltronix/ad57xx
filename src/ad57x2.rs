//! Dual channel implementation
use bitfield_struct::bitfield;
use embedded_hal::spi::SpiDevice;

use crate::{
    Ad57xx, Ad57xxPrivate, Ad57xxShared, Command, CommandByte, Config, Data, Error, Function,
    OutputRange,
};

/// Dac Channel
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Channel {
    /// DAC Channel A
    DacA = 0,
    /// DAC Channel B
    DacB = 2,
    /// All DAC Channels
    AllDacs = 4,
}
impl From<Channel> for u8 {
    fn from(value: Channel) -> Self {
        match value {
            Channel::DacA => 0,
            Channel::DacB => 2,
            Channel::AllDacs => 4,
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

impl<DEV, E> Ad57xxShared<DEV, crate::marker::Ad57x2, Channel, PowerConfigDual>
where
    DEV: SpiDevice<Error = E>,
{
    /// Create a new dual channel AD57xx DAC SPI Device on a shared bus
    pub fn new_ad57x2(spi: DEV) -> Self {
        Self::create(spi)
    }
    /// Power up or down a single or all DAC channels
    /// After power up a timeout of 10us is required before loading the corresponding DAC register
    pub fn set_power(&mut self, chan: Channel, pwr: bool) -> Result<(), Error<E>> {
        if let Data::PowerControl(pcfg) = self.read(Command::PowerControlRegister)? {
            let pcfg = match chan {
                Channel::DacA => pcfg.with_pu_a(pwr),
                Channel::DacB => pcfg.with_pu_b(pwr),
                Channel::AllDacs => pcfg.with_pu_a(pwr).with_pu_b(pwr),
            };
            self.write(Command::PowerControlRegister, Data::PowerControl(pcfg))
        } else {
            Err(Error::ReadError)
        }
    }
}
