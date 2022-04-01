use serde::{Serialize, Deserialize};
use rand::prelude::IteratorRandom;
use strum_macros::*;
use strum::IntoEnumIterator;
use std::fmt;
use std::str::FromStr;
use rand::thread_rng;
use rand::seq::SliceRandom;

use crate::boss::BossRound;
use crate::batlog::Batlog;
use crate::fighter::Fighter;


#[derive(Serialize, Deserialize, Debug, EnumIter, Clone, PartialEq)]
pub enum Arena {
    Ampitheater, // does nothing
    // this is spelled wrong SHUT UP
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
    OlympicInspector,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameRound {
    pub matchups: Vec<(usize, usize)>, // indexes into a list of fighters
    pub sitting_out: Option<usize>, 
    pub arena: Arena,
    pub modifier: Modifier,
    pub log: Batlog,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Round {
    Standard(GameRound),
    Boss(BossRound)
}

impl GameRound {
    pub fn new(fighters: &Vec<Fighter>, pre_matches: &mut Vec<(usize, usize)>, round_no: i32, arena: Option<Arena>, modifier: Option<Modifier>) -> GameRound {
        let modifier = match modifier {
            None => Modifier::iter().choose(&mut rand::thread_rng()).unwrap(),
            Some(m) => m
        };
        let (mut matchups, sitting_out) = match modifier {
            Modifier::OlympicInspector => {
                generate_olympics(fighters)
            }
            _ => {
                generate_matchups(fighters)
            }
        };
        matchups.append(pre_matches);

        let arena = match arena {
            None => Arena::iter().choose(&mut rand::thread_rng()).unwrap(),
            Some(a) => a
        };

        GameRound {
            matchups, sitting_out, arena, modifier,
            log: Batlog::new(round_no)
        }
    }
}

#[allow(unreachable_patterns)]
impl fmt::Display for Arena {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Arena::Ampitheater => "amphitheatre",
            Arena::Siphon => "siphon",
            Arena::ClimbingWall => "climbing wall",
            Arena::Hills => "hills",
            Arena::Library => "mech suits",
            Arena::CrocPit => "crocodile pit",
            Arena::SoftPlayArea => "soft play area"
        })
    }
}
impl FromStr for Arena {
    type Err = String;

    fn from_str(s: &str) -> Result<Arena, String> {
        Ok(match s.to_lowercase().as_str() {
            "amphitheater" | "amp" | "amphithetre" => Arena::Ampitheater,
            "siphon" => Arena::Siphon,
            "climbingwall" => Arena::ClimbingWall,
            "hills" => Arena::Hills,
            "library" | "mechsuits" | "mechs" => Arena::Library,
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
            Modifier::OhShitSheHasAGun => "oh shit the empress has a gun", // you may wish to change this if you build this yourself
            Modifier::OlympicInspector => "olympic inspector"
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
            "olympicinspector" | "olympic" | "inspector" => Modifier::OlympicInspector,
            _ => return Err(format!("modifier {} failed to parse!", s))
        })
    }
}

fn generate_matchups(fighters: &[Fighter]) -> (Vec<(usize, usize)>, Option<usize>) {
    let mut ret: Vec<(usize, usize)> = Vec::new();
    let mut living_fighters: Vec<usize> = Vec::new();
    
    for i in 0..fighters.len() { // select fighters elegible for auto matching
        if !fighters[i].dead && !fighters[i].pre_matched { // dead fighters can't fight, pre matched fighters should not be auto matched
            living_fighters.push(i);
        }
    }

    //println!("{:?}", living_fighters);
    
    living_fighters.shuffle(&mut thread_rng()); // shuffle
    
    let sitting_out = if living_fighters.len() % 2 != 0 { // odd number of fighters
        living_fighters.pop() // this is easier than impling 3 ways
    }
    else {
        None
    };

    for i in (0..living_fighters.len()).step_by(2) { // step through in pairs
        ret.push((living_fighters[i], living_fighters[i + 1])); // list SHOULD only ever be multiple of 2 length
    }
    
    (ret, sitting_out)
}

fn generate_olympics(fighters: &[Fighter]) -> (Vec<(usize, usize)>, Option<usize>) {
    let mut ret: Vec<(usize, usize)> = Vec::new();
    let mut living_fighters: Vec<usize> = Vec::new();
    
    for i in 0..fighters.len() { // select fighters elegible for auto matching
        if !fighters[i].dead && !fighters[i].pre_matched { // dead fighters can't fight, pre matched fighters should not be auto matched
            living_fighters.push(i);
        }
    }
    
    living_fighters.shuffle(&mut thread_rng()); // shuffle
    
    let sitting_out = if living_fighters.len() % 2 != 0 { // odd number of fighters
        living_fighters.pop() // this is easier than impling 3 ways
    }
    else {
        None
    };

    living_fighters.sort_by(|a, b| fighters[*a].rating.partial_cmp(&fighters[*b].rating).unwrap());
    for i in (0..living_fighters.len()).step_by(2) { // step through in pairs
        ret.push((living_fighters[i], living_fighters[i + 1])); // list SHOULD only ever be multiple of 2 length
    }

    (ret, sitting_out)
}