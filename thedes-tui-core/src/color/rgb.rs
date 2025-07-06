use super::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    MutableApproxBrightness,
    channel_vector::{
        self,
        BLUE_MILLI_WEIGHT,
        Channel,
        ChannelValue,
        ChannelVector,
        GREEN_MILLI_WEIGHT,
        RED_MILLI_WEIGHT,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Rgb {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Rgb {
    pub fn new(red: u8, green: u8, blue: u8) -> Self {
        Self { red, green, blue }
    }

    fn to_channel_buf(self) -> Result<[Channel; 3], channel_vector::Error> {
        Ok([
            Channel::new(ChannelValue::from(self.red), RED_MILLI_WEIGHT)?,
            Channel::new(ChannelValue::from(self.green), GREEN_MILLI_WEIGHT)?,
            Channel::new(ChannelValue::from(self.blue), BLUE_MILLI_WEIGHT)?,
        ])
    }
}

impl ApproxBrightness for Rgb {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        let mut buf =
            self.to_channel_buf().map_err(BrightnessError::approximation)?;
        ChannelVector::new(&mut buf, ChannelValue::from(u8::MAX))
            .map_err(BrightnessError::approximation)?
            .approx_brightness()
    }
}

impl MutableApproxBrightness for Rgb {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        let mut buf =
            self.to_channel_buf().map_err(BrightnessError::approximation)?;
        ChannelVector::new(&mut buf, ChannelValue::from(u8::MAX))
            .map_err(BrightnessError::approximation)?
            .set_approx_brightness(brightness)?;
        *self = Self::new(
            u8::try_from(buf[0].value())?,
            u8::try_from(buf[1].value())?,
            u8::try_from(buf[2].value())?,
        );
        Ok(())
    }
}
