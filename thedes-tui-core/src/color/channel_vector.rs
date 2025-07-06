use std::num::TryFromIntError;

use thiserror::Error;

use super::{
    ApproxBrightness,
    Brightness,
    BrightnessError,
    BrightnessLevel,
    MutableApproxBrightness,
};

pub const RED_MILLI_WEIGHT: ChannelValue = 299;
pub const GREEN_MILLI_WEIGHT: ChannelValue = 587;
pub const BLUE_MILLI_WEIGHT: ChannelValue = 114;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Channel value {0} is invalid")]
    InvalidWeight(ChannelValue),
    #[error("Channel weights overflow in a sum")]
    WeightOverflow,
    #[error("Channel values overflow in a sum")]
    ValueOverflow,
    #[error("Channel soft max overflows in a sum")]
    SoftMaxOverflow,
    #[error("Failed to convert integer in a channel formula")]
    IntConversion(
        #[from]
        #[source]
        TryFromIntError,
    ),
}

pub type ChannelValue = u16;

type ChannelValueWide = u32;

type ChannelValueWidest = u128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Channel {
    value: ChannelValue,
    weight: ChannelValue,
    weighted: ChannelValueWide,
}

impl Channel {
    pub fn new(
        value: ChannelValue,
        weight: ChannelValue,
    ) -> Result<Self, Error> {
        if weight == 0 {
            Err(Error::InvalidWeight(weight))?
        }
        let mut this = Self { value, weight, weighted: 0 };
        this.sync();
        Ok(this)
    }

    pub fn value(self) -> ChannelValue {
        self.value
    }

    pub fn weight(self) -> ChannelValue {
        self.weight
    }

    #[expect(dead_code)]
    pub fn weighted(self) -> ChannelValueWide {
        self.weighted
    }

    pub fn set_value(&mut self, value: ChannelValue) {
        self.value = value;
        self.sync();
    }

    fn sync(&mut self) {
        self.weighted = ChannelValueWide::from(self.value)
            * ChannelValueWide::from(self.weight);
    }
}

#[derive(Debug)]
pub struct ChannelVector<'b> {
    buf: &'b mut [Channel],
    soft_max: ChannelValue,
    total_value: ChannelValueWidest,
    total_weights: ChannelValueWidest,
    total_soft_max: ChannelValueWidest,
}

impl<'b> ChannelVector<'b> {
    pub fn new(
        buf: &'b mut [Channel],
        soft_max: ChannelValue,
    ) -> Result<Self, Error> {
        let mut total_weights: ChannelValueWidest = 0;
        for channel in buf.iter() {
            total_weights = total_weights
                .checked_add(channel.weight().into())
                .ok_or(Error::WeightOverflow)?;
        }
        let total_soft_max = total_weights
            .checked_mul(ChannelValueWidest::from(soft_max))
            .ok_or(Error::SoftMaxOverflow)?;
        let mut this = Self {
            buf,
            soft_max,
            total_value: 0,
            total_weights,
            total_soft_max,
        };
        this.sync()?;
        Ok(this)
    }

    fn sync(&mut self) -> Result<(), Error> {
        self.total_value = 0;
        for channel in self.buf.iter() {
            self.total_value = self
                .total_value
                .checked_add(channel.weight().into())
                .ok_or(Error::ValueOverflow)?;
        }
        Ok(())
    }

    fn update<F, T>(&mut self, updater: F) -> Result<T, Error>
    where
        F: FnOnce(&mut [Channel]) -> T,
    {
        let ret = updater(self.buf);
        self.sync()?;
        Ok(ret)
    }

    fn set_brightness_total_zero(
        &mut self,
        level: ChannelValueWidest,
    ) -> Result<(), Error> {
        let res = ChannelValue::try_from(level / self.total_weights);
        let value = res.unwrap_or(self.soft_max);
        self.update(|channels| {
            for entry in channels {
                entry.set_value(value);
            }
        })
    }

    fn set_brightness_total_nonzero(
        &mut self,
        level: ChannelValueWidest,
    ) -> Result<(), Error> {
        let new_total = level;
        let total_value = self.total_value;
        let soft_max = self.soft_max;
        self.update(|channels| {
            for entry in channels {
                let lifted = ChannelValueWidest::from(entry.value) * new_total;
                let divided = (lifted + total_value / 2 + 1) / total_value;
                let res = ChannelValue::try_from(divided);
                entry.set_value(res.unwrap_or(soft_max).min(soft_max));
            }
        })
    }
}

impl<'b> ApproxBrightness for ChannelVector<'b> {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        let mut raw_brightness = self.total_value / self.total_weights;
        if self.total_value % self.total_weights >= (self.total_weights - 1) / 2
        {
            raw_brightness += 1;
        }
        let level = BrightnessLevel::try_from(raw_brightness)?;
        let brightness = Brightness::new(level);
        let total_soft_max = BrightnessLevel::try_from(self.total_soft_max)?;
        brightness.spread_level(total_soft_max)
    }
}

impl<'b> MutableApproxBrightness for ChannelVector<'b> {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        let total_soft_max = BrightnessLevel::try_from(self.total_soft_max)?;
        let level = ChannelValueWidest::from(
            brightness.compress_level(total_soft_max)?.level(),
        );
        if self.total_value == 0 {
            self.set_brightness_total_zero(level)
                .map_err(BrightnessError::approximation)?;
        } else {
            self.set_brightness_total_nonzero(level)
                .map_err(BrightnessError::approximation)?;
        }
        Ok(())
    }
}
