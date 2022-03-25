use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BossRound {
    ver: i32,
    matchups: Vec<Vec<usize>> // indexes into fighter list
}

impl BossRound {
    
}