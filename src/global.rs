use serde::{Serialize, Deserialize};
use std::fs;

use crate::game;

#[derive(Serialize, Deserialize, Debug)]
pub struct GwGlobalData {
    pub saves: Vec<String>,
    pub default_batlog_name: String
}
impl Default for GwGlobalData {
    fn default() -> Self {
        GwGlobalData {
            default_batlog_name: String::from("%S_batlog_%R.txt"),
            saves: Vec::new()
        }
    }
}

impl GwGlobalData {
    #[allow(dead_code)] // shut up
    pub fn add_save(&mut self, filename: &str) -> Result<(), String> {
        match game::GameState::load_from_file(filename) {
            Ok(_) => {
                self.saves.push(String::from(filename));
                Ok(())
            }
            Err(e) => Err(e)
        }
    }

    pub fn load_from_file(filename: &str) -> Result<Self, String> {
        match fs::read_to_string(filename) {
            Ok(s) => {
                //println!("{}", s);
                match serde_json::from_str(&s) {
                    Ok(v) => Ok(v),
                    Err(e) => {
                        Err(format!("json parse failed: {}", e))
                    }
                }
            }
            Err(_) => {
                Err(format!("file read failed"))
            }
        }
    }

    pub fn save_to_file(&self, filename: &str) -> Result<(), String> {
        match fs::write(filename, serde_json::to_string_pretty(self).unwrap()) {
            Ok(_) => Ok(()),
            Err(_) => Err(format!("failed to write file {}", filename))
        }
    }
}
