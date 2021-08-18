use rand::Rng;
use rand_pcg::Pcg64;
use std::fmt::Display;
use clap::ArgEnum;
use strum::{EnumIter, EnumString};

#[derive(ArgEnum, Debug, PartialEq, EnumString, EnumIter)]
pub enum Dice {
    D3,
    D4,
    D6,
    D8,
    D10,
    D12,
    D20,
    D100,
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
            Dice::D3 => 3_u16,
            Dice::D4 => 4_u16,
            Dice::D6 => 6_u16,
            Dice::D8 => 8_u16,
            Dice::D10 => 10_u16,
            Dice::D12 => 12_u16,
            Dice::D20 => 20_u16,
            Dice::D100 => 100_u16,
        }
    }

    pub fn roll_die(rng: &mut Pcg64, dice_string: &str) -> u16 {
        rng.gen_range(1..=Dice::from_str(dice_string, true).unwrap().get_upper_bound())
    }
}
