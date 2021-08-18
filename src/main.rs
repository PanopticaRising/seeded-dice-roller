mod utils;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::{error::Error, fs::{File, OpenOptions}, io::{BufRead, BufReader, stdout}, sync::mpsc};
use tui::{Terminal, backend::CrosstermBackend};

use clap::{AppSettings, Clap};

use crate::utils::{app::App, user_interface::Event};
use std::io::Write;
use lazy_static::lazy_static;
use regex::Regex;



/// This is a simple CLI dice roller. It optionally allows you to specify a seed for a fun repeatable experience.
#[derive(Clap)]
#[clap(version = "1.0", author = "Gib <gibryon@protonmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// A string (word, sentence, etc) to initialize the random generator.
    #[clap(short, long)]
    seed: String,
    // TODO: Add an option that lets people opt in/out of dumping the state.
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opts::parse();
    let save_file = format!("{}.seed.state", opt.seed);
    
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\((\d+), (\d+)\)").unwrap();
    }

    let file = File::open(&save_file);
    
    let mut rng: Pcg64 = if let Ok(f) = file {
        let reader = BufReader::new(f);
        let lines = reader.lines();
        let mut peek = lines.peekable();
        let mut tup: Option<(u128, u128)> = None;

        while let Some(line) = peek.next() {
            if peek.peek().is_none() {
                let line = line.unwrap();
                let capa = RE.captures_iter(&line).next();
                if let Some(cap) = capa {
                    let state: u128 = u128::from_str_radix(&cap[1], 10).unwrap();
                    let incr: u128 =  u128::from_str_radix(&cap[2], 10).unwrap();
                    tup = Some((state, incr));
                }
            }
        };

        if let Some(tup) = tup {
            Pcg64::reinit(tup.0, tup.1)
        } else {
            Seeder::from(opt.seed).make_rng()
        }
    } else {
        Seeder::from(opt.seed).make_rng()
    };

    enable_raw_mode()?;

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup input handling
    let (tx, rx) = mpsc::channel();

    utils::user_interface::initialize_ui_thread(tx);

    let mut app = App::new();
    terminal.clear()?;

    loop {
        terminal.draw(|f| utils::user_interface::draw(f, &mut app))?;

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
                KeyCode::Enter => app.roll_die(&mut rng),
                KeyCode::Left => app.items.previous(),
                KeyCode::Down => app.items.next(),
                KeyCode::Up => app.items.previous(),
                KeyCode::Right => app.items.next(),
                KeyCode::Char('s') => {
                    let mut file = OpenOptions::new().append(true).write(true).create(true).open(&save_file)?;
                    write!(file, "{:?}\n", rng.dump())?;
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
