use argparse::{ArgumentParser, Store, Collect, IncrBy, DecrBy, StoreTrue, StoreOption};
use std::process::exit;
use std::io::{stdout, stderr};
use std::path::Path;
use std::env::var;
use std::fs::remove_file;

use global::GwGlobalData;
use game::GameState;
use fighter::Fighter;
use utils::confirm;

mod game;
mod fighter;
mod utils;
mod batlog;
mod global;
mod round;
mod battle;
mod boss;

const VERSION: &str = "0.1.0";

fn main() {
    match run() {
        Ok(_) => {
            exit(0)
        }
        Err(e) => {
            exit(e)
        }
    }
}

fn run() -> Result<(), i32> {
    let mut po = utils::ProgramOptions::default(); // fucking awful

    let mut command = String::new();
    let mut args_2: Vec<String> = Vec::new();
    let mut global_path_option: Option<String> = None;

    {
        let mut ap = argparse::ArgumentParser::new();

        ap.set_description("gladiator war in rust! see the readme for usage. it's kinda complicated");

        ap.refer(&mut command).add_argument("command", Store, "the command to execute").required();
        ap.refer(&mut args_2).add_argument("command", Collect, ".");

        ap.refer(&mut po.verbosity).add_option(&["-v"], IncrBy(1), "controls how loud the program is").add_option(&["-q"], DecrBy(1), "quiet");
        ap.refer(&mut po.logging).add_option(&["-l"], StoreTrue, "automatically logs battles to a text file");
        ap.refer(&mut global_path_option).add_option(&["-g"], StoreOption, "path to a global data file");

        ap.stop_on_first_argument(true);

        match ap.parse_args() {
            Ok(_) => {},
            Err(e) => return Err(e)
        }
    }

    let global_path = match global_path_option {
        Some(p) => p,
        None => {
            match var("AJAL_GW_DATA_PATH") { // try to get environment var
                Ok(p) => {
                    if po.verbosity > 0 {println!("got global path from AJAL_GW_DATA_PATH environment variable")}
                    p
                } // if it exists, use it
                Err(_) => "gw_global.json".to_string() // if it doesnt, use current folder default
            }
        }
    };
    let global_data = if !Path::new(&global_path).exists() { // file does NOT exist
        if po.verbosity > -1 {
            println!("could not find global data file, creating a default in current location (consider setting the AJAL_GW_DATA_PATH environment variable)")
        }
        let data = GwGlobalData::default();
        let _ = data.save_to_file(&global_path); // save it here as well as later
        // not checking the write here is fine
        // cuz it's a blank data file
        data
    }
    else { // file exists
        if po.verbosity > 0 {
            println!("using data file at {}", global_path)
        }
        match GwGlobalData::load_from_file(&global_path) {
            Ok(v) => v, // load + parse worked
            Err(e) => { // error
                println!("failed to load global data: {}", e);
                return Err(1)
            }
        }
    };

    po.global_data = global_data;

    //println!("{}", po.verbosity);

    //println!("outer: {}, {:?}", command, args_2);

    match command.as_str() { // ajal-gw-rs [options] command ...
        "load" => {
            if args_2.len() < 2 {
                println!("not enough arguments for load command (expected >=2, found {})", args_2.len());
                return Err(2)                
            }

            let game_index = match args_2[0].parse::<usize>() { // get index of game filename in global data
                Ok(i) => i,
                Err(_) => {
                    println!("game number failed to parse");
                    return Err(2)
                }
            };
            
            if po.global_data.saves.len() <= game_index { // check game exists
                println!("index out of range");
                return Err(2)
            }

            let game_file = &po.global_data.saves[game_index]; // pull out path

            let game = match GameState::load_from_file(game_file) {
                Ok(g) => {
                    if po.verbosity > 0 {
                        println!("loaded game {} from file {}", g.season_name, game_file)
                    }
                    g
                }
                Err(e) => {
                    println!("{}", e);
                    return Err(1)
                }
            };

            let game = match do_things_to_existing_game(args_2, game, &po) {
                Ok(g) => g,
                Err((e, c)) => {
                    println!("error: {}", e);
                    return Err(c)
                }
            };

            match game.save_to_file(game_file) {
                Ok(_) => {}
                Err(e) => {
                    println!("{}", e);
                    return Err(1)
                }
            }
        }

        "help" => {
            print_help(); // unfinished
            return Ok(())
        }

        "list-saves" => { // list save games w/ numbers? exit
            println!("printing save games:");
            for i in 0..po.global_data.saves.len() {
                println!("[{}] {}", i, po.global_data.saves[i])
            }
        }
        "add-save" => { // check validity of save, add, exit
            if args_2.len() != 1 {
                println!("wrong number of arguments for add-save command (expected 1)");
                return Err(2)
            }
            match po.global_data.add_save(&args_2[0]) {
                Ok(_) => {},
                Err(e) => {
                    println!("error adding save: {}", e);
                    return Err(1)
                }
            }
        }
        "delete-save" => {
            if args_2.len() != 1 {
                println!("wrong number of arguments for delete-save command (expected 1)");
                return Err(2)
            }
            let game_index = match args_2[0].parse::<usize>() { // get index of game filename in global data
                Ok(i) => i,
                Err(_) => {
                    println!("game number failed to parse");
                    return Err(2)
                }
            };
            if game_index >= po.global_data.saves.len() {
                println!("index out of range");
                return Err(2)
            }
            let game_path = &po.global_data.saves[game_index];
            if confirm(&format!("are you sure you want to delete the saved game at {}? [y/n]", game_path)) {
                if po.verbosity > 0 { println!("removing file...") }
                let _ = remove_file(game_path);
                if po.verbosity > 0 { println!("removing entry in list...") }
                po.global_data.saves.remove(game_index);
            }
        }

        "new-game" => { // new-game name path
            let alen = args_2.len();
            if alen == 0 {
                println!("not enough arguments for new-game command (expected >=1, found {})", alen);
                return Err(2)
            }
            let game = GameState::new_game(&args_2[0]);
            let filename = match alen {
                1 => {
                    let mut fname = args_2[0].replace(' ', "_");
                    fname.push_str(".json");
                    match utils::get_non_repeating_filename(&fname) {
                        Ok(n) => n,
                        Err(e) => {
                            println!("{}", e);
                            return Err(1)
                        }
                    }
                }
                2..=usize::MAX => {
                    match utils::get_non_repeating_filename(&args_2[1]) {
                        Ok(n) => n,
                        Err(e) => {
                            println!("{}", e);
                            return Err(1)
                        }
                    }
                }
                _ => {
                    unreachable!()
                }
            };
            match game.save_to_file(&filename) {
                Ok(_) => {
                    let path = Path::new(&filename);
                    let path = path.canonicalize().unwrap(); // this shouldnt fail because the file definitely exists
                    let filename = path.to_str().unwrap(); // also shouldnt fail unless the user summons demons with their file system
                    po.global_data.saves.push(filename.to_string())
                }
                Err(e) => {
                    println!("{}", e);
                    return Err(1)
                }
            }
        }

        _ => {
            println!("unrecognised command {}", command);
            return Err(1)
        }
    }

    match po.global_data.save_to_file(&global_path) {
        Ok(_) => Ok(()),
        Err(e) => {
            println!("{}", e);
            Err(1)
        }
    }
}

#[allow(unused_variables)]
fn do_things_to_existing_game(args: Vec<String>, mut game: GameState, po: &utils::ProgramOptions) -> Result<GameState, (String, i32)> {
    let mut command = String::new();
    let mut args_2: Vec<String> = Vec::new();

    //println!("{:?}", args);

    {
        let mut ap = ArgumentParser::new();

        ap.refer(&mut command).add_argument("command", Store, "the command to execute").required();
        ap.refer(&mut args_2).add_argument("command", Collect, ".");

        ap.stop_on_first_argument(true);

        match ap.parse(args, &mut stdout(), &mut stderr()) {
            Ok(_) => {},
            Err(e) => return Err((format!(""), e))
        }
    }

    //println!("inner: {}, {:?}", command, args_2);

    match command.as_str() {
        //bookkeeping
        "info" => {
            println!("{}", game.season_name);
            println!("number of rounds: {}", game.num_rounds)
        }
        "add-fighter" => { // take args, parse into vec, parse into numbers, add as fighter, exit
            match Fighter::from_vec(&args_2) {
                Ok(f) => game.add_fighter(f),
                Err(e) => return Err((e, 1))
            }
        }
        "list-fighters" => {
            game.list_fighters();
        }
        "next-round" => {
            game.display_next_round()
        }
        "show-round" => {
            if args_2.len() != 1 {
                return Err((String::from("round number required"), 2))
            }
            let ri = match args_2[0].parse::<usize>() {
                Ok(v) => v,
                Err(_) => return Err((String::from("round number failed to parse"), 2))
            };
            game.display_round(ri)
        }
        "log-round" => {
            if args_2.len() == 0 {
                return Err((String::from("round number required"), 2))
            }
            let mut path = None;
            if args_2.len() > 1 {
                path = Some(args_2[1].as_str())
            }
            //dbg!(path);
            let ri = match args_2[0].parse::<usize>() { // round index
                Ok(v) => v,
                Err(_) => return Err((String::from("round number failed to parse"), 2))
            };
            game.log_round(ri, path, po)
        }
        "add-stats" => { // add-stats fi st sp sk
            let a2l = args_2.len();
            if a2l != 4 {
                return Err((format!("expected 1 index and 3 stats, found {} arguments", a2l), 2))
            }

            let fi = match args_2[0].parse::<usize>() { // fighter index
                Ok(v) => v,
                Err(_) => {
                    return Err((format!("{} does not parse to usize", args_2[0]), 2))
                }
            };

            args_2.remove(0);

            let f = &mut game.fighters[fi];

            let mut stats_v: Vec<i32> = Vec::new();
            for stat in args_2 {
                match stat.parse::<i32>() {
                    Ok(v) => stats_v.push(v),
                    Err(_) => {
                        return Err((format!("{} does not parse to int!", stat), 2))
                    }
                }
            }

            f.strength += stats_v[0];
            f.speed += stats_v[1];
            f.skill += stats_v[2];
            f.unspent_points -= stats_v.iter().sum::<i32>();

            println!("adding stats {}, {}, {} to fighter {}", stats_v[0], stats_v[1], stats_v[2], f.name)
        }
        "arrange-match" => { // arrange-match f1i f2i
            if args_2.len() != 2 {
                return Err((String::from("2 args required"), 2))
            }
            let f1i = match args_2[0].parse::<usize>() { // fighter index
                Ok(v) => v,
                Err(_) => {
                    return Err((format!("{} does not parse to usize", args_2[0]), 2))
                }
            };
            let f2i = match args_2[1].parse::<usize>() { // fighter index
                Ok(v) => v,
                Err(_) => {
                    return Err((format!("{} does not parse to usize", args_2[1]), 2))
                }
            };
            match game.arrange_match(f1i, f2i) {
                Ok(_) => {}
                Err(e) => return Err((e, 1))
            }
        }

        // running
        "run-round" => { // run match, store results in log, possibly give hr text log file, exit
            game.run_round(po)
        }
        "new-round" => { // new argparser for arena and mod??
            match game.new_round(po, &mut args_2) {
                Ok(()) => {},
                Err(e) => return Err((e, 2))
            }
        }
        "cancel-round" => {
            game.cancel_next_round(po)
        }

        _ => {
            return Err((format!("unrecognised command {}", command), 1))
        }
    }

    Ok(game)
}

fn print_help() { // FIX THIS
    println!("welcome to version {} of the gladiator war CLI", VERSION);
    println!("please see the readme for usage instructions. it's kinda complicated")
}