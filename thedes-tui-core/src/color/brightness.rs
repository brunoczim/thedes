use std::{num::TryFromIntError, rc::Rc, sync::Arc};

use thiserror::Error;

pub type BrightnessLevel = u16;

pub const BRIGHTNESS_CONTRAST: BrightnessLevel = u16::MAX / 2;

pub const BRIGHTNESS_ADAPTATION: BrightnessLevel = u16::MAX / 4;

#[derive(Debug, Error)]
pub enum BrightnessError {
    #[error(
        "Brightness spreading overflow, current is {current}, attempted new \
         soft maximum is {soft_max}"
    )]
    SpreadOverflow { current: BrightnessLevel, soft_max: BrightnessLevel },
    #[error("Failed to convert level back from higher levels")]
    LevelConversion(
        #[source]
        #[from]
        TryFromIntError,
    ),
    #[error("Failed to approximate brightness or set approximation")]
    Approximation(
        #[from]
        #[source]
        Box<dyn std::error::Error + Send + Sync>,
    ),
}

impl BrightnessError {
    pub fn approximation<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Approximation(Box::new(error))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Brightness {
    level: BrightnessLevel,
}

impl Brightness {
    pub const MIN: Self = Self { level: 0 };

    pub const MAX: Self = Self { level: BrightnessLevel::MAX };

    pub const fn new(level: BrightnessLevel) -> Self {
        Self { level }
    }

    pub const fn level(self) -> BrightnessLevel {
        self.level
    }

    pub fn with_level<F>(self, mapper: F) -> Self
    where
        F: FnOnce(BrightnessLevel) -> BrightnessLevel,
    {
        Self::new(mapper(self.level()))
    }

    pub fn spread_level(
        self,
        soft_max: BrightnessLevel,
    ) -> Result<Self, BrightnessError> {
        if self.level() > soft_max {
            Err(BrightnessError::SpreadOverflow {
                current: self.level(),
                soft_max,
            })?;
        }

        let curr = u32::from(self.level());
        let soft_max = u32::from(soft_max);
        let max = u32::from(Self::MAX.level());

        let compressed = (curr * max + soft_max / 2) / soft_max;
        let converted = BrightnessLevel::try_from(compressed)?;
        Ok(Self::new(converted))
    }

    pub fn spread(self, soft_max: Self) -> Result<Self, BrightnessError> {
        self.spread_level(soft_max.level())
    }

    pub fn compress_level(
        self,
        soft_max: BrightnessLevel,
    ) -> Result<Self, BrightnessError> {
        let curr = u32::from(self.level());
        let soft_max = u32::from(soft_max);
        let max = u32::from(Self::MAX.level());

        let compressed = (curr * soft_max + max / 2) / max;
        let converted = BrightnessLevel::try_from(compressed)?;
        Ok(Self::new(converted))
    }

    pub fn compress(self, soft_max: Self) -> Result<Self, BrightnessError> {
        self.compress_level(soft_max.level())
    }

    pub fn adapt_to(self, reference_brightness: Self) -> Self {
        let reference_level = reference_brightness.level();
        self.with_level(|self_level| {
            let diff = self_level.abs_diff(reference_level);
            if diff <= BRIGHTNESS_ADAPTATION {
                self_level
            } else if self_level < reference_level {
                reference_level.saturating_sub(BRIGHTNESS_ADAPTATION)
            } else {
                reference_level.saturating_add(BRIGHTNESS_ADAPTATION)
            }
        })
    }

    pub fn contrast_to(self, reference_brightness: Self) -> Self {
        let reference_level = reference_brightness.level();
        self.with_level(|self_level| {
            let diff = self_level.abs_diff(reference_level);
            if diff >= BRIGHTNESS_CONTRAST {
                self_level
            } else if self_level < reference_level {
                reference_level
                    .checked_sub(BRIGHTNESS_CONTRAST)
                    .unwrap_or_else(|| reference_level + BRIGHTNESS_CONTRAST)
            } else {
                reference_level
                    .checked_add(BRIGHTNESS_CONTRAST)
                    .unwrap_or_else(|| reference_level - BRIGHTNESS_CONTRAST)
            }
        })
    }
}

impl Default for Brightness {
    fn default() -> Self {
        Self::MAX
    }
}

impl ApproxBrightness for Brightness {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        Ok(*self)
    }
}

impl MutableApproxBrightness for Brightness {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        *self = brightness;
        Ok(())
    }
}

pub trait ApproxBrightness {
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError>;
}

impl<'a, A> ApproxBrightness for &'a A
where
    A: ApproxBrightness + ?Sized,
{
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        (**self).approx_brightness()
    }
}

impl<'a, A> ApproxBrightness for &'a mut A
where
    A: ApproxBrightness + ?Sized,
{
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        (**self).approx_brightness()
    }
}

impl<A> ApproxBrightness for Box<A>
where
    A: ApproxBrightness + ?Sized,
{
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        (**self).approx_brightness()
    }
}

impl<A> ApproxBrightness for Rc<A>
where
    A: ApproxBrightness + ?Sized,
{
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        (**self).approx_brightness()
    }
}

impl<A> ApproxBrightness for Arc<A>
where
    A: ApproxBrightness + ?Sized,
{
    fn approx_brightness(&self) -> Result<Brightness, BrightnessError> {
        (**self).approx_brightness()
    }
}

pub trait MutableApproxBrightness: ApproxBrightness {
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError>;

    fn with_approx_brightness(
        mut self,
        brightness: Brightness,
    ) -> Result<Self, BrightnessError>
    where
        Self: Sized,
    {
        self.set_approx_brightness(brightness)?;
        Ok(self)
    }

    fn set_adapt_to<A>(&mut self, other: A) -> Result<(), BrightnessError>
    where
        A: ApproxBrightness,
    {
        let self_brightness = self.approx_brightness()?;
        let other_brightness = other.approx_brightness()?;
        let new_brightness = self_brightness.adapt_to(other_brightness);
        self.set_approx_brightness(new_brightness)?;
        Ok(())
    }

    fn with_adapt_to<A>(mut self, other: A) -> Result<Self, BrightnessError>
    where
        Self: Sized,
        A: ApproxBrightness,
    {
        self.set_adapt_to(other)?;
        Ok(self)
    }

    fn set_contrast_to<A>(&mut self, other: A) -> Result<(), BrightnessError>
    where
        A: ApproxBrightness,
    {
        let self_brightness = self.approx_brightness()?;
        let other_brightness = other.approx_brightness()?;
        let new_brightness = self_brightness.contrast_to(other_brightness);
        self.set_approx_brightness(new_brightness)?;
        Ok(())
    }

    fn with_contrast_to<A>(mut self, other: A) -> Result<Self, BrightnessError>
    where
        Self: Sized,
        A: ApproxBrightness,
    {
        self.set_contrast_to(other)?;
        Ok(self)
    }
}

impl<'a, A> MutableApproxBrightness for &'a mut A
where
    A: MutableApproxBrightness + ?Sized,
{
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        (**self).set_approx_brightness(brightness)
    }
}

impl<A> MutableApproxBrightness for Box<A>
where
    A: MutableApproxBrightness + ?Sized,
{
    fn set_approx_brightness(
        &mut self,
        brightness: Brightness,
    ) -> Result<(), BrightnessError> {
        (**self).set_approx_brightness(brightness)
    }
}

#[cfg(test)]
mod test {
    use crate::color::{
        BrightnessLevel,
        brightness::{BRIGHTNESS_ADAPTATION, BRIGHTNESS_CONTRAST},
    };

    use super::Brightness;

    #[test]
    fn compress_correctly() {
        let soft_max = Brightness::new(2);
        let brightness = Brightness::new(0);
        let compressed = brightness.compress(soft_max).unwrap();
        assert_eq!(compressed.level(), 0);

        let brightness = Brightness::new(
            BrightnessLevel::MAX / 3 + BrightnessLevel::MAX / 6,
        );
        let compressed = brightness.compress(soft_max).unwrap();
        assert_eq!(compressed.level(), 1);

        let brightness = Brightness::new(
            BrightnessLevel::MAX / 3 * 2 + BrightnessLevel::MAX / 6,
        );
        let compressed = brightness.compress(soft_max).unwrap();
        assert_eq!(compressed.level(), 2);

        let brightness = Brightness::new(BrightnessLevel::MAX);
        let compressed = brightness.compress(soft_max).unwrap();
        assert_eq!(compressed.level(), 2);
    }

    #[test]
    fn spread_correctly() {
        let soft_max = Brightness::new(2);
        let brightness = Brightness::new(0);
        let spread = brightness.spread(soft_max).unwrap();
        assert!(
            spread.level() <= BrightnessLevel::MAX / 3,
            "Found level {}",
            spread.level()
        );

        let brightness = Brightness::new(1);
        let spread = brightness.spread(soft_max).unwrap();
        assert!(
            spread.level() > BrightnessLevel::MAX / 3,
            "Found level {}",
            spread.level()
        );
        assert!(
            spread.level() <= BrightnessLevel::MAX / 3 * 2 + 1,
            "Found level {}",
            spread.level()
        );

        let brightness = Brightness::new(2);
        let spread = brightness.spread(soft_max).unwrap();
        assert!(
            spread.level() > BrightnessLevel::MAX / 3 * 2 + 1,
            "Found level {}",
            spread.level()
        );
    }

    #[test]
    fn adapt_nop_correctly_self_below() {
        let brightn_self = Brightness::new(
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_ADAPTATION + 1,
        );
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let output = brightn_self.adapt_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_ADAPTATION + 1,
        );
    }

    #[test]
    fn adapt_nop_correctly_self_above() {
        let brightn_self = Brightness::new(
            BrightnessLevel::MAX / 3 * 2 + BRIGHTNESS_ADAPTATION - 1,
        );
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let output = brightn_self.adapt_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 3 * 2 + BRIGHTNESS_ADAPTATION - 1,
        );
    }

    #[test]
    fn adapt_needed_correctly_self_below() {
        let brightn_self = Brightness::new(BrightnessLevel::MAX / 100);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let output = brightn_self.adapt_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_ADAPTATION,
        );
    }

    #[test]
    fn adapt_needed_correctly_self_above() {
        let brightn_self = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 100);
        let output = brightn_self.adapt_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 100 + BRIGHTNESS_ADAPTATION,
        );
    }

    #[test]
    fn adapt_needed_correctly_self_limit_below() {
        let brightn_self = Brightness::new(
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_ADAPTATION,
        );
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let output = brightn_self.adapt_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_ADAPTATION,
        );
    }

    #[test]
    fn adapt_needed_correctly_self_limit_above() {
        let brightn_self =
            Brightness::new(BrightnessLevel::MAX / 100 + BRIGHTNESS_ADAPTATION);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 100);
        let output = brightn_self.adapt_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 100 + BRIGHTNESS_ADAPTATION,
        );
    }

    #[test]
    fn contrast_nop_correctly_self_below() {
        let brightn_self = Brightness::new(
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_CONTRAST - 1,
        );
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_CONTRAST - 1,
        );
    }

    #[test]
    fn contrast_nop_correctly_self_above() {
        let brightn_self = Brightness::new(
            BrightnessLevel::MAX / 2 - 2 + BRIGHTNESS_CONTRAST + 1,
        );
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 2 - 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 2 - 2 + BRIGHTNESS_CONTRAST + 1,
        );
    }

    #[test]
    fn contrast_needed_correctly_self_above_chosen_above() {
        let brightn_self = Brightness::new(BrightnessLevel::MAX / 2 - 2 + 10);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 2 - 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 2 - 2 + BRIGHTNESS_CONTRAST,
        );
    }

    #[test]
    fn contrast_needed_correctly_self_above_chosen_below() {
        let brightn_self = Brightness::new(BrightnessLevel::MAX / 2 + 2 + 10);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 2 + 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 2 + 2 - BRIGHTNESS_CONTRAST,
        );
    }

    #[test]
    fn contrast_needed_correctly_self_below_chosen_above() {
        let brightn_self = Brightness::new(BrightnessLevel::MAX / 2 - 2 - 10);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 2 - 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 2 - 2 + BRIGHTNESS_CONTRAST,
        );
    }

    #[test]
    fn contrast_needed_correctly_self_below_chosen_below() {
        let brightn_self = Brightness::new(BrightnessLevel::MAX / 2 + 2 - 10);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 2 + 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 2 + 2 - BRIGHTNESS_CONTRAST,
        );
    }

    #[test]
    fn contrast_at_limit_correctly_self_below() {
        let brightn_self =
            Brightness::new(BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_CONTRAST);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 3 * 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 3 * 2 - BRIGHTNESS_CONTRAST,
        );
    }

    #[test]
    fn contrast_at_limit_correctly_self_above() {
        let brightn_self =
            Brightness::new(BrightnessLevel::MAX / 2 - 2 + BRIGHTNESS_CONTRAST);
        let brightn_ref = Brightness::new(BrightnessLevel::MAX / 2 - 2);
        let output = brightn_self.contrast_to(brightn_ref);
        assert_eq!(
            output.level(),
            BrightnessLevel::MAX / 2 - 2 + BRIGHTNESS_CONTRAST,
        );
    }
}
