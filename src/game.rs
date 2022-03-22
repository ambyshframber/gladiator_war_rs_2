use serde::{Serialize, Deserialize};
use rand::thread_rng;
use rand::seq::SliceRandom;
use std::fs;
use pad::{PadStr, Alignment};
use argparse::{ArgumentParser, StoreOption};
use std::io::{stdout, stderr};

use super::fighter::*;
use super::round::{GameRound, Arena, Modifier};
use super::battle;
use super::utils::{ProgramOptions, fmt_vec, fmt_option, get_non_repeating_filename, fmt_vec_with_tabs};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub fighters: Vec<Fighter>,
    prev_rounds: Vec<GameRound>,
    next_round: Option<GameRound>,
    pub num_rounds: i32,
    pub season_name: String,
    pre_matches: Vec<(usize, usize)> // indexes into fighters
}

impl GameState {
    // bookkeeping

    pub fn new_game(season_name: &str) -> GameState {
        let season_name = season_name.to_string();
        GameState {
            fighters: Vec::new(),
            prev_rounds: Vec::new(),
            next_round: None,
            num_rounds: 0,
            season_name,
            pre_matches: Vec::new()
        }
    }

    pub fn add_fighter(&mut self, f: Fighter) {
        self.fighters.push(f); // fighter creation is handled in the fighter module
    }

    pub fn arrange_match(&mut self, f1i: usize, f2i: usize) -> Result<(), String> {
        if self.fighters[f1i].dead || self.fighters[f2i].dead {
            return Err(String::from("cannot prematch dead fighters!")) // self explanatory
        }
        if let None = self.next_round {
            return Err(String::from("cannot prematch while a round is scheduled!"))
        }
        self.fighters[f1i].pre_matched = true; // avoid auto matching them later
        self.fighters[f2i].pre_matched = true;
        self.pre_matches.push((f1i, f2i)); // cant add to a round cuz the round doesnt exist
        Ok(())
    }

    const TABLE_SEP: &'static str = " | "; // avoid magic numbers
    pub fn list_fighters(&self) {
        let mut longest_name_len: usize = 4; // length of "name"
        let mut longest_owner_len: usize = 5; // length of "owner"

        for f in &self.fighters { // get size of longest name for padding
            let ln = f.name.len();
            let lo = f.owner.len();
            if ln > longest_name_len {
                longest_name_len = ln
            }
            if lo > longest_owner_len {
                longest_owner_len = lo
            }
        }

        let index_pad_amt = (self.fighters.len() - 1).to_string().len(); // ew
        for _ in 0..(index_pad_amt + 3) { // adds spaces to pad the front of the headings
            print!(" ") // a stupid solution to a stupid problem
        }

        let name_head = "name".pad_to_width(longest_name_len); // pad out headings
        let owner_head = "owner".pad_to_width(longest_owner_len);

        println!("{1}{0}{2}{0} class{0}strength{0}speed{0}skill{0}points{0}total{0}rating{0}kills", Self::TABLE_SEP, name_head, owner_head);
        // class is done lazily cuz it's a discrete thing
        // class as string will never be longer than 6
        // if this fact ever changes FIX THIS

        let mut i = 0; // didnt want to do the with index thing
        for f in &self.fighters {
            let name_pad = f.name.pad_to_width_with_alignment(longest_name_len, Alignment::Right); // pad out names etc
            let owner_pad = f.owner.pad_to_width_with_alignment(longest_owner_len, Alignment::Right);
            let class_pad = format!("{}", f.class).pad_to_width_with_alignment(6, Alignment::Right); // should possibly eliminate a magic number here
            let index_pad = i.to_string().pad_to_width_with_alignment(index_pad_amt, Alignment::Right);

            // name owner class st sp sk us tt rt kl
            print!("[{9}] {1}{0}{2}{0}{3}{0}{4:>8}{0}{5:>5}{0}{6:>5}{0}{11:>6}{0}{10:>5}{0}{7:>6}{0}{8:>5}", 
                Self::TABLE_SEP, name_pad, owner_pad, class_pad, f.strength, f.speed, f.skill, f.rating, f.kills, index_pad, f.total(), f.unspent_points); // the Worst format string
            if f.dead {
                print!("  (dead)")
            }
            println!("");
            i += 1
        }
    }
    fn format_round(&self, r: &GameRound) -> String {
        let mut ret = String::new();
        let round_run = r.log.fights.len() != 0; // check if the round is in the past
        // i coulda done that with a bool but it would fuck up the existing test save

        ret.push_str(&format!("round {}\narena: {}\nmodifier: {}\nmatchups:\n", r.log.round_no, r.arena, r.modifier));
        let mut i = 0;
        for matchup in &r.matchups {
            let (f1, f2) = *matchup;
            let f1name = &self.fighters[f1].name;
            let f2name = &self.fighters[f2].name;
            ret.push_str(&format!("\t{} VS {}\n", f1name, f2name));
            if round_run { // only log results if the round has been run. they don't exist otherwise
                let battle = &r.log.fights[i];
                ret.push_str(&format!("\t\trolls:\n\t\t\t{} VS {}\n", fmt_vec(&battle.rolls_1), fmt_vec(&battle.rolls_2)));
                ret.push_str(&format!("\t\tinjuries:\n\t\t\t{}: {}\n\t\t\t{}: {}\n", f1name, fmt_option(&battle.injury_1), f2name, fmt_option(&battle.injury_2))); // lotsa tabs
                ret.push_str(&format!("\t\tother events:\n{}", fmt_vec_with_tabs(&battle.other_events, 3)))
                // result here
            }
            i += 1;
        }
        match r.sitting_out {
            Some(i) => {
                ret.push_str(&format!("\t{} sits out", self.fighters[i].name))
            }
            None => {}
        }

        ret
    }
    fn log_round_priv(&self, r: &GameRound, po: &ProgramOptions) {
        let filename = &po.global_data.default_batlog_name; // get template
        let filename = filename.replace("%S", &self.season_name); // run replacements
        let filename = filename.replace("%R", &r.log.round_no.to_string());
        let filename = filename.replace(" ", "_"); // not strictly necessary but fuck you
        let filename = match get_non_repeating_filename(".", &filename, "txt") {
            Ok(v) => v,
            Err(v) => v
        };
        let _ = fs::write(filename, self.format_round(r)); // "lol", said the lazy dev. "lmao"
    }
    pub fn display_next_round(&self) {
        match &self.next_round { // only display if a round exists
            Some(r) => println!("{}", self.format_round(&r)),
            None => println!("no round scheduled") // exit without panicking
        }
    }
    pub fn display_round(&self, number: usize) {
        if self.prev_rounds.len() > number {
            println!("{}", self.format_round(&self.prev_rounds[number]))
        }
        else {
            println!("index out of range!") // exit without panicking
        }
    }
    pub fn log_round(&self, number: usize, po: &ProgramOptions) {
        if self.prev_rounds.len() > number {
            self.log_round_priv(&self.prev_rounds[number], po)
        }
        else {
            println!("index out of range!") // exit without panicking
        }
    }

    pub fn load_from_file(filename: &str) -> Result<Self, String> {
        match fs::read_to_string(filename) {
            Ok(s) => {
                match serde_json::from_str::<GameState>(&s) {
                    Ok(g) => Ok(g),
                    Err(e) => Err(format!("json parse error for file {} ({})", filename, e)) // exit without panicking
                }
            }
            Err(_) => Err(format!("file read error for file {}", filename)) // exit without panicking
        }
    }
    pub fn save_to_file(&self, filename: &str) -> Result<(), String> {
        match fs::write(filename, serde_json::to_string(self).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("failed to write file {}", filename))
        }
    }

    // game features

    pub fn new_round(&mut self, po: &ProgramOptions, args: &mut Vec<String>) -> Result<(), String> { // generate round and store
        let (mut matchups, sitting_out) = generate_matchups(&self.fighters);
        matchups.append(&mut self.pre_matches);
        let mut round = GameRound::new(matchups, sitting_out, self.num_rounds + 1);

        if args.len() != 0 { // manual arena/mod choice
            let mut arena: Option<String> = None;
            let mut modifier: Option<String> = None;

            args.insert(0, String::from("new-round")); // argparse needs the name of the program/command as args[0] to work

            //println!("{:?}", args);

            {
                let mut ap = ArgumentParser::new();
                ap.set_description("generates a new round in the current loaded game");
                ap.refer(&mut arena).add_option(&["-a"], StoreOption, "choose an arena manually");
                ap.refer(&mut modifier).add_option(&["-m"], StoreOption, "chose a modifier manually");

                match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
                    Ok(_) => {},
                    Err(_) => return Err(String::from("unknown argument parser error!")) // reports an error on --help. maybe fix?
                }
            }

            match arena {
                Some(a) => {
                    match a.parse::<Arena>() {
                        Ok(v) => round.arena = v,
                        Err(e) => return Err(e)
                    }
                }
                None => {}
            }
            match modifier {
                Some(m) => {
                    match m.parse::<Modifier>() {
                        Ok(v) => round.modifier = v,
                        Err(e) => return Err(e)
                    }
                }
                None => {}
            }
        }

        if po.verbosity > -1 {
            println!("{}", self.format_round(&round))
        }
        self.next_round = Some(round);

        Ok(())
    }
    pub fn cancel_next_round(&mut self, po: &ProgramOptions) {
        if po.verbosity > -1 {
            match self.next_round {
                Some(_) => println!("cancelling round {}", self.num_rounds + 1),
                None => println!("no scheduled round to cancel")
            }
        }
        self.next_round = None
    }

    pub fn run_round(&mut self, po: &ProgramOptions) {
        let r = match &mut self.next_round { // check next round exists
            Some(r) => r,
            None => {
                println!("next round not yet generated!"); 
                return // exit without panicking
            }
        };
        
        for (f1i, f2i) in &r.matchups {
            r.log.advance_to_next_battle(*f1i, *f2i);
            let mut f1 = self.fighters[*f1i].clone(); // cant take 2 mut slices even though they don't overlap
            let mut f2 = self.fighters[*f2i].clone();
            f1.pre_matched = false; // if you leave prematched on they wont get matched again next round
            f2.pre_matched = false;

            battle::battle(&mut f1, &mut f2, &r.arena, &r.modifier, &mut r.log);

            self.fighters[*f1i] = f1; // put back into list
            self.fighters[*f2i] = f2;
        }

        self.prev_rounds.push(r.clone());
        let r = &self.prev_rounds[self.num_rounds as usize]; // probably a better way to do this but the borrow checker gets angry if i use r from earlier
        if po.verbosity > -1 {
            println!("{}", self.format_round(&r))
        }
        if po.logging {
            self.log_round_priv(&r, po)
        }
        self.next_round = None;
        self.num_rounds += 1;
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