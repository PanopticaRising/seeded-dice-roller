// This is a simple application that copies directly from the TUI-rs examples.

use std::borrow::Borrow;
use std::fmt::Display;

use clap::ArgEnum;
use clap::{AppSettings, Clap};
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use strum::{EnumIter, EnumString, IntoEnumIterator};
use termion::event::{self, Key};
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::widgets::{Block, Borders, List, ListItem};
// use youchoose;
use crate::utils::{
    event::{Event, Events},
    StatefulList,
};
use std::io::{self, Error};
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

mod utils;

#[derive(ArgEnum, Debug, PartialEq, EnumString, EnumIter)]
enum Dice {
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

/// This doc string acts as a help message when the user runs '--help'
/// as do all doc strings on fields
#[derive(Clap)]
#[clap(version = "1.0", author = "Gibryon Bhojraj <gibryon@protonmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(short, long)]
    seed: String,
}

fn get_upper_bound_of_dice(d: Dice) -> u16 {
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

struct App {
    items: StatefulList<String>,
    events: Vec<u16>,
}
impl App {
    fn new() -> App {
        App {
            items: StatefulList::with_items(Dice::iter().map(|d| d.to_string()).collect()),
            events: vec!(),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opt = Opts::parse();

    let mut rng: Pcg64 = Seeder::from(opt.seed).make_rng();

    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let events = Events::new();

    terminal.clear()?;

    // Initialize selector
    app.items.next();

    loop {
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(2)
                .constraints([Constraint::Percentage(20), Constraint::Percentage(50)].as_ref())
                .split(f.size());

            let items: Vec<tui::widgets::ListItem> =
                Dice::iter().map(|d| ListItem::new(d.to_string())).collect();

            let block = Block::default().title("Block").borders(Borders::ALL);
            let list = List::new(items)
                .block(block)
                .highlight_style(
                    Style::default()
                        .bg(Color::LightGreen)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");
            f.render_stateful_widget(list, chunks[0], &mut app.items.state);
            // f.render_widget(block, chunks[0]);
            let block = Block::default().title("Block 2").borders(Borders::ALL);
            let event_items: Vec<tui::widgets::ListItem> = app.events.iter().map(|i| {
                ListItem::new(i.to_string())
            }).collect();
            let list = List::new(event_items).block(block);
            f.render_widget(list, chunks[1]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    terminal.clear()?;
                    break;
                }
                Key::Right => {
                    if let Some(i) = app.items.state.selected() {
                        // First run, 10 d100s:                 100, 71, 74, 29, 32, 30, 11, 5, 95, 56
                        // Second run, 3 d100s, 4d12s, 3d100s:  100, 71, 74,  4,  4,  4,  2, 5, 95, 56
                        // Third run, 10 d12s:                   12,  9,  9,  4,  4,  4,  2, 1, 12,  7
                        let r: u16 = rng.gen_range(1..=get_upper_bound_of_dice(Dice::from_str(app.items.items[i].borrow(), true).unwrap()));
                        app.events.push(r);
                    }
                }
                Key::Down => {
                    app.items.next();
                }
                Key::Up => {
                    app.items.previous();
                }
                _ => {}
            },
            Event::Tick => {
                // app.advance();
            }
        }
    }


    Ok(())
}
