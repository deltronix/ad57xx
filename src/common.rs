//! Common functionality for the AD57xx chips
use core::marker::PhantomData;
use embedded_hal::spi::SpiDevice;
use crate::{Ad57xxShared, Config, Error, OutputRange, PowerConfig};


/*

/// Trait defining Ad57xx functionality
pub trait Ad57xx<DEV, E, CH> where
    DEV: SpiDevice<Error = E>, {
    /// Power up or down a single or all DAC channels
    /// After power up a timeout of 10us is required before loading the corresponding DAC register
     fn set_power(&mut self, chan: CH, pwr: bool) -> Result<(), Error<E>>;
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

    /// Set the output range of the selected DAC channel
     fn set_output_range(&mut self, chan: CH, range: OutputRange) -> Result<(), Error<E>>;

    /// This function sets the DAC registers to the clear code and updates the outputs.
     fn clear_dacs(&mut self) -> Result<(), Error<E>>;
    /// This function updates the DAC registers and, consequently, the DAC outputs.
     fn load_dacs(&mut self) -> Result<(), Error<E>>;
}
*/
