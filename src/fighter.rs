use serde::{Serialize, Deserialize};
use rand::{thread_rng, Rng};
use std::fmt;
use std::str::FromStr;

use super::utils;
use super::batlog::Batlog;
use super::round::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Fighter {
    pub name: String,
    pub owner: String,
    pub class: Class,
    pub strength: i32,
    pub speed: i32,
    pub skill: i32,
    pub dead: bool,
    pub rating: i32,
    pub kills: i32,
    pub battles_won: i32,
    pub battles_fought: i32,
    pub unspent_points: i32,
    pub pre_matched: bool, // whether the player has organised a matchup
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub enum Class {
    Swarm, // 2d5 instead of d10
    Dom, // additional bonus point on domination
    // called "angry skeleton" for reasons of 14 year olds
    Turtle, // opponent doesn't get bonus points on domination
    Tank, // roll 2 injury rolls and pick the highest
    // called "chicken" in game
    Mutant, // roll 2 dice for a random stat and pick the highest
    Cleric, // win on a draw
    Naked, // extra point to start with
    // called "senator" in game for reasons of 14 year olds
}

impl Default for Class {
    fn default() -> Class {Class::Mutant}
}

impl Fighter {
    pub fn new(name: String, owner: String, class: Class, strength: i32, speed: i32, skill: i32) -> Fighter{
        Fighter {
            name, owner, class, strength, speed, skill,
            ..Fighter::default()
        }
    }

    pub fn from_vec(v: &[String]) -> Result<Fighter, String> { // used to add fighters from the command line
        match v.len() { // check you have all the fields
            6 => {}
            _ => {
                return Err(format!("fighter vec parsing requires vec of length 6"))
            }
        }

        let name = String::from(&v[0]);
        let owner = String::from(&v[1]);
        let class = match v[2].parse::<Class>() { // check everything parses and bail out if anything errors
            Ok(v) => v,
            Err(e) => return Err(e)
        };
        let strength = match v[3].parse::<i32>() {
            Ok(v) => v,
            Err(_) => return Err(format!("strength value {} failed to parse!", v[3]))
        };
        let speed = match v[4].parse::<i32>() {
            Ok(v) => v,
            Err(_) => return Err(format!("strength value {} failed to parse!", v[4]))
        };
        let skill = match v[5].parse::<i32>() {
            Ok(v) => v,
            Err(_) => return Err(format!("strength value {} failed to parse!", v[5]))
        };

        Ok(Fighter::new(name, owner, class, strength, speed, skill))
    }

    pub fn total(&self) -> i32 {
        self.strength + self.speed + self.skill + self.unspent_points
    }

    #[allow(unused_variables)]
    pub fn injure(&mut self, arena: &Arena, modifier: &Modifier, batlog: &mut Batlog, will_injure: bool) -> Option<i32> {
        if !will_injure { // work this out in the battle method
            batlog.set_injury(None);
            return None
        }

        let roll = if let Arena::Siphon = arena {thread_rng().gen_range(0..8)} 
        else {
            match self.class {
                Class::Tank => {
                    utils::select_largest(thread_rng().gen_range(0..8), thread_rng().gen_range(0..8)) // best of 2 rolls
                }
                _ => thread_rng().gen_range(0..8) // no other classes affect injury rolls (yet)
            }
        };

        batlog.set_injury(Some(roll));

        match roll { // injury table
            i32::MIN..=0 => { // injury rolls can go negative
                self.dead = true;
            }
            1 => {
                self.strength -= 1;
                self.speed -= 1;
                self.skill -= 1
            }
            2 => {
                self.strength -= 1
            }
            3 => {
                self.speed -= 1
            }
            4 => {
                self.skill -= 1
            }
            _ => {}
        }
        return Some(roll)
    }

    pub fn roll_for_stats(&self, arena: &Arena, modifier: &Modifier) -> Vec<i32> {
        let mut stats = vec![self.strength, self.speed, self.skill]; // easier to manupilate a vec later than an array

        match arena {
            Arena::Hills => {
                stats[1] *= 2; // double speed
            }
            Arena::Library => {
                stats[0] = utils::select_largest(stats[0], stats[2]);
                stats[1] = utils::select_largest(stats[1], stats[2]);
            }
            _ => {}
        }

        match modifier { // not entirely sure i need this but whatever
            _ => {}
        }
         
        let mut mutant_roll = 4usize; // saves checking if the fighter is a mutant twice
        if let Arena::Siphon = arena {} else { // no class effects if its siphon
            match self.class {
                Class::Swarm => { // 2d5
                    for i in 0..3 {
                        stats[i] += thread_rng().gen_range(1..6) + thread_rng().gen_range(1..6);

                        return stats;
                    }
                }
                Class::Mutant => { // random stat gets best of 2 rolls
                    mutant_roll = thread_rng().gen_range(0..3);
                    stats[mutant_roll] = utils::select_largest(thread_rng().gen_range(1..11), thread_rng().gen_range(1..11))
                }
                _ => {}
            }
        }


        for i in 0..3 {
            if i != mutant_roll { // don't roll for the mutant stat
                stats[i] += thread_rng().gen_range(1..11);
            }
        }

        stats
    }
}

/*impl Class {
    pub fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s.to_lowercase().as_str() {
            "swarm" => Class::Swarm,
            "dom" => Class::Dom,
            "turtle" => Class::Turtle,
            "tank" => Class::Tank,
            "mutant" => Class::Mutant,
            "cleric" => Class::Cleric,
            "naked" => Class::Naked,
            _ => return Err(format!("class {} not recognised!", s))
        })
    }
}*/

impl FromStr for Class { // much cleaner
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        Ok(match s.to_lowercase().as_str() {
            "swarm" => Class::Swarm,
            "dom" | "skeleton" => Class::Dom,
            "turtle" => Class::Turtle,
            "tank" | "chicken" => Class::Tank,
            "mutant" => Class::Mutant,
            "cleric" => Class::Cleric,
            "naked" | "senator" => Class::Naked,
            _ => return Err(format!("class {} failed to parse!", s))
        })
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Class::Cleric => "cleric",
            Class::Dom => "skeleton",
            Class::Turtle => "turtle",
            Class::Tank => "chicken",
            Class::Mutant => "mutant",
            Class::Swarm => "swarm",
            Class::Naked => "senator",
        })
    }
}
