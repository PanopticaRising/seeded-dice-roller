use rand::Rng;
use rand_pcg::Pcg64;
use std::fmt::Display;
use clap::ArgEnum;
use strum::{EnumIter, EnumString};

#[derive(ArgEnum, Debug, PartialEq, EnumString, EnumIter)]
pub enum Dice {
    d3,
    d4,
    d6,
    d8,
    d10,
    d12,
    d20,
    d100,
}

impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Dice {
    fn get_upper_bound(&self) -> u16 {
        Dice::get_upper_bound_of_dice(self)
    }

    fn get_upper_bound_of_dice(d: &Dice) -> u16 {
        match d {
            Dice::d3 => 3_u16,
            Dice::d4 => 4_u16,
            Dice::d6 => 6_u16,
            Dice::d8 => 8_u16,
            Dice::d10 => 10_u16,
            Dice::d12 => 12_u16,
            Dice::d20 => 20_u16,
            Dice::d100 => 100_u16,
        }
    }

    pub fn roll_die(rng: &mut Pcg64, dice_string: &str) -> u16 {
        rng.gen_range(1..=Dice::from_str(dice_string, true).unwrap().get_upper_bound())
    }
}

// fn roll_die(app: &mut App, rng: &mut Pcg64) {
//     if let Some(i) = app.items.state.selected() {
//         // First run, 10 d100s:                 100, 71, 74, 29, 32, 30, 11, 5, 95, 56
//         // Second run, 3 d100s, 4d12s, 3d100s:  100, 71, 74,  4,  4,  4,  2, 5, 95, 56
//         // Third run, 10 d12s:                   12,  9,  9,  4,  4,  4,  2, 1, 12,  7
//         let r: u16 = rng.gen_range(1..=Dice::from_str(app.items.items[i].borrow(), true).unwrap().get_upper_bound());
//         app.events.push(r);
//     }
// }

