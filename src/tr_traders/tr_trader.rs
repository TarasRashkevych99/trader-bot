use std::cell::RefCell;
use std::collections::HashMap;
use std::mem::take;
use std::rc::Rc;
use unitn_market_2022::good::consts::{DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::Market;
use unitn_market_2022::wait_one_day;
use rand::thread_rng;
use rand::Rng;
use futures::executor::block_on;
use tokio::runtime::Runtime;
use crate::common::visualizer::{craft_log_event, CustomEvent, CustomEventKind, TraderGood};


type ChosenMarket = Rc<RefCell<dyn Market>>;
type MarketName = String;
type Action = String;
type Wallet = HashMap<GoodKind, f32>;
type Transactions = Vec<(u32, MarketName, Action, Good)>;

pub struct Trader_TR {
    name: String,
    wallet: Wallet,
    register: Register,
    initial_budget_in_euro: f32,
}

struct Register {
    transactions: Transactions,
    day: u32,
}

impl Trader_TR {
    pub fn new(name: String, budget_per_good: f32) -> Self {
        Trader_TR {
            name,
            wallet: HashMap::from_iter(vec![
                (GoodKind::EUR, budget_per_good),
                (GoodKind::USD, budget_per_good),
                (GoodKind::YUAN, budget_per_good),
                (GoodKind::YEN, budget_per_good),
            ]),
            register: Register {
                transactions: Vec::new(),
                day: 0,
            },
            initial_budget_in_euro: budget_per_good,
        }
    }

    pub fn trade_with_all_markets(&mut self, bfb: &mut ChosenMarket, rcnz: &mut ChosenMarket, zse: &mut ChosenMarket, trading_days: u32) {
        let run_time = Runtime::new().unwrap();
        loop {
            if self.register.day == trading_days {
                break;
            }
            let (good_kind, market_to_buy_from, market_to_sell_to) = self.get_max_profit_pair_by_exchange_rate(Rc::clone(bfb), Rc::clone(rcnz), Rc::clone(zse), self.get_priority());

            let available_good_quantity_to_buy = self.get_market_good_quantity_by_kind(&market_to_buy_from, &good_kind);

            if self.is_positive_and_big_enough(available_good_quantity_to_buy) && self.is_wallet_euro_balance_smaller_than_initial_euro_balance_after_buying(&market_to_buy_from, available_good_quantity_to_buy, &good_kind) {
                let (good_quantity_to_buy, price_market_wants_to_be_paid) = self.calculate_optimal_purchase_option(Rc::clone(&market_to_buy_from), available_good_quantity_to_buy, good_kind.clone());
                if !self.is_positive_and_big_enough(good_quantity_to_buy) {
                    run_time.block_on(self.send_trader_status());
                    self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                    wait_one_day!(bfb, rcnz, zse);
                    self.register.day += 1;
                    run_time.block_on(self.send_trader_status());
                    self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                    run_time.block_on(self.send_wait_event(good_kind, 0.0, &market_to_buy_from));
                    continue;
                }
                run_time.block_on(self.send_trader_status());
                self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                let lock_for_buying = market_to_buy_from.borrow_mut().lock_buy(good_kind.clone(), good_quantity_to_buy, price_market_wants_to_be_paid, self.name.clone());
                self.register.day += 1;
                run_time.block_on(self.send_trader_status());
                self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                run_time.block_on(self.send_lock_buy_event(good_kind, good_quantity_to_buy, &market_to_buy_from, price_market_wants_to_be_paid));
                if let Ok(token) = lock_for_buying {
                    run_time.block_on(self.send_trader_status());
                    self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                    let purchase = market_to_buy_from.borrow_mut().buy(token, &mut Good::new(GoodKind::EUR, price_market_wants_to_be_paid));
                    self.register.day += 1;
                    run_time.block_on(self.send_trader_status());
                    self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                    run_time.block_on(self.send_buy_event(good_kind, good_quantity_to_buy, &market_to_buy_from, price_market_wants_to_be_paid));
                    // println!("Price market wants to be paid: {}, good quantity in the market {}", price_market_wants_to_be_paid, self.get)
                    if let Ok(good) = purchase {
                        println!("Purchased successfully {} of {} and paid {}", good.get_qty(), good.get_kind(), price_market_wants_to_be_paid);
                        self.update_internal_state_after_buying(&market_to_buy_from, price_market_wants_to_be_paid, good, "sold".to_string());
                    }
                }

                let available_quantity_to_pay_with = self.get_market_good_quantity_by_kind(&market_to_sell_to, &GoodKind::EUR);

                if self.is_positive_and_big_enough(available_quantity_to_pay_with) {
                    let (good_quantity_to_sell, price_market_has_to_pay) = self.calculate_optimal_sale_option(Rc::clone(&market_to_sell_to), available_quantity_to_pay_with, good_kind.clone(), good_quantity_to_buy);
                    if !self.is_positive_and_big_enough(good_quantity_to_sell) {
                        run_time.block_on(self.send_trader_status());
                        self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                        wait_one_day!(bfb, rcnz, zse);
                        self.register.day += 1;
                        run_time.block_on(self.send_trader_status());
                        self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                        run_time.block_on(self.send_wait_event(good_kind, 0.0, &market_to_sell_to));
                        continue;
                    }
                    run_time.block_on(self.send_trader_status());
                    self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                    let lock_for_selling = market_to_sell_to.borrow_mut().lock_sell(good_kind.clone(), good_quantity_to_sell, price_market_has_to_pay, self.name.clone());
                    self.register.day += 1;
                    run_time.block_on(self.send_trader_status());
                    self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                    run_time.block_on(self.send_lock_sell_event(good_kind, good_quantity_to_sell, &market_to_sell_to, price_market_has_to_pay));
                    if let Ok(token) = lock_for_selling {
                        run_time.block_on(self.send_trader_status());
                        self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                        let sale = market_to_sell_to.borrow_mut().sell(token, &mut Good::new(good_kind.clone(), good_quantity_to_sell));
                        self.register.day += 1;
                        run_time.block_on(self.send_trader_status());
                        self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                        run_time.block_on(self.send_sell_event(good_kind, good_quantity_to_sell, &market_to_sell_to, price_market_has_to_pay));
                        if let Ok(good) = sale {
                            println!("Sold successfully {} of {} and earned {}", good_quantity_to_sell, good_kind, price_market_has_to_pay);
                            self.update_internal_state_after_selling(&market_to_sell_to, good.get_qty(), Good::new(good_kind, good_quantity_to_sell), "bought".to_string());
                        }
                    }
                }
            } else {
                println!("Waiting for a day");
                run_time.block_on(self.send_trader_status());
                self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                wait_one_day!(bfb, rcnz, zse);
                self.register.day += 1;
                run_time.block_on(self.send_trader_status());
                self.send_market_status_of_all_markets(&run_time, bfb, rcnz, zse);
                run_time.block_on(self.send_wait_event(good_kind, 0.0, &market_to_buy_from));
            }
        }
    }

    async fn send_trader_status(&self) {
        let client = reqwest::Client::new();
        let trader_goods: Vec<TraderGood> = self.wallet
            .iter()
            .map(|(good_kind, quantity)| TraderGood { kind: *good_kind, quantity: *quantity })
            .collect();
        let _ = client.post("http://localhost:8000/traderGoods").json(&trader_goods).send().await;
    }

    async fn send_market_status_of_all_markets(&self, run_time: &Runtime, bfb: &mut ChosenMarket, rcnz: &mut ChosenMarket, zse: &mut ChosenMarket) {
        run_time.block_on(self.send_market_status(bfb));
        run_time.block_on(self.send_market_status(rcnz));
        run_time.block_on(self.send_market_status(zse));
    }


    async fn send_market_status(&self, market: &ChosenMarket) {
        let client = reqwest::Client::new();
        let market_name_for_sending = self.get_market_name_for_sending(market);
        let labels: Vec<GoodLabel> = market.borrow().get_goods();
        let _res = client.post("http://localhost:8000/currentGoodLabels/".to_string() + &*market_name_for_sending).json(&labels).send().await;
    }

    async fn send_wait_event(&mut self, good_kind: GoodKind, quantity: f32, market: &ChosenMarket) {
        let client = reqwest::Client::new();
        let market_name_for_sending = self.get_market_name_for_sending(market);
        let log_event = craft_log_event(self.register.day, CustomEventKind::Wait, good_kind, quantity, 0.0, market_name_for_sending, true, None);
        let _ = client.post("http://localhost:8000/log").json(&log_event).send().await;
    }

    async fn send_buy_event(&mut self, good_kind: GoodKind, quantity: f32, market: &ChosenMarket, price: f32) {
        let client = reqwest::Client::new();
        let market_name_for_sending = self.get_market_name_for_sending(market);
        let log_event = craft_log_event(self.register.day, CustomEventKind::Bought, good_kind, quantity, price, market_name_for_sending, true, None);
        let _ = client.post("http://localhost:8000/log").json(&log_event).send().await;
    }

    async fn send_sell_event(&mut self, good_kind: GoodKind, quantity: f32, market: &ChosenMarket, price: f32) {
        let client = reqwest::Client::new();
        let market_name_for_sending = self.get_market_name_for_sending(market);
        let log_event = craft_log_event(self.register.day, CustomEventKind::Sold, good_kind, quantity, price, market_name_for_sending, true, None);
        let _ = client.post("http://localhost:8000/log").json(&log_event).send().await;
    }

    async fn send_lock_buy_event(&mut self, good_kind: GoodKind, quantity: f32, market: &ChosenMarket, price: f32) {
        let client = reqwest::Client::new();
        let market_name_for_sending = self.get_market_name_for_sending(market);
        let log_event = craft_log_event(self.register.day, CustomEventKind::LockedBuy, good_kind, quantity, price, market_name_for_sending, true, None);
        let _ = client.post("http://localhost:8000/log").json(&log_event).send().await;
    }

    async fn send_lock_sell_event(&mut self, good_kind: GoodKind, quantity: f32, market: &ChosenMarket, price: f32) {
        let client = reqwest::Client::new();
        let market_name_for_sending = self.get_market_name_for_sending(market);
        let log_event = craft_log_event(self.register.day, CustomEventKind::LockedSell, good_kind, quantity, price, market_name_for_sending, true, None);
        let _ = client.post("http://localhost:8000/log").json(&log_event).send().await;
    }

    fn get_market_name_for_sending(&self, market: &ChosenMarket) -> String {
        let market_name = self.get_market_name(market);
        if market_name == "Baku stock exchange" {
            return "BFB".to_string();
        }
        return market_name;
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
        for (day, market_name, action, good) in self.register.transactions.iter() {
            let mut to_from_pair = "to".to_string();
            if action == "bought" {
                to_from_pair = "from".to_string();
            }
            println!("Day {}: {} {} {} {} {}", day, market_name, action, good, to_from_pair, self.name);
        }
    }

    fn get_budget_in_euro(&self) -> f32 {
        self.get_money_by_kind(GoodKind::EUR)
    }

    fn is_positive_and_big_enough(&self, quantity: f32) -> bool {
        quantity > 0.0 && quantity > 1.0
    }

    // calculate_optimal_purchase_option always gives a pair that makes the trader earn money, but you only buy if
    fn is_wallet_euro_balance_smaller_than_initial_euro_balance_after_buying(&self, market: &ChosenMarket, available_good_quantity_to_buy: f32, good_kind: &GoodKind) -> bool {
        let (_, price_market_wants_to_be_paid) = self.calculate_optimal_purchase_option(Rc::clone(market), available_good_quantity_to_buy, good_kind.clone());
        let mut wallet_euro_balance = self.get_money_by_kind(GoodKind::EUR);
        wallet_euro_balance -= price_market_wants_to_be_paid;
        println!("Wallet euro balance if buy: {}", wallet_euro_balance);
        return wallet_euro_balance < self.initial_budget_in_euro;
    }

    fn calculate_optimal_purchase_option(&self, market: ChosenMarket, available_good_quantity_to_buy: f32, good_kind: GoodKind) -> (f32, f32) {
        let mut good_quantity_to_buy = available_good_quantity_to_buy * 2.0 / 3.0;
        let mut price_market_wants_to_be_paid = market.borrow().get_buy_price(good_kind.clone(), good_quantity_to_buy).unwrap();
        while price_market_wants_to_be_paid > self.get_budget_in_euro() {
            good_quantity_to_buy /= 2.0;
            price_market_wants_to_be_paid = market.borrow().get_buy_price(good_kind.clone(), good_quantity_to_buy).unwrap();
        }
        (good_quantity_to_buy, price_market_wants_to_be_paid)
    }

    fn calculate_optimal_sale_option(&self, market: ChosenMarket, good_available_quantity_to_pay_with: f32, good_kind: GoodKind, good_quantity_to_buy: f32) -> (f32, f32) {
        let mut good_quantity_to_pay_with = good_available_quantity_to_pay_with * 2.0 / 3.0;
        let mut good_quantity_to_sell = good_quantity_to_buy;
        let mut price_market_has_to_pay = market.borrow().get_sell_price(good_kind.clone(), good_quantity_to_sell).unwrap();
        while price_market_has_to_pay > good_quantity_to_pay_with {
            good_quantity_to_sell /= 2.0;
            price_market_has_to_pay = market.borrow().get_sell_price(good_kind.clone(), good_quantity_to_sell).unwrap();
        }
        (good_quantity_to_sell, price_market_has_to_pay)
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
        self.register.transactions.push((self.register.day, self.get_market_name(&market), action, good.clone()));
        self.wallet.insert(good.get_kind().clone(), self.get_money_by_kind(good.get_kind().clone()) + good.get_qty());
        self.wallet.insert(GoodKind::EUR, self.get_money_by_kind(GoodKind::EUR) - amount);
    }

    fn update_internal_state_after_selling(&mut self, market: &ChosenMarket, amount: f32, good: Good, action: Action) {
        self.register.transactions.push((self.register.day, self.get_market_name(&market), action, good.clone()));
        self.wallet.insert(good.get_kind().clone(), self.get_money_by_kind(good.get_kind().clone()) - good.get_qty());
        self.wallet.insert(GoodKind::EUR, self.get_money_by_kind(GoodKind::EUR) + amount);
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
        market.borrow().get_goods().iter().find(|good| good.good_kind == *good_kind).unwrap().to_owned()
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
        let mut trader = Trader_TR::new("RAST".to_string(), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::EUR), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::USD), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::YUAN), 100.0);
        assert_eq!(trader.get_money_by_kind(GoodKind::YEN), 100.0);
    }

    #[test]
    fn test_get_default_exchange_rates() {
        let mut trader = Trader_TR::new("RAST".to_string(), 100.0);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::EUR).unwrap(), &1.0);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::USD).unwrap(), &DEFAULT_EUR_USD_EXCHANGE_RATE);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::YUAN).unwrap(), &DEFAULT_EUR_YUAN_EXCHANGE_RATE);
        assert_eq!(trader.get_default_exchange_rates().get(&GoodKind::YEN).unwrap(), &DEFAULT_EUR_YEN_EXCHANGE_RATE);
    }

    #[test]
    fn test_get_all_money_in_euro() {
        let mut trader = Trader_TR::new("RAST".to_string(), 100.0);
        let mut total_amount = trader.get_money_by_kind(GoodKind::EUR) / 1.0;
        total_amount += trader.get_money_by_kind(GoodKind::USD) / DEFAULT_EUR_USD_EXCHANGE_RATE;
        total_amount += trader.get_money_by_kind(GoodKind::YUAN) / DEFAULT_EUR_YUAN_EXCHANGE_RATE;
        total_amount += trader.get_money_by_kind(GoodKind::YEN) / DEFAULT_EUR_YEN_EXCHANGE_RATE;
        assert!(f32::abs(trader.get_all_money_in_euro() - total_amount) < 0.0001);
    }
}
