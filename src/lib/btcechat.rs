extern crate websocket;

use std::thread::sleep_ms;
use websocket::stream::WebSocketStream;
use websocket::client::request::Url;
use websocket::client;

pub use websocket::{Client, DataFrame, Message, Sender, Receiver};
pub use websocket::result::WebSocketError;

static BTCE_CHAT_URL: &'static str = "wss://ws.pusherapp.com/app/4e0ebd7a8b66fa3554a4?protocol=6&client=js&version=2.0.0&flash=false";
//static BTCE_CHAT_SUBSCRIBE: &'static str = "{\"event\":\"pusher:subscribe\",\"data\":{\"channel\": \"{}\"}}";

pub type WsSender = client::sender::Sender<WebSocketStream>;
pub type WsReceiver = client::receiver::Receiver<WebSocketStream>;
pub type WsClient = Client<DataFrame, WsSender, WsReceiver>;

/// Build complite JSON string for BTC-E subscribe message
/// Wrap set of concatinations in to function call
fn build_subscription_message(channel: String) -> Message {
    let mut result: String =
            "{\"event\":\"pusher:subscribe\",\"data\":{\"channel\": \""
            .to_owned();
    let end = "\"}}";

    result.push_str(&channel);
    result.push_str(end);
    Message::Text(result)
}

#[derive(Clone, Debug, RustcDecodable, RustcEncodable)]
pub struct ChatMessage {
    pub uid     : String,
    pub login   : String,
    pub msg     : String,
    pub date    : String,
    pub usr_clr : String,
    pub channel : Option<String>,
}

/*
impl ChatMessage {
    fn new() -> ChatMessage{
        let tmp = ChatMessage {
            uid    : String::new(),
            login  : String::new(),
            msg    : String::new(),
            date   : String::new(),
            usr_clr: String::new(),
            channel: Some(String::new()),
        };
        tmp
    }
}
*/

/// Wraps BTC-E chat websocket init, subscribe procedure.
/// State:
///     channel: BTC-E channel (room) subscription name
///     failures: invalid connections counter
pub struct BtceChatTransport {
    channel: String,
    failures: u32,
}

/// Simple constructor
/// ch: subscribe channel name
impl BtceChatTransport {
    pub fn new(ch: &str) -> BtceChatTransport {
        BtceChatTransport {channel: ch.to_string(), failures: 0}
    }
}

/// Create new Websocket connection with topic (channel) subscription
/// on demand.
impl Iterator for BtceChatTransport {
    type Item = WsClient;

    fn next(&mut self) -> Option<WsClient> {

        let url = Url::parse(BTCE_CHAT_URL).unwrap();
        loop {
            // TODO: Add failures limit
            if self.failures > 0 {
                sleep_ms(2000);
            };

            let request = match Client::connect(url.clone()) {
                Ok(request) => request,
                Err(_) => {
                    self.failures += 1;
                    continue
                },
            };

            let response = match request.send() {
                Ok(response) => response,
                Err(_) => {
                    self.failures += 1;
                    continue;
                },
            };

            let mut ws = response.begin();
            let msg = build_subscription_message(self.channel.clone());
            match ws.send_message(msg) {
                Ok(_) => (),
                Err(_) => {
                    self.failures += 1;
                    continue;
                },
            };

            self.failures = 0;
            return Some(ws)
        }
    }
}
