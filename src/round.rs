use serde::{Serialize, Deserialize};
use rand::prelude::IteratorRandom;
use strum_macros::*;
use strum::IntoEnumIterator;
use std::fmt;
use std::str::FromStr;

use super::batlog::Batlog;

#[derive(Serialize, Deserialize, Debug, EnumIter, Clone, PartialEq)]
pub enum Arena {
    Ampitheater, // does nothing
    Siphon, // no class effects
    ClimbingWall, // dominate on strength to win instantly
    Hills, // double speed
    Library, // stats lower than skill are reduced to the level of skill
    CrocPit, // draw is double loss
    SoftPlayArea, // -1 from highest stat
}

#[derive(Serialize, Deserialize, Debug, EnumIter, Clone, PartialEq)]
pub enum Modifier {
    Rulebook, // does nothing
    TheCrowdDemandsBlood, // -1 to injury and bonus stat up on kill
    MedicalAssistance, // +1 to injury
    OhShitSheHasAGun, // 10% chance to lose instantly (both players)
    PumpkinSpiceEyeExams, // all stat ups must be spent on skill (doesnt need impl here)
    // something to do with strength
    // something to do with speed
    //FineArt, // something to do with skill
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameRound {
    pub matchups: Vec<(usize, usize)>, // indexes into a list of fighters
    pub sitting_out: Option<usize>, 
    pub arena: Arena,
    pub modifier: Modifier,
    pub log: Batlog,
}

impl GameRound {
    pub fn new(matchups: Vec<(usize, usize)>, sitting_out: Option<usize>, round_no: i32) -> GameRound {
        GameRound {
            matchups, sitting_out,
            arena: Arena::iter().choose(&mut rand::thread_rng()).unwrap(), // unwrap instead of match bc iters will never be empty
            modifier: Modifier::iter().choose(&mut rand::thread_rng()).unwrap(),
            log: Batlog::new(round_no)
        }
    }
}

#[allow(unreachable_patterns)]
impl fmt::Display for Arena {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Arena::Ampitheater => "ampitheater",
            Arena::Siphon => "siphon",
            Arena::ClimbingWall => "climbing wall",
            Arena::Hills => "hills",
            Arena::Library => "library",
            Arena::CrocPit => "crocodile pit",
            Arena::SoftPlayArea => "soft play area"
        })
    }
}
impl FromStr for Arena {
    type Err = String;

    fn from_str(s: &str) -> Result<Arena, String> { // DO THIS AGAIN FOR MODS
        Ok(match s.to_lowercase().as_str() {
            "ampitheater" | "amp" => Arena::Ampitheater,
            "siphon" => Arena::Siphon,
            "climbingwall" => Arena::ClimbingWall,
            "hills" => Arena::Hills,
            "library" => Arena::Library,
            "crocpit" => Arena::CrocPit,
            "softplayarea" | "softplay" => Arena::SoftPlayArea,
            _ => return Err(format!("arena {} failed to parse!", s))
        })
    }
}

#[allow(unreachable_patterns)]
impl fmt::Display for Modifier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Modifier::Rulebook => "rulebook",
            Modifier::MedicalAssistance => "medical assistance",
            Modifier::TheCrowdDemandsBlood => "the crowd demands blood",
            Modifier::PumpkinSpiceEyeExams => "pumpkin spice eye exams",
            Modifier::OhShitSheHasAGun => "oh shit the empress has a gun" // you may wish to change this if you build this yourself
        })
    }
}

impl FromStr for Modifier {
    type Err = String;

    fn from_str(s: &str) -> Result<Modifier, String> {
        Ok(match s.to_lowercase().as_str() {
            "rulebook" | "rules" => Modifier::Rulebook,
            "medicalassistance" | "meds" => Modifier::MedicalAssistance,
            "thecrowddemandsblood" | "blood" => Modifier::TheCrowdDemandsBlood,
            "pumpkinspiceeyeexams" | "eyeexams" | "eyes" => Modifier::PumpkinSpiceEyeExams,
            "ohshitshehasagun" | "ohshit" | "gun" => Modifier::OhShitSheHasAGun,
            _ => return Err(format!("modifier {} failed to parse!", s))
        })
    }
}
