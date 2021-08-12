mod utils;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand_pcg::Pcg64;
use rand_seeder::Seeder;
use std::{error::Error, io::{stdout}, sync::mpsc};
use tui::{Terminal, backend::CrosstermBackend};

use clap::{AppSettings, Clap};

use crate::utils::{app::App, ui::Event};


/// This is a simple CLI dice roller. It optionally allows you to specify a seed for a fun repeatable experience.
#[derive(Clap)]
#[clap(version = "1.0", author = "Gib <gibryon@protonmail.com>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    /// A string (word, sentence, etc) to initialize the random generator.
    #[clap(short, long)]
    seed: String,
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

    utils::ui::initialize_ui_thread(tx);

    let mut app = App::new();
    terminal.clear()?;

    loop {
        terminal.draw(|f| utils::ui::draw(f, &mut app))?;

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
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}
