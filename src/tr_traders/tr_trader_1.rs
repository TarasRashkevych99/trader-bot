use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::Market;
use unitn_market_2022::wait_one_day;
use rand::thread_rng;
use rand::Rng;


type ChosenMarket = Rc<RefCell<dyn Market>>;
type MarketName = String;
type Action = String;
type Wallet = HashMap<GoodKind, f32>;
type Transactions = Vec<(i32, MarketName, Action, Good)>;

pub struct Trader {
    name: String,
    wallet: Wallet,
    register: Register,
}

struct Register {
    transactions: Transactions,
    day: i32,
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
            register: Register {
                transactions: Vec::new(),
                day: 0,
            },
        }
    }

    pub fn trade_with_all_markets(&mut self, bfb: &mut ChosenMarket, rcnz: &mut ChosenMarket, zse: &mut ChosenMarket) {
        loop {
            if self.register.day == 10000 {
                break;
            }
            let (good_kind, market_to_buy_from, market_to_sell_to) = self.get_max_profit_pair_by_exchange_rate(Rc::clone(bfb), Rc::clone(rcnz), Rc::clone(zse), self.get_priority());

            let available_quantity_to_buy = self.get_market_good_quantity_by_kind(&market_to_buy_from, &good_kind);

            if available_quantity_to_buy > 0.0 && available_quantity_to_buy > 1.0e-3 {
                let quantity_to_buy = available_quantity_to_buy * 2.0 / 3.0;
                let bid = market_to_buy_from.borrow().get_buy_price(good_kind.clone(), quantity_to_buy).unwrap();
                let lock_for_buying = market_to_buy_from.borrow_mut().lock_buy(good_kind.clone(), quantity_to_buy, bid, self.name.clone());
                match lock_for_buying {
                    Ok(token) => {
                        println!("Locked successfully");
                        let purchase = market_to_buy_from.borrow_mut().buy(token, &mut Good::new(GoodKind::EUR, bid));
                        match purchase {
                            Ok(good) => {
                                println!("Purchase successful {}", good.get_qty());
                                self.update_internal_state_after_buying(&market_to_buy_from, bid, good, "sold".to_string());
                            }
                            Err(e) => {
                                println!("Purchase failed");
                            }
                        }
                    }
                    Err(e) => {
                        println!("Transaction failed");
                    }
                }

                let available_quantity_to_pay_with = self.get_market_good_quantity_by_kind(&market_to_sell_to, &GoodKind::EUR);

                if available_quantity_to_pay_with > 0.0 && available_quantity_to_pay_with > 1.0e-3 {
                    let quantity_to_pay_with = available_quantity_to_pay_with * 2.0 / 3.0;
                    let mut offer = market_to_sell_to.borrow().get_sell_price(good_kind.clone(), quantity_to_buy).unwrap();
                    if offer > quantity_to_pay_with {
                        offer = quantity_to_pay_with;
                    }
                    let lock_for_selling = market_to_sell_to.borrow_mut().lock_sell(good_kind.clone(), quantity_to_buy, offer, self.name.clone());
                    match lock_for_selling {
                        Ok(token) => {
                            println!("Locked successfully");
                            let sale = market_to_sell_to.borrow_mut().sell(token, &mut Good::new(good_kind.clone(), quantity_to_buy));
                            match sale {
                                Ok(good) => {
                                    println!("Sale successful {}", good.get_qty());
                                    self.update_internal_state_after_selling(&market_to_sell_to, offer, Good::new(good_kind, quantity_to_buy), "bought".to_string());
                                }
                                Err(e) => {
                                    println!("Sale failed");
                                }
                            }
                        }
                        Err(e) => {
                            println!("Transaction failed");
                        }
                    }
                }
            } else {
                // println!("Waiting for a day");
                wait_one_day!(bfb, rcnz, zse);
            }
        }
    }

    fn get_max_profit_pair_by_exchange_rate(&self, bfb: ChosenMarket, rcnz: ChosenMarket, zse: ChosenMarket, priority: usize) -> (GoodKind, ChosenMarket, ChosenMarket) {
        let mut candidates_to_trade_with = Vec::new();
        for (kind, _) in self.wallet.iter() {
            let mut bfb_good = self.get_market_good_label_by_kind(&bfb, &kind);
            let mut rcnz_good = self.get_market_good_label_by_kind(&rcnz, &kind);
            let mut zse_good = self.get_market_good_label_by_kind(&zse, &kind);
            let min_buy_price = bfb_good.exchange_rate_buy.min(rcnz_good.exchange_rate_buy.min(zse_good.exchange_rate_buy));
            let max_sell_price = bfb_good.exchange_rate_sell.max(rcnz_good.exchange_rate_sell.max(zse_good.exchange_rate_sell));
            let buy_sell_diff = max_sell_price - min_buy_price;

            let market_to_buy_from = if bfb_good.exchange_rate_buy == min_buy_price {
                Rc::clone(&bfb)
            } else if rcnz_good.exchange_rate_buy == min_buy_price {
                Rc::clone(&rcnz)
            } else {
                Rc::clone(&zse)
            };
            let market_to_sell_to = if bfb_good.exchange_rate_sell == max_sell_price {
                Rc::clone(&bfb)
            } else if rcnz_good.exchange_rate_sell == max_sell_price {
                Rc::clone(&rcnz)
            } else {
                Rc::clone(&zse)
            };
            candidates_to_trade_with.push((buy_sell_diff, kind, market_to_buy_from, market_to_sell_to));
        }
        candidates_to_trade_with.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
        let (_, good, from, to) = &candidates_to_trade_with[priority];
        (*good.clone(), Rc::clone(from), Rc::clone(to))
    }

    fn update_internal_state_after_buying(&mut self, market: &ChosenMarket, amount: f32, good: Good, action: Action) {
        self.register.day += 1;
        self.register.transactions.push((self.register.day, self.get_market_name(&market), action, good.clone()));
        self.wallet.insert(good.get_kind().clone(), self.get_money_by_kind(good.get_kind().clone()) + good.get_qty());
        self.wallet.insert(GoodKind::EUR, self.get_money_by_kind(GoodKind::EUR) - amount);
    }

    fn update_internal_state_after_selling(&mut self, market: &ChosenMarket, amount: f32, good: Good, action: Action) {
        self.register.day += 1;
        self.register.transactions.push((self.register.day, self.get_market_name(&market), action, good.clone()));
        self.wallet.insert(good.get_kind().clone(), self.get_money_by_kind(good.get_kind().clone()) - good.get_qty());
        self.wallet.insert(GoodKind::EUR, self.get_money_by_kind(GoodKind::EUR) + amount);
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

    pub fn print_register(&self) {
        println!("Register of {}", self.name);
        for (day, market, action, good) in self.register.transactions.iter() {
            println!("Day {}: {} {} {}", day, market, action, good);
        }
    }

    pub fn get_all_money_in_euro(&self) -> f32 {
        self.wallet
            .iter()
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

    fn get_priority(&self) -> usize {
        let mut rng = thread_rng();
        rng.gen_range(0..3)
    }

    fn get_market_good_quantity_by_kind(&self, market: &ChosenMarket, good_kind: &GoodKind) -> f32 {
        self.get_market_good_label_by_kind(market, good_kind).quantity
    }

    fn get_market_good_buy_exchange_rate_by_kind(&self, market: &ChosenMarket, good_kind: &GoodKind) -> f32 {
        self.get_market_good_label_by_kind(market, good_kind).exchange_rate_buy
    }

    fn get_market_good_sell_exchange_rate_by_kind(&self, market: &ChosenMarket, good_kind: &GoodKind) -> f32 {
        self.get_market_good_label_by_kind(market, good_kind).exchange_rate_sell
    }

    fn get_market_good_label_by_kind(&self, market: &ChosenMarket, good_kind: &GoodKind) -> GoodLabel {
        market.borrow().get_goods().iter().find(|(good)| good.good_kind == *good_kind).unwrap().to_owned()
    }

    fn get_market_name(&self, market: &ChosenMarket) -> String {
        market.borrow().get_name().to_string()
    }
}

#[cfg(test)]
mod test {
    use crate::common::markets::new_with_quantities;
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

    // #[test]
    // fn test_get_max_profit_pair_by_exchange_rate() {
    //     let mut trader = Trader::new("RAST".to_string());
    //     let (mut bfb, mut rcnz, mut zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);
    //     let (good_to_trade, market_to_buy_from, market_to_sell_to) = trader.get_max_profit_pair_by_exchange_rate(bfb.clone(), rcnz.clone(), zse.clone());
    //     assert_eq!(good_to_trade, GoodKind::YEN);
    //     assert_eq!(market_to_buy_from.borrow().get_name(), "Baku stock exchange");
    //     assert_eq!(market_to_sell_to.borrow().get_name(), "ZSE");
    // }
}
