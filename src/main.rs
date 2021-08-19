mod utils;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::{error::Error, fs::{File, OpenOptions}, io::{BufRead, BufReader, ErrorKind, stdout}, sync::mpsc};
use tui::{Terminal, backend::CrosstermBackend};

use clap::{AppSettings, Clap, ArgEnum};

use crate::utils::{app::App, user_interface::Event};
use std::io::Write;
use lazy_static::lazy_static;
use regex::Regex;

#[derive(ArgEnum, Debug, PartialEq)]
enum SaveMode {
    /// Do not save any information. Default behavior.
    NONE,
    /// Only save the last state before exiting with Q.
    LAST,
    /// Record all rolled values, but not the internal state.
    ROLLS,
    /// Save both rolled values and every state update.
    FULL
}

/// This is a simple CLI dice roller. It optionally allows you to specify a seed for a fun repeatable experience.
#[derive(Clap)]
#[clap(version = "1.0", author = "Gib <gibryon@protonmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// A string (word, sentence, etc) to initialize the random generator.
    #[clap(short, long)]
    seed: String,
    /// Whether or not to save the RNG internals for continuity.
    #[clap(short = 'm', long, arg_enum, default_value = "none")]
    save_mode: SaveMode,
    #[clap(short, long)]
    /// Whether or not to load an existing save file that matches the name of the seed.
    load_state_file: bool
}

fn rebuild_rng(f: File) -> Option<Pcg64> {
    let reader = BufReader::new(f);
    let lines = reader.lines();
    let mut peek = lines.peekable();
    let mut tup: Option<(u128, u128)> = None;
   
    lazy_static! {
        static ref RE: Regex = Regex::new(r"\d+ - \((\d+), (\d+)\)").unwrap();
    }

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
        Some(Pcg64::reinit(tup.0, tup.1))
    } else {
        None
    }
}

fn get_rng(load_state_file: bool, seed: &str) -> Pcg64 {
    if load_state_file {
        let save_file = get_save_file_from_seed(seed);
        let file = File::open(&save_file);
        
        match file {
            Ok(f) => {
                let rng = rebuild_rng(f);
    
                if let Some(rng) = rng {
                    rng
                } else {
                    panic!("Failed to reinitialize the RNG. Please remove {} if it is corrupted.", save_file);
                }
            },
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    eprintln!("Could not find a state file to load, creating a new RNG from the seed.");
                    Seeder::from(seed).make_rng()
                } else {
                    panic!("Unknown error {}", e);
                }
            },
        }
    } else {
        Seeder::from(seed).make_rng()
    }
}

fn get_save_file_from_seed(seed: &str) -> String { format!("{}.seed.state", seed) }

fn main() -> Result<(), Box<dyn Error>> {
    let opt = Opts::parse();
    
    let mut rng: Pcg64 = get_rng(opt.load_state_file, &opt.seed);

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
                KeyCode::Enter => {
                    let val = app.roll_die(&mut rng);
                    if opt.save_mode == SaveMode::NONE {
                        continue;
                    }

                    if let Some(val) = val {
                        let should_not_overwrite = opt.save_mode != SaveMode::LAST;
                        // TODO: This is currently inefficient. File handle should be hoisted to the top of the app, and we should figure out how to handle different save modes
                        // (i.e. different file names for save modes, different storage syntax, etc.).
                        let mut file = OpenOptions::new().append(should_not_overwrite).write(true).create(true).open(get_save_file_from_seed(&opt.seed))?;

                        match opt.save_mode {
                            SaveMode::NONE => (),
                            SaveMode::LAST => write!(file, "{} - {:?}\n", val, rng.dump())?,
                            SaveMode::ROLLS => write!(file, "{}\n", val)?,
                            SaveMode::FULL => write!(file, "{} - {:?}\n", val, rng.dump())?,
                        }
                    }
                },
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
