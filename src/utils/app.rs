use rand_pcg::Pcg64;
use strum::IntoEnumIterator;

use super::stateful_list::StatefulList;
use super::dice::Dice;

// items and events have to be public to display to the UI.
pub struct App {
    pub items: StatefulList<String>,
    pub events: Vec<u16>,
}
impl App {
    pub fn new() -> App {
        App {
            items: StatefulList::with_items(Dice::iter().map(|d| d.to_string()).collect()),
            events: vec!(),
        }
    }

    pub fn roll_die(&mut self, rng: &mut Pcg64) {
        if let Some(i) = self.items.state.selected() {
            let val = Dice::roll_die(rng, self.items.items.get(i).unwrap());
            self.events.push(val);
        }
    }
}

