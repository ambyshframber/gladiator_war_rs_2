use serde::{Serialize, Deserialize};

use super::battle::BattleResult;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Batlog { // nananananananana bat log (battle log)
    pub fights: Vec<Battle>,
    count: i32,
    pub round_no: i32,
}

// needs to have methods to add events (injuries, battles etc)
// needs to be serializable and then easily interpreted by js

impl Batlog {
    pub fn new(round_no: i32) -> Self {
        Batlog {
            fights: Vec::new(),
            count: 0,
            round_no,
        }
    }

    pub fn advance_to_next_battle(&mut self, f1: usize, f2: usize) {
        self.fights.push(Battle::new(f1, f2))
    }

    pub fn set_rolls(&mut self, rolls: Vec<i32>) { // i dont know why i did it this way but i did
        let i = self.fights.len() - 1;
        if self.count == 0 {
            self.fights[i].rolls_1 = rolls;
            self.count = 1
        }
        else {
            self.fights[i].rolls_2 = rolls;
            self.count = 0
        }
    }
    pub fn set_injury(&mut self, injury: Option<i32>) {
        let i = self.fights.len() - 1;
        if self.count == 0 {
            self.fights[i].injury_1 = injury;
            self.count = 1
        }
        else {
            self.fights[i].injury_2 = injury;
            self.count = 0
        }
    }
    pub fn set_points(&mut self, points: i32) {
        let i = self.fights.len() - 1;
        self.fights[i].points = points
    }
    pub fn set_result(&mut self, r: BattleResult) {
        let i = self.fights.len() - 1;
        self.fights[i].result = r
    }
    pub fn add_events(&mut self, event: String) {
        let i = self.fights.len() - 1;
        let e_log = &mut self.fights[i].other_events;
        e_log.push(event)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct Battle {
    pub fighter_1: usize, // index into fighter list
    pub rolls_1: Vec<i32>,
    pub injury_1: Option<i32>, // winner gets None (usually)

    pub fighter_2: usize, // index into fighter list
    pub rolls_2: Vec<i32>,
    pub injury_2: Option<i32>,

    pub points: i32,
    pub result: BattleResult,

    pub other_events: Vec<String>
}

impl Battle {
    fn new(fighter_1: usize, fighter_2: usize) -> Self {
        Battle {
            fighter_1, fighter_2,
            ..Battle::default()
        }
    }
}