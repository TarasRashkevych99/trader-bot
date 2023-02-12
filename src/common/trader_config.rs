use std::fs::File;
use std::io::Read;
use toml::Value;

pub fn get_trader_config() -> TraderConfig {
    let mut config_file = File::open("trader.config.toml").expect("No trader config file found");
    let mut configuration_string = String::new();
    config_file.read_to_string(&mut configuration_string).expect("Error reading config file");
    let config = configuration_string.parse::<Value>().unwrap();

    TraderConfig::new(
        config["trading_days"].as_integer().unwrap() as u32,
        config["budget"].as_float().unwrap() as f32,
        config["delay_in_milliseconds"].as_integer().unwrap() as u64,
        config["trader_TR"].as_bool().unwrap(),
        config["trader_SA"].as_bool().unwrap(),
        config["trader_AB"].as_bool().unwrap(),
    )
}

pub struct TraderConfig {
    trading_days: u32,
    budget: f32,
    delay_in_milliseconds: u64,
    trader_TR: bool,
    trader_SA: bool,
    trader_AB: bool,
}

impl TraderConfig {
    pub fn new(trading_days: u32, budget: f32, delay_in_milliseconds: u64, trader_TR: bool, trader_SA: bool, trader_AB: bool) -> Self {
        TraderConfig {
            trading_days,
            budget,
            delay_in_milliseconds,
            trader_TR,
            trader_SA,
            trader_AB,
        }
    }

    pub fn get_trading_days(&self) -> u32 {
        self.trading_days
    }
    pub fn get_budget(&self) -> f32 {
        self.budget
    }
    pub fn get_delay_in_milliseconds(&self) -> u64 {
        self.delay_in_milliseconds
    }
    pub fn is_trader_TR(&self) -> bool {
        self.trader_TR
    }
    pub fn is_trader_SA(&self) -> bool {
        self.trader_SA
    }
    pub fn is_trader_AB(&self) -> bool {
        self.trader_AB
    }
}