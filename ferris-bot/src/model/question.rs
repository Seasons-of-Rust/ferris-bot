use std::error::Error as StdError;
use std::str::FromStr;

use std::fmt;

use serenity::{
    builder::{CreateActionRow, CreateButton},
    model::interactions::message_component::ButtonStyle,
};

#[derive(Debug, PartialEq, Eq)]
pub enum QuestionTF {
    True,
    False,
}

#[derive(Debug)]
pub struct ParseComponentError(pub String);

impl fmt::Display for ParseComponentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse {} as component", self.0)
    }
}

impl StdError for ParseComponentError {}

impl fmt::Display for QuestionTF {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::True => write!(f, "True"),
            Self::False => write!(f, "False"),
        }
    }
}

impl FromStr for QuestionTF {
    type Err = ParseComponentError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "true" => Ok(Self::True),
            "false" => Ok(Self::False),
            _ => Err(ParseComponentError(s.to_string())),
        }
    }
}

impl QuestionTF {
    pub fn emoji(&self) -> char {
        match self {
            Self::True => '✅',
            Self::False => '❌',
        }
    }

    fn button(&self) -> CreateButton {
        let mut b = CreateButton::default();
        b.custom_id(self.to_string().to_ascii_lowercase());
        b.emoji(self.emoji());
        b.label(self);
        b.style(ButtonStyle::Primary);
        b
    }

    pub fn action_row() -> CreateActionRow {
        let mut ar = CreateActionRow::default();
        // We can add up to 5 buttons per action row
        ar.add_button(QuestionTF::True.button());
        ar.add_button(QuestionTF::False.button());
        ar
    }
}
