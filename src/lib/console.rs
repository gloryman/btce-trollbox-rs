extern crate term;

use std::sync::mpsc::Sender;
use std::sync::mpsc::Receiver;
use super::regex::Regex;
use std::hash::{Hash, SipHasher, Hasher};

use super::{TicksList, TickType, ChatMessage, };

static COLORS : &'static [term::color::Color] = &[
    term::color::RED,
    term::color::BRIGHT_RED,
    term::color::GREEN,
    term::color::BRIGHT_GREEN,
    term::color::YELLOW,
    term::color::BRIGHT_YELLOW,
    term::color::BLUE,
    term::color::BRIGHT_BLUE,
    term::color::MAGENTA,
    term::color::BRIGHT_MAGENTA,
    term::color::CYAN,
    term::color::BRIGHT_CYAN,
];

pub trait Console: Send {
    fn console(&self);
}

/*
volume              : 263.959  0.0126786
                    : 263.959  0.0126786
                    : 263.959  0.0126786
                    : 263.959  0.0126786
                    : 263.959  0.0126786
                    : 263.959  0.0126786
*/
impl Console for TicksList {
    fn console(&self) {
        let mut tm = match term::stdout() {
            Some(val) => val,
            None => {
                println!("Unable to open Terminal");
                return;
            }
        };

        // Display prefix
        tm.fg(term::color::BRIGHT_BLACK).unwrap();
        print!("volume alert        : ");

        let ln = self.len() - 1;
        for (i, rec) in self.iter().rev().enumerate() {
            let _ = match rec.typ {
                TickType::BID => tm.fg(term::color::BRIGHT_GREEN),
                TickType::ASK => tm.fg(term::color::BRIGHT_RED),
            };
            print!("{0:8.4}  ", rec.price);
            tm.fg(term::color::BRIGHT_WHITE).unwrap();
            println!("{0:<12.6}", rec.amount);

            if i <  ln {
                tm.fg(term::color::BRIGHT_BLACK).unwrap();
                print!("{0:<20}: ", "");
            }
        }
    }
}

impl Console for ChatMessage {
    fn console(&self) {
        let re = Regex::new(r"^([[:alpha:]|\d]+),\s").unwrap();
        let mut tm = match term::stdout() {
            Some(val) => val,
            None => {
                println!("Unable to open Terminal");
                return;
            }
        };

        let rec = self.clone();
        // Display Channel
        tm.fg(term::color::BRIGHT_BLACK).unwrap();
        print!("[{}] ", rec.channel.unwrap_or(String::new()));
        // Display Login
        tm.fg(hash_color(&rec.login)).unwrap();
        print!("{0:<15}", rec.login);

        // Colorefy user name at start of line
        tm.fg(term::color::BRIGHT_BLACK).unwrap();
        print!(": ");
        match re.find(&rec.msg) {
            Some(_) => {
                let tmp: Vec<&str> = rec.msg.splitn(2, ",")
                    .filter(|st| st.len() > 0)
                    .collect();
                let (login, rest) = (tmp[0], tmp[1]);

                tm.fg(hash_color(&login)).unwrap();
                print!("{}", login);

                tm.fg(term::color::WHITE).unwrap();
                println!(", {}",rest);
            },
            None => {
                tm.fg(term::color::WHITE).unwrap();
                println!("{}", rec.msg);
            }
        };
    }
}

pub type BtcePipeSender = Sender<Box<Console>>;
pub type BtcePipeReceiver = Receiver<Box<Console>>;

pub fn hash_color(st: &str) -> term::color::Color {
    let mut hasher = SipHasher::new();
    st.hash(&mut hasher);
    let hashsum = hasher.finish();

    let idx: usize = hashsum as usize % COLORS.len();
    COLORS[idx]
}
