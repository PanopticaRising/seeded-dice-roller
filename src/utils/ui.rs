
use crossterm::{event::{self, Event as CEvent, KeyEvent}};

use std::{convert::TryInto, io::{Stdout}, sync::mpsc::Sender, thread, time::{Duration, Instant}};
use tui::{Frame, backend::CrosstermBackend, layout::{Constraint, Direction, Layout}, style::{Color, Modifier, Style}, widgets::{Block, Borders, List, ListItem}};

use strum::IntoEnumIterator;

use super::{app::App, dice::Dice};

pub enum Event<I> {
    Input(I),
    Tick,
}

// Can this be more generic?
pub fn draw(f: &mut Frame<CrosstermBackend<Stdout>>, app: &mut App) {
    let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .margin(2)
    .constraints([Constraint::Percentage(20), Constraint::Percentage(50)].as_ref())
    .split(f.size());

    let items: Vec<tui::widgets::ListItem> =
        Dice::iter().map(|d| ListItem::new(d.to_string().to_ascii_lowercase())).collect();

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

pub fn initialize_ui_thread(tx: Sender<Event<KeyEvent>>) {
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
}