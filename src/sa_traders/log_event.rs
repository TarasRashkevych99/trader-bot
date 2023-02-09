use unitn_market_2022::good::good_kind::GoodKind;
use serde::Serialize;
use serde::Deserialize;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CustomEventKind {
    Bought, Sold, LockedBuy, LockedSell, Wait
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomEvent {
    pub kind: CustomEventKind,
    pub good_kind: GoodKind,
    pub quantity: f32,
    pub price: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LogEvent {
    pub time: u32,
    pub event: CustomEvent,
    pub market: String,
    pub result: bool,
    pub error: Option<String>
}

pub fn craft_log_event(time: u32, kind: CustomEventKind, good_kind: GoodKind, quantity: f32, price: f32, market: String, result: bool, error: Option<String>) -> LogEvent {
    let custom_ev = CustomEvent {
        kind,
        good_kind,
        quantity,
        price,
    };
    LogEvent {
        market,
        result,
        error,
        time,
        event: custom_ev,
    }
}