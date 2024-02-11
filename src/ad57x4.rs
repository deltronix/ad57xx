//! Quad channel implementation
use bitfield_struct::bitfield;
use embedded_hal::spi::{Operation, SpiDevice};

use crate::{
    Ad57xx, Ad57xxPrivate, Ad57xxShared, Command, CommandByte, Config, Data, Error, Function,
    OutputRange,
};

/// Dac Channel
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum ChannelQuad {
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
impl From<ChannelQuad> for u8 {
    fn from(value: ChannelQuad) -> Self {
        match value {
            ChannelQuad::DacA => 0,
            ChannelQuad::DacB => 1,
            ChannelQuad::DacC => 2,
            ChannelQuad::DacD => 3,
            ChannelQuad::AllDacs => 4,
        }
    }
}
/// Definition of the power configuration register
#[bitfield(u16)]
pub struct PowerConfigQuad {
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

impl<DEV, E> Ad57xxShared<DEV, crate::marker::Ad57x4, ChannelQuad, PowerConfigQuad>
where
    DEV: SpiDevice<Error = E>,
{
    /// Create a new quad channel AD57xx DAC SPI Device on a shared bus
    pub fn new_ad57x4(spi: DEV) -> Self {
        Self::create(spi)
    }
    /// Power up or down a single or all DAC channels
    /// After power up a timeout of 10us is required before loading the corresponding DAC register
    pub fn set_power(&mut self, chan: ChannelQuad, pwr: bool) -> Result<(), Error<E>> {
        if let Data::PowerControl(pcfg) = self.read(Command::PowerControlRegister)? {
            let pcfg = match chan {
                ChannelQuad::DacA => pcfg.with_pu_a(pwr),
                ChannelQuad::DacB => pcfg.with_pu_b(pwr),
                ChannelQuad::DacC => pcfg.with_pu_c(pwr),
                ChannelQuad::DacD => pcfg.with_pu_d(pwr),
                ChannelQuad::AllDacs => pcfg
                    .with_pu_a(pwr)
                    .with_pu_b(pwr)
                    .with_pu_c(pwr)
                    .with_pu_d(pwr),
            };
            self.write(Command::PowerControlRegister, Data::PowerControl(pcfg))
        }
        else{
            Err(Error::ReadError)
        }
    }
}
