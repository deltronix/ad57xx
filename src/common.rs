use embedded_hal::spi::SpiDevice;


use crate::{
    Ad57xxShared,Channel, Command, CommandBits, Config, Data, Error, Function, OutputRange, PowerConfig,
};


impl<DEV, E> Ad57xxShared <DEV>
where
    DEV: SpiDevice<Error = E>,
{
    /// 
    pub fn new(spi: DEV) -> Self {
        Self {
            spi,
            cfg: Config::default(),
            pcfg: PowerConfig::default(),
        }

    }
    /// Power up or down a single or all DAC channels
    /// After power up a timeout of 10us is required before loading the corresponding DAC register
    pub fn set_power(&mut self, chan: Channel, pwr: bool) -> Result<(), Error<E>> {
        match chan {
            Channel::DacA => self.pcfg.set_pu_a(pwr),
            Channel::DacB => self.pcfg.set_pu_b(pwr),
            Channel::DacC => self.pcfg.set_pu_c(pwr),
            Channel::DacD => self.pcfg.set_pu_d(pwr),
            Channel::AllDacs => {
                self.pcfg = self
                    .pcfg
                    .with_pu_a(pwr)
                    .with_pu_b(pwr)
                    .with_pu_c(pwr)
                    .with_pu_d(pwr)
            }
        }
        self.write(Command::PowerControlRegister, Data::PowerControl(self.pcfg))
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
    pub fn set_dac_output(&mut self, chan: Channel, val: u16) -> Result<(), Error<E>>{
        self.write(Command::DacRegister(chan), Data::DacValue(val))
    }

    /// Set the device configuration
    pub fn set_config(&mut self, cfg: Config) -> Result<(), Error<E>>{
        self.write(Command::ControlRegister(Function::Config), Data::Control(cfg))
    }

    /// Set the output range of the selected DAC channel
    pub fn set_output_range(&mut self, chan: Channel, range: OutputRange) -> Result<(), Error<E>>{
        self.write(Command::RangeSelectRegister(chan), Data::OutputRange(range))
    }

    /// This function sets the DAC registers to the clear code and updates the outputs.
    pub fn clear_dacs(&mut self) -> Result<(), Error<E>>{
        self.write(Command::ControlRegister(Function::Clear), Data::None)
    }
    /// This function updates the DAC registers and, consequently, the DAC outputs. 
    pub fn load_dacs(&mut self) -> Result<(), Error<E>> {
        self.write(Command::ControlRegister(Function::Load), Data::None)
    }

    fn write(&mut self, cmd: Command, data: Data) -> Result<(), Error<E>> {
        let payload: [u8; 3] = match cmd {
            Command::DacRegister(addr) => {
                if let Data::DacValue(val) = data {
                    [
                        CommandBits::new()
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
                        CommandBits::new()
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
                        CommandBits::new()
                            .with_reg(u8::from(cmd))
                            .into(),
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
                        CommandBits::new()
                            .with_reg(u8::from(cmd))
                            .with_addr(func as u8)
                            .into(),
                        0x00,
                        u8::from(cfg),
                    ]
                } else {
                    return Err(Error::InvalidArgument);
                }
            },
            Command::ControlRegister(func) => {
                if let Data::None = data {
                    [
                    CommandBits::new()
                        .with_reg(u8::from(cmd))
                        .with_addr(func as u8)
                        .into(),
                        0x00,
                        0x00,
                    ]
                } else {
                    return Err(Error::InvalidArgument)
                }
            }
        };
        self.spi_write(&payload)
    }
    fn spi_write(&mut self, payload: &[u8]) -> Result<(), Error<E>>{
        self.spi.write(&payload).map_err(Error::Spi)
    }
    /*
    fn spi_read(&mut self, cmd: Command) -> Result<Data, Error<E>>{
        let addr = match cmd {
            Command::DacRegister(addr) => addr as u8,
            Command::RangeSelectRegister(addr) => addr as u8,
            Command::PowerControlRegister => 0,
            Command::ControlRegister(function) => function as u8,
        };
        let cmd = CommandBits::new().with_rw(true).with_reg(u8::from(cmd)).with_reg(addr);
        self.spi.write(&[u8::from(cmd), 0, 0])
    }
    */
}
