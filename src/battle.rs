use std::cmp::{Ord, Ordering};
use serde::{Serialize, Deserialize};
use rand::{thread_rng, Rng};

use super::fighter::{Fighter, Class};
use super::round::{Arena, Modifier};
use super::batlog::Batlog;

pub fn battle(f1: &mut Fighter, f2: &mut Fighter, arena: &Arena, modifier: &Modifier, log: &mut Batlog) {
    let f1_stats = f1.roll_for_stats(arena, modifier);
    let f2_stats = f2.roll_for_stats(arena, modifier);

    f1.battles_fought += 1;
    f2.battles_fought += 1;

    let mut points = 0;
    for i in 0..3 {
        let (pts, event) = points_from_stats(&f1.class, f1_stats[i], &f2.class, f2_stats[i], arena, modifier, i);
        points += pts;
        if let Some(e) = event {
            log.add_events(e)
        }
    }
    log.set_points(points);
    log.set_rolls(f1_stats); // this works
    log.set_rolls(f2_stats); // trust me
    let mut result = get_result(points, &f1.class, &f2.class);
    
    if let Modifier::OhShitSheHasAGun = modifier {
        let mut f1_shot = false;
        let mut f2_shot = false;
        if thread_rng().gen_range(0..10) == 0 {
            f1_shot = true
        }
        if thread_rng().gen_range(0..10) == 0 {
            f2_shot = true
        }
        let events = if f1_shot {
            if f2_shot { // both
                result = BattleResult::Draw;
                String::from("both fighters got shot")
            }
            else { // only f1
                result = BattleResult::F2Win;
                format!("{} got shot", f1.name)
            }
        }
        else if f2_shot {
            result = BattleResult::F1Win;
            format!("{} got shot", f2.name)
        }
        else {
            String::new()
        };
        if f1_shot || f2_shot {
            log.add_events(events)
        }
    }

    match result {
        BattleResult::F1Win | BattleResult::F1WinFromCleric => {
            f1.injure(arena, modifier, log, false);
            let inj = f2.injure(arena, modifier, log, true);
            let rdiff = f2.rating - f1.rating; // how much bigger is f2s rating

            if rdiff > 3 { // double stat ups and rating
                f1.rating += 1;
                f2.rating -= 1;
                f1.unspent_points += 1
            }
            else if rdiff < -3 {
                f1.rating -= 1;
                f2.rating += 1;
            }
            f1.rating += 1;
            f1.unspent_points += 1;
            f1.battles_won += 1;
            f2.rating -= 1;

            if inj.unwrap() < 1 && modifier == &Modifier::TheCrowdDemandsBlood {
                f1.unspent_points += 1;
            }
        }
        BattleResult::F2Win | BattleResult::F2WinFromCleric => {
            let inj = f1.injure(arena, modifier, log, true);
            f2.injure(arena, modifier, log, false);
            let rdiff = f1.rating - f2.rating; // how much bigger is f1s rating
            
            if rdiff > 3 {
                f1.rating -= 1;
                f2.rating += 1;
                f2.unspent_points += 1
            }
            else if rdiff < -3 {
                f1.rating += 1;
                f2.rating -= 1;
            }
            f1.rating -= 1;
            f2.rating += 1;
            f2.unspent_points += 1;
            f2.battles_won += 1;

            if inj.unwrap() < 1 && modifier == &Modifier::TheCrowdDemandsBlood {
                f2.unspent_points += 1;
            }
        }
        BattleResult::Draw | BattleResult::DrawFromCleric => {
            match arena {
                Arena::CrocPit => {
                    f1.injure(arena, modifier, log, true);
                    f2.injure(arena, modifier, log, true);
                }
                _ => {
                    f1.injure(arena, modifier, log, false);
                    f2.injure(arena, modifier, log, false);
                }
            }
        }
    }
    log.set_result(result);
}

#[allow(unused_variables)]
fn points_from_stats(c1: &Class, stat_1: i32, c2: &Class, stat_2: i32, arena: &Arena, modifier: &Modifier, stat: usize) -> (i32, Option<String>) {
    let mut pts = 0;
    // positive points are f1
    // negative points are f2
    let mut ret = None; // for insta win etc logging

    let diff = stat_1 - stat_2;

    if diff == 0 {
        return (0, None);
    }

    if diff > 0 {
        pts += 1;
    }
    else if diff < 0 {
        pts -= 1;
    }

    if diff >= 5 { // f1 dominates
        pts += 1;
        if stat == 0 && arena == &Arena::ClimbingWall {
            pts += 999999;
            ret = Some(String::from("f1 wins instantly"))
        }
        if c1 == &Class::Dom {
            pts += 1;
        }
        if c2 == &Class::Turtle {
            pts -=1 ;
        }
    }

    if diff <= -5 { // f2 dominates
        pts -= 1;
        if stat == 0 && arena == &Arena::ClimbingWall {
            pts -= 999999;
            ret = Some(String::from("f2 wins instantly"))
        }
        if c2 == &Class::Dom {
            pts -= 1;
        }
        if c1 == &Class::Turtle {
            pts +=1 ;
        }
    }

    (pts, ret)
}

pub fn get_result(points: i32, c1: &Class, c2: &Class) -> BattleResult {
    match points.cmp(&0) {
        Ordering::Less => {
            BattleResult::F2Win
        }
        Ordering::Greater => {
            BattleResult::F1Win
        }
        Ordering::Equal => {
            if c1 == &Class::Cleric {
                if c2 == &Class::Cleric {
                    BattleResult::DrawFromCleric
                } 
                else {
                    BattleResult::F1WinFromCleric
                }
            }
            else if c2 == &Class::Cleric {
                BattleResult::F2WinFromCleric
            } 
            else {
                BattleResult::Draw
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum BattleResult {
    F1Win,
    F2Win,
    F1WinFromCleric,
    F2WinFromCleric,
    Draw,
    DrawFromCleric
}

impl Default for BattleResult {
    fn default() -> Self {
        Self::Draw
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_pts_from_stats() {
        assert_eq!(points_from_stats(&Class::Naked, 1, &Class::Naked, 0, &Arena::Ampitheater, &Modifier::Rulebook, 0), (1, None));
        assert_eq!(points_from_stats(&Class::Naked, 0, &Class::Naked, 1, &Arena::Ampitheater, &Modifier::Rulebook, 0), (-1, None));
        assert_eq!(points_from_stats(&Class::Naked, 5, &Class::Naked, 0, &Arena::Ampitheater, &Modifier::Rulebook, 0), (2, None));
        assert_eq!(points_from_stats(&Class::Naked, 1, &Class::Naked, 7, &Arena::Ampitheater, &Modifier::Rulebook, 0), (-2, None));
        assert_eq!(points_from_stats(&Class::Dom, 5, &Class::Naked, 0, &Arena::Ampitheater, &Modifier::Rulebook, 0), (3, None));
        assert_eq!(points_from_stats(&Class::Dom, 0, &Class::Dom, 5, &Arena::Ampitheater, &Modifier::Rulebook, 0), (-3, None));
        assert_eq!(points_from_stats(&Class::Naked, 5, &Class::Turtle, 0, &Arena::Ampitheater, &Modifier::Rulebook, 0), (1, None));
        assert_eq!(points_from_stats(&Class::Turtle, 0, &Class::Naked, 5, &Arena::Ampitheater, &Modifier::Rulebook, 0), (-1, None));
        assert_eq!(points_from_stats(&Class::Naked, 5, &Class::Naked, 0, &Arena::ClimbingWall, &Modifier::Rulebook, 0), (1000001, Some(String::from("f1 wins instantly"))));
        assert_eq!(points_from_stats(&Class::Naked, 0, &Class::Naked, 5, &Arena::ClimbingWall, &Modifier::Rulebook, 0), (-1000001, Some(String::from("f2 wins instantly"))));
    }
}
