mod menu;
mod info;
mod input;

pub use self::{
    info::InfoDialog,
    input::InputDialog,
    menu::{DangerPromptOption, Menu},
};
use crate::graphics::{ColoredGString, ColorsKind};

/// A struct representing an UI's labeled option's labels.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Labels {
    /// The option's label when selected.
    pub selected: ColoredGString<ColorsKind>,
    /// The option's label when unselected.
    pub unselected: ColoredGString<ColorsKind>,
}

impl LabeledOption for Labels {
    fn label(&self, selected: bool) -> ColoredGString<ColorsKind> {
        if selected {
            self.selected()
        } else {
            self.unselected()
        }
    }

    fn unselected(&self) -> ColoredGString<ColorsKind> {
        self.unselected.clone()
    }

    fn selected(&self) -> ColoredGString<ColorsKind> {
        self.selected.clone()
    }

    fn labels(&self) -> ColoredGString<ColorsKind> {
        self.clone()
    }
}

/// A trait representing an UI's named option.
pub trait LabeledOption {
    /// Returns the label of this option given the parameter telling it is
    /// selected or not.
    fn label(&self, selected: bool) -> ColoredGString<ColorsKind>;

    /// Returns the label of this option when unselected.
    fn unselected(&self) -> ColoredGString<ColorsKind> {
        self.label(false)
    }

    /// Returns the label of this option when selected.
    fn selected(&self) -> ColoredGString<ColorsKind> {
        self.label(true)
    }

    /// Unites every label into a struct.
    fn labels(&self) -> Labels {
        Labels { selected: self.selected(), unselected: self.unselected() }
    }
}
