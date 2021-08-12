mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::{borrow::Borrow, convert::TryInto, error::Error, fmt::Display, io::{Stdout, stdout}, sync::mpsc, thread, time::{Duration, Instant}};
use tui::{Frame, Terminal, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, Borders, List, ListItem}};

use clap::{AppSettings, Clap, ArgEnum};
use strum::{EnumIter, EnumString, IntoEnumIterator};
use utils::StatefulList;

enum Event<I> {
    Input(I),
    Tick,
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

// Can this be more generic?
fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
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
    
    // I think this is -2 because of the margin.
    let height = chunks[1].height - 2;
    let list = if event_items.len() > 0 {
        let can_fit_x_items: u16 = (height as  usize / event_items[0].height()).try_into().unwrap();

        if can_fit_x_items < event_items.len() as u16 {
            let sliding_frame_index: usize = event_items.len() - can_fit_x_items as usize;
            List::new(event_items.get(sliding_frame_index..).unwrap()).block(block)
        } else {
            List::new(event_items).block(block)
        }
    } else {
        List::new(event_items).block(block)
    };
    
    f.render_widget(list, chunks[1]);
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

fn roll_die(app: &mut App, rng: &mut Pcg64) {
    if let Some(i) = app.items.state.selected() {
        // First run, 10 d100s:                 100, 71, 74, 29, 32, 30, 11, 5, 95, 56
        // Second run, 3 d100s, 4d12s, 3d100s:  100, 71, 74,  4,  4,  4,  2, 5, 95, 56
        // Third run, 10 d12s:                   12,  9,  9,  4,  4,  4,  2, 1, 12,  7
        let r: u16 = rng.gen_range(1..=get_upper_bound_of_dice(Dice::from_str(app.items.items[i].borrow(), true).unwrap()));
        app.events.push(r);
    }
}


fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opts::parse();

    let mut rng: Pcg64 = Seeder::from(opt.seed).make_rng();
    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);

    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    let tick_rate = Duration::from_millis(250);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            // poll for tick rate duration, if no events, sent tick event.
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));
            if event::poll(timeout).unwrap() {
                if let CEvent::Key(key) = event::read().unwrap() {
                    tx.send(Event::Input(key)).unwrap();
                }
            }
            if last_tick.elapsed() >= tick_rate {
                tx.send(Event::Tick).unwrap();
                last_tick = Instant::now();
            }
        }
    });

    let mut app = App::new();
    terminal.clear()?;

    loop {
        terminal.draw(|f| draw(f, &mut app))?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    execute!(
                        terminal.backend_mut(),
                        LeaveAlternateScreen,
                        DisableMouseCapture
                    )?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Enter => roll_die(&mut app, &mut rng),
                KeyCode::Left => app.items.previous(),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                KeyCode::Right => app.items.next(),
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
