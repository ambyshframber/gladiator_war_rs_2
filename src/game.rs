use serde::{Serialize, Deserialize};
use std::fs;
use pad::{PadStr, Alignment};
use argparse::{ArgumentParser, StoreOption};
use std::io::{stdout, stderr};

use super::fighter::*;
use super::round::{GameRound, Arena, Modifier, Round};
use super::battle::{battle, BattleResult};
use super::utils::{ProgramOptions, fmt_vec, fmt_option, get_non_repeating_filename, fmt_vec_with_tabs};

#[derive(Serialize, Deserialize, Debug)]
pub struct GameState {
    pub fighters: Vec<Fighter>,
    prev_rounds: Vec<Round>,
    next_round: Option<Round>,
    pub num_rounds: i32,
    pub season_name: String,
    pre_matches: Vec<(usize, usize)>, // indexes into fighters
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

        
        let index_pad_amt = if self.fighters.len() > 0 {
            (self.fighters.len() - 1).to_string().len() // ew
        }
        else {
            0usize
        };
        for _ in 0..(index_pad_amt + 3) { // adds spaces to pad the front of the headings
            print!(" ") // a stupid solution to a stupid problem
        }

        let name_head = "name".pad_to_width(longest_name_len); // pad out headings
        let owner_head = "owner".pad_to_width(longest_owner_len);

        let headings = format!("{1}{0}{2}{0}class   {0}strength{0}speed{0}skill{0}points{0}total{0}rating{0}kills", Self::TABLE_SEP, name_head, owner_head);
        // class is done lazily cuz it's a discrete thing
        // class as string will never be longer than 8
        // if this fact ever changes FIX THIS
        println!("{}", headings);

        for (i, f) in self.fighters.iter().enumerate() {
            let name_pad = f.name.pad_to_width_with_alignment(longest_name_len, Alignment::Right); // pad out names etc
            let owner_pad = f.owner.pad_to_width_with_alignment(longest_owner_len, Alignment::Right);
            let class_pad = format!("{}", f.class).pad_to_width_with_alignment(8, Alignment::Right); // should possibly eliminate a magic number here
            let index_pad = i.to_string().pad_to_width_with_alignment(index_pad_amt, Alignment::Right);

            // name owner class st sp sk us tt rt kl
            print!("[{9}] {1}{0}{2}{0}{3}{0}{4:>8}{0}{5:>5}{0}{6:>5}{0}{11:>6}{0}{10:>5}{0}{7:>6}{0}{8:>5}", 
                Self::TABLE_SEP, name_pad, owner_pad, class_pad, f.strength, f.speed, f.skill, f.rating, f.kills, index_pad, f.total(), f.unspent_points); // the Worst format string
            if f.dead {
                print!("  (dead)")
            }
            println!("");
        }
    }
    fn format_round(&self, round: &Round) -> String {
        let mut ret = String::new();

        match round {
            Round::Standard(r) => {
                let round_run = r.log.fights.len() != 0; // check if the round is in the past
                // i coulda done that with a bool but it would fuck up the existing test save (i am Very Lazy)

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
                        ret.push_str(&format!("\t\tother events:\n{}", fmt_vec_with_tabs(&battle.other_events, 3)));

                        type Res = BattleResult; // arguably makes code easier to read i guess???
                        match battle.result {
                            Res::F1Win | Res::F1WinFromCleric => {
                                ret.push_str(&format!("\t\twinner: {}", f1name))
                            }
                            Res::F2Win | Res::F2WinFromCleric => {
                                ret.push_str(&format!("\t\twinner: {}", f2name))
                            }
                            Res::Draw | Res::DrawFromCleric => {
                                ret.push_str(&format!("\t\tdraw!"))
                            }
                        }
                    }
                    i += 1;
                }
                match r.sitting_out {
                    Some(i) => {
                        ret.push_str(&format!("\t{} sits out", self.fighters[i].name))
                    }
                    None => {}
                }
            }
            Round::Boss(_) => unreachable!() // not implemented
        }

        ret
    }
    fn log_round_priv(&self, r: &Round, po: &ProgramOptions, path: Option<&str>) {
        let filename = match path {
            None => &po.global_data.default_batlog_name, // get template
            Some(p) => p
        };
        let filename = filename.replace("%S", &self.season_name); // run replacements
        let round_num = match r {
            Round::Standard(v) => v.log.round_no,
            Round::Boss(_) => unreachable!()
        };
        let filename = filename.replace("%R", &round_num.to_string());
        let filename = filename.replace(" ", "_"); // not strictly necessary but fuck you
        //filename.push_str(".txt");
        let filename = match get_non_repeating_filename(&filename) {
            Ok(v) => v,
            Err(v) => v
        };
        match fs::write(&filename, self.format_round(r)) {
            Ok(_) => {},
            Err(_) => {
                println!("failed to write file {}!", filename)
            }
        }
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
    pub fn log_round(&self, number: usize, path: Option<&str>, po: &ProgramOptions) {
        if self.prev_rounds.len() > number {
            self.log_round_priv(&self.prev_rounds[number], po, path)
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
        match fs::write(filename, serde_json::to_string_pretty(self).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("failed to write file {}", filename))
        }
    }

    // game features

    pub fn new_round(&mut self, po: &ProgramOptions, args: &mut Vec<String>) -> Result<(), String> { // generate round and store
        //let (mut matchups, sitting_out) = generate_matchups(&self.fighters);
        //matchups.append(&mut self.pre_matches);
        //let mut round = GameRound::new(&self.fighters, &mut self.pre_matches, self.num_rounds + 1); // FIX THIS

        let mut arena: Option<String> = None;
        let mut modifier: Option<String> = None;
        args.insert(0, String::from("new-round")); // argparse needs the name of the program/command as args[0] to work
        
        {
            let mut ap = ArgumentParser::new();
            ap.set_description("generates a new round in the current loaded game");
            ap.refer(&mut arena).add_option(&["-a"], StoreOption, "choose an arena manually");
            ap.refer(&mut modifier).add_option(&["-m"], StoreOption, "chose a modifier manually");
            match ap.parse(args.clone(), &mut stdout(), &mut stderr()) {
                Ok(_) => {},
                Err(e) => match e {
                    0 => {}
                    _ => return Err(String::from("unknown argument parser error!"))
                }
            }
        }
        let arena_parsed = match arena {
            Some(a) => {
                match a.parse::<Arena>() {
                    Ok(v) => Some(v),
                    Err(e) => return Err(e)
                }
            }
            None => None
        };
        let modifier_parsed = match modifier {
            Some(m) => {
                match m.parse::<Modifier>() {
                    Ok(v) => Some(v),
                    Err(e) => return Err(e)
                }
            }
            None => None
        };

        let round = GameRound::new(&self.fighters, &mut self.pre_matches, self.num_rounds + 1, arena_parsed, modifier_parsed);

        let r = Round::Standard(round);

        if po.verbosity > -1 {
            println!("{}", self.format_round(&r))
        }
        self.next_round = Some(r);

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
        let round = match &mut self.next_round { // check next round exists
            Some(r) => r,
            None => {
                println!("next round not yet generated!"); 
                return // exit without panicking
            }
        };

        match round {
            Round::Standard(r) => {
                for (f1i, f2i) in &r.matchups {
                    r.log.advance_to_next_battle(*f1i, *f2i);
                    let mut f1 = self.fighters[*f1i].clone(); // cant take 2 mut slices even though they don't overlap
                    let mut f2 = self.fighters[*f2i].clone();
                    f1.pre_matched = false; // if you leave prematched on they wont get matched again next round
                    f2.pre_matched = false;
        
                    battle(&mut f1, &mut f2, &r.arena, &r.modifier, &mut r.log);
        
                    self.fighters[*f1i] = f1; // put back into list
                    self.fighters[*f2i] = f2;
                }
        
                self.prev_rounds.push(round.clone());
                let r = &self.prev_rounds[self.num_rounds as usize]; // probably a better way to do this but the borrow checker gets angry if i use r from earlier
                if po.verbosity > -1 {
                    println!("{}", self.format_round(&r))
                }
                if po.logging {
                    self.log_round_priv(&r, po, None)
                }
            }
            Round::Boss(_) => unreachable!()
        }
        self.next_round = None;
        self.num_rounds += 1;
    }
}
