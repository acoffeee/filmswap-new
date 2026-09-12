/*
CREATE TABLE users (
    discord_id INTEGER PRIMARY KEY,
    username TEXT UNIQUE,
    letter TEXT,
    submitted_gift TEXT,
    giftee_id INTEGER
);
*/

use poise::ChoiceParameter;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct UserInfo {
    pub discord_id: u64,
    pub username: String,
    pub letter: Option<String>,
    pub submission: Option<String>,
    pub giftee_id: Option<u64>,
    pub is_banned: bool,
    pub has_joined: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ChoiceParameter)]
pub enum Phase {
    Join,
    Swap,
    Watch,
}
#[derive(Debug, PartialEq, ChoiceParameter, Copy, Clone)]
pub enum GraphLayout {
    Dot,
    Neato,
    Fdp,
    Circo,
    Twopi,
    Osage,
    Patchwork,
    Default, //circo
    Random,
}

impl Phase {
    pub fn as_str(&self) -> &'static str {
        match self {
            Phase::Join => "Join",
            Phase::Swap => "Swap",
            Phase::Watch => "Watch",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "Join" => Some(Phase::Join),
            "Swap" => Some(Phase::Swap),
            "Watch" => Some(Phase::Watch),
            _ => None,
        }
    }
}
