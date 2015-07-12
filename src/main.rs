extern crate websocket;
extern crate rustc_serialize;
extern crate term;
extern crate hyper;
extern crate docopt;

use std::thread;
use std::thread::sleep_ms;
use std::sync::mpsc;
use std::sync::{Mutex, Arc};
use std::io::Read;
use hyper::Client;
use hyper::header::Connection;
use rustc_serialize::json::{Json, decode};
use docopt::Docopt;

use lib::{BtceChatTransport, Message,
    ChatMessage,
    Tick, TicksList,
    Console, BtcePipeSender, BtcePipeReceiver,
    WsSender,
    Sender, Receiver};

mod lib;

// - Static --------------------------------------------------------------------
static USAGE: &'static str = "
Usage:
    wstest [-v <limit> | --volume=<limit>] [-c <arguments>... | --channels=<arguments>...] [-h | --help]

Options:
    -h --help                  Show this screen.
    -v --volume=<limit>        Show tick price with volume > limit [default: 10.0].
                               0 == Disable.
    -c --channels=<arguments>  Listen channels [default: chat_en chat_ru chat_ch].
";

static TICK_FETCH_URL        : &'static str  = "https://btc-e.com/api/3/trades/btc_usd?limit=";
static TICK_FETCH_ELEMENTS   : u64  = 2000;
static TICK_FETCH_INIT_DELAY : u32  = 2000;
static TICK_FETCH_DELAY_DELTA: u32  = 500;
static TICK_FETCH_DELAY_LIMIT: u32  = 30000;



// - Usage Struct --------------------------------------------------------------
#[derive(RustcDecodable, Debug)]
struct Args {
    flag_channels: Vec<String>,
    flag_volume : f64,
}

// -----------------------------------------------------------------------------

fn deserealise_msg(js: &str) -> Option<ChatMessage> {
    let jsobj = match Json::from_str(js) {
        Ok(jsobj) => jsobj,
        _ => return None
    };
    let obj = match jsobj.as_object() {
        Some(obj) => obj,
        _ => return None
    };

    let channel = &obj.get("channel")
        .unwrap_or(&Json::String(String::new()))
        .to_string()
        .replace("\"", "")
        .replace("chat_", "");
    let channel = Some(channel.to_string());

    let data = obj.get("data");
    let jsobj = match Json::from_str(
        &data.unwrap_or(&Json::String(String::new()))
        .to_string()) {
            Ok(jsobj) => jsobj,
            _ => return None
        };

    let data = &jsobj.as_string();
    let jsobj = match Json::from_str(data.unwrap_or("{}")) {
        Ok(jsobj) => jsobj,
        _ => return None
    };

    let data = &jsobj.as_string();
    let mut msg: ChatMessage = match decode(data.unwrap_or("{}")) {
        Ok(msg) => msg,
        Err(_) => return None,
    };

    msg.channel = channel;
    Some(msg)
}

fn pereodic_ping(sndr: Arc<Mutex<WsSender>>) {
    loop {
        match sndr.lock() {
            Ok(mut v) => {
                match v.send_message(Message::Ping(vec![])) {
                    Err(e) => {
                        println!("Websocket Error: {}", e);
                        let _ = v.send_message(Message::Close(None));
                        return;
                    },
                    _ => (),
                };
            },
            Err(e) => {
                println!("Websocket Error: {}", e);
                return;
            },
        };
        sleep_ms(120_000);
    }
}

fn listner(ch: &str, tx: BtcePipeSender) {
    for ws in BtceChatTransport::new(ch) {
        let (sender, mut receiver) = ws.split();
        let sender = Arc::new(Mutex::new(sender));
        let tmp = sender.clone();

        thread::spawn(move || pereodic_ping(tmp));

        for inres in  receiver.incoming_messages::<Message>() {
            let msg = match inres {
                Ok(msg) => msg,
                Err(_) => break,
            };

            match msg {
                Message::Text(data) => {
                    let bmsg = match deserealise_msg(&data) {
                        Some(val) => val,
                        None => continue,
                    };
                    tx.send(Box::new(bmsg)).is_ok();
                },
                Message::Ping(data) => {
                    match sender.lock() {
                        Ok(mut sender) => {
                            match sender.send_message(Message::Pong(data)) {
                                Err(_) => {
                                    let _ = sender.send_message(Message::Close(None));
                                    break;
                                },
                                _ => (),
                            };
                        },
                        Err(_) => break,
                    }
                },
                Message::Close(_) => break,
                _ => continue,
            }
        }
    }
}


// - Ticks ---------------------------------------------------------------------
fn fetch_ticks(flimit: u64, vlimit: f64,
                last_tid: u64, http_client: &Client) -> TicksList {

    let default = vec![];
    let url = &format!("{}{}", TICK_FETCH_URL, flimit);
    let mut reply = match http_client.get(url)
                    .header(Connection::close())
                    .send() {
                        Ok(val) => val,
                        Err(_) => return default,
    };

    let mut body = String::new();
    match reply.read_to_string(&mut body) {
        Err(_) => return default,
        _ => (),
    };

    let res = match Json::from_str(&body) {
        Err(_) => return default,
        Ok(val) => val,
    };
    let res = match res.as_object() {
        None => return default,
        Some(val) => val,
    };
    let res = res.get("btc_usd").unwrap().as_array().unwrap();
    let res: TicksList = res.iter()
                            .map(|x| Tick::from_json(x))
                            .filter(|x| x.is_some())
                            .map(|x| x.unwrap())
                            .filter(|x| x.tid > last_tid && x.amount > vlimit)
                            .collect();
    return res
}

fn get_new_delay(delay: u32) -> u32 {
    let delay = delay + TICK_FETCH_DELAY_DELTA;
    if delay > TICK_FETCH_DELAY_LIMIT {
        return TICK_FETCH_DELAY_LIMIT
    }
    return delay
}

fn volume_monitor(vlimit: f64, tx: BtcePipeSender) {
    let http_client = &Client::new();
    let mut last_tid =
        match fetch_ticks(1, 0.0001, 0, http_client)
                .iter()
                .next() {
        Some(x) => x.tid,
        None => 0,
    };
    let mut delay = TICK_FETCH_INIT_DELAY;

    loop {
        sleep_ms(delay);
        let res = fetch_ticks(TICK_FETCH_ELEMENTS, vlimit,
                                    last_tid, http_client);

        match res.first() {
            Some(tic) => {
                if tic.tid == last_tid {
                    delay = get_new_delay(delay);
                    continue;
                }
                last_tid = tic.tid;
            },
            None => {
                delay = get_new_delay(delay);
                continue;
            },
        }

        delay = (delay / res.len() as u32) / 100 * 100;
        if delay < TICK_FETCH_INIT_DELAY {
            delay = TICK_FETCH_INIT_DELAY
        }
        tx.send(Box::new(res.clone())).is_ok();
    }
}

// - M A I N -------------------------------------------------------------------
fn main_loop(btce_chs: Vec<String>, volume_limit: f64) {
    let (tx, rx): (BtcePipeSender, BtcePipeReceiver) = mpsc::channel();

    // Spawn Websocket listners
    for ch in btce_chs {
        let c_tx = tx.clone();
        thread::spawn(move || {
            listner(&ch, c_tx)
        });
    }

    // Spawn Volume monitor
    let _ = thread::Builder::new()
                            .name("volume_monitor".to_string())
                            .spawn(move || {
                                volume_monitor(volume_limit, tx.clone())
                            });

    // Display results
    loop {
        let rec = match rx.recv() {
            Ok(val) => val,
            Err(_) => continue,
        };
        rec.console();
    }
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.decode())
                            .unwrap_or_else(|e| e.exit());

    main_loop(args.flag_channels, args.flag_volume);
}
