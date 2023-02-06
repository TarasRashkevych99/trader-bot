use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;


type ChosenMarket = Rc<RefCell<dyn Market>>;

pub struct Trader {
    name: String,
    wallet: HashMap<GoodKind, f32>,
}

impl Trader {
    pub fn new(name: String) -> Self {
        Trader {
            name,
            wallet: HashMap::from_iter(vec![
                (GoodKind::EUR, 100.0),
                (GoodKind::USD, 100.0),
                (GoodKind::YUAN, 100.0),
                (GoodKind::YEN, 100.0),
            ]),
        }
    }

    pub fn print_wallet_per_kind(&self) {
        println!("Wallet of {}", self.name);
        for (kind, qty) in self.wallet.iter() {
            println!("{}: {}", kind, qty);
        }
    }

    pub fn print_wallet_in_euro(&self) {
        println!("Wallet of {} in euro", self.name);
        println!("Amount: {}", self.get_all_money_in_euro());
    }

    pub fn get_all_money_in_euro(&self) -> f32 {
        self.wallet.iter()
            .map(|(kind, _)|
                self.get_money_by_kind(kind.clone()) / self.get_default_exchange_rates().get(kind).unwrap())
            .sum()
    }

    pub fn get_money_by_kind(&self, kind: GoodKind) -> f32 {
        *self.wallet.get(&kind).unwrap()
    }

    pub fn get_default_exchange_rates(&self) -> HashMap<GoodKind, f32> {
        vec![
            (GoodKind::USD, DEFAULT_EUR_USD_EXCHANGE_RATE),
            (GoodKind::EUR, 1.0),
            (GoodKind::YEN, DEFAULT_EUR_YEN_EXCHANGE_RATE),
            (GoodKind::YUAN, DEFAULT_EUR_YUAN_EXCHANGE_RATE),
        ].into_iter().collect()
    }

    pub fn trade_with_one_market(&mut self, market: &mut ChosenMarket) {
        loop {
            let lock = market
                .borrow_mut()
                .lock_sell(GoodKind::USD, 10.0, 9.0, self.name.clone());
            match lock {
                Ok(token) => {
                    println!("Locked successfully");
                    let purchase = market
                        .borrow_mut()
                        .sell(token, &mut Good::new(GoodKind::USD, 10.0));
                    match purchase {
                        Ok(good) => {
                            println!("Purchase successful {}", good.get_qty());
                            self.wallet.insert(GoodKind::USD, self.get_money_by_kind(GoodKind::USD) - 10.0);
                            self.wallet.insert(GoodKind::EUR, self.get_money_by_kind(GoodKind::EUR) + 9.0);
                        }
                        Err(e) => {
                            println!("Purchase failed");
                        }
                    }
                }
                Err(e) => {
                    println!("Transaction failed");
                    break;
                }
            }
        }
    }

    pub fn trade_with_all_markets(&mut self, bfb: &mut ChosenMarket, rcnz: &mut ChosenMarket, zse: &mut ChosenMarket) {
        loop {
            // trader logic
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_get_money_by_kind() {
        let mut trader = Trader::new("RAST".to_string());
        assert_eq!(trader.get_money_by_kind(GoodKind::EUR), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::USD), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::YUAN), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::YEN), 100.0);
    }

    #[test]
    fn test_get_default_exchange_rates() {
        let mut trader = Trader::new("RAST".to_string());
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::EUR).unwrap(), &1.0);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::USD).unwrap(), &DEFAULT_EUR_USD_EXCHANGE_RATE);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::YUAN).unwrap(), &DEFAULT_EUR_YUAN_EXCHANGE_RATE);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::YEN).unwrap(), &DEFAULT_EUR_YEN_EXCHANGE_RATE);
    }

    #[test]
    fn test_get_all_money_in_euro() {
        let mut trader = Trader::new("RAST".to_string());
        let mut total_amount = trader.get_money_by_kind(GoodKind::EUR) / 1.0;
        total_amount += trader.get_money_by_kind(GoodKind::USD) / DEFAULT_EUR_USD_EXCHANGE_RATE;
        total_amount += trader.get_money_by_kind(GoodKind::YUAN) / DEFAULT_EUR_YUAN_EXCHANGE_RATE;
        total_amount += trader.get_money_by_kind(GoodKind::YEN) / DEFAULT_EUR_YEN_EXCHANGE_RATE;
        assert!(f32::abs(trader.get_all_money_in_euro() - total_amount) < 0.0001);
    }
}