use serde::{Deserialize, Serialize};

pub type StatValue = u32;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Serialize,
    Deserialize,
)]
pub struct Stat {
    value: StatValue,
    max: StatValue,
}

impl Stat {
    pub const fn new(value: StatValue, max: StatValue) -> Self {
        let mut this = Self { value, max };
        this.normalize();
        this
    }

    pub const fn value(&self) -> StatValue {
        self.value
    }

    pub fn set_value(&mut self, value: StatValue) {
        self.value = value;
        self.normalize();
    }

    pub fn increase_value(&mut self, amount: StatValue) {
        self.set_value(self.value().saturating_add(amount));
    }

    pub fn decrease_value(&mut self, amount: StatValue) {
        self.set_value(self.value().saturating_sub(amount));
    }

    pub const fn curr_max(&self) -> StatValue {
        self.max
    }

    pub fn set_max(&mut self, max: StatValue) {
        self.max = max;
        self.normalize();
    }

    pub fn increase_max(&mut self, amount: StatValue) {
        self.set_max(self.curr_max().saturating_add(amount));
    }

    pub fn decrease_max(&mut self, amount: StatValue) {
        self.set_max(self.curr_max().saturating_sub(amount));
    }

    const fn normalize(&mut self) {
        if self.value > self.max {
            self.value = self.max;
        }
    }
}
