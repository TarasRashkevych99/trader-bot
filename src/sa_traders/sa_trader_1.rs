use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{LockSellError, Market, MarketGetterError};
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use unitn_market_2022::wait_one_day;
use ZSE::market::ZSE;
use crate::sa_traders::log_event::{craft_log_event, CustomEventKind, LogEvent};
use futures::executor::block_on;


//the struct for the trader
#[derive(Clone)]
pub struct Trader_SA {
    pub name: String,
    pub cash: f32,
    pub goods: Vec<Rc<RefCell<Good>>>,
    pub bfb: Rc<RefCell<dyn Market>>,
    pub rcnz: Rc<RefCell<dyn Market>>,
    pub zse: Rc<RefCell<dyn Market>>,
    pub register: Vec<LogEvent>,
    pub time: u32
}

impl Trader_SA {
    //the constructor for the trader
    pub(crate) fn new(name: String, cash: f32, bfb: Rc<RefCell<dyn Market>>, rcnz: Rc<RefCell<dyn Market>>, zse: Rc<RefCell<dyn Market>>, ) -> Self {
        //create the list of goods, we start with only euros, which means all other goods will have quantity 0.0
        let mut goods: Vec<Rc<RefCell<Good>>> = vec![];
        goods.push(Rc::new(RefCell::new(Good::new(GoodKind::USD, 0.0))));
        goods.push(Rc::new(RefCell::new(Good::new(GoodKind::YEN, 0.0))));
        goods.push(Rc::new(RefCell::new(Good::new(GoodKind::YUAN, 0.0))));

        //return the new trader
        Trader_SA {
            name,
            cash,
            goods,
            bfb,
            rcnz,
            zse,
            register: vec![],
            time: 1
        }
    }

    //GETTER METHODS:

    //get trader name
    fn get_trader_name(&self) -> String {
        self.name.clone()
    }

    //get trader cash at its disposal
    fn get_trader_cash(&self) -> f32 {
        self.cash
    }

    //get trader goods at its disposal
    fn get_trader_goods(&self) -> Vec<Rc<RefCell<Good>>> {
        self.goods.clone()
    }

    fn get_trader_register(&self) -> Vec<LogEvent>{
        self.register.clone()
    }

    //get the quantity of a certain good, EUR included
    fn get_trader_goodquantity(&self, goodkind: GoodKind) -> f32 {
        match goodkind {
            GoodKind::EUR => {
                self.get_trader_cash()
            }
            _ => {
                let mut result = 0.0;
                for good in &self.goods {
                    if good.borrow().get_kind() == goodkind {
                        result = good.borrow().get_qty();
                    }
                }
                result
            }
        }
    }

    fn update_time(&mut self){
        self.time += 1;
    }

    fn wait(&mut self){
        let client = reqwest::Client::new();
        wait_one_day!(self.bfb);
        wait_one_day!(self.rcnz);
        wait_one_day!(self.zse);
        self.time += 1;
    }

    //OTHER METHODS USED FOR STRATEGY:

    //function for finding how much of a product i can buy from a market given euros at disposal
    pub fn find_best_buy_quantity(&self, market: &Rc<RefCell<dyn Market>>) -> (f32, GoodKind) {
        let mut best_quantity = 0.0;
        let mut best_kind = GoodKind::USD;
        let mut lowest_price = -1.0;

        for good in &self.goods {
            let mut temp_best_qty = 0.0;
            for market_good in market.borrow().get_goods() {
                if good.borrow().get_kind() == market_good.good_kind {
                    temp_best_qty = market_good.quantity;
                }
            }
            let mut buy_price = f32::MAX;
            if temp_best_qty > 0.0 {
                buy_price = market.borrow().get_buy_price(good.borrow().get_kind(), temp_best_qty).expect("Error in find_best_buy_quantity function");
                while self.cash < buy_price && temp_best_qty > 0.01 {
                    temp_best_qty = temp_best_qty * 0.5;
                    buy_price = market.borrow().get_buy_price(good.borrow().get_kind(), temp_best_qty).expect("Error in find_best_buy_quantity function");
                }
            }
            if (lowest_price > buy_price) || (lowest_price < 0.0) {
                lowest_price = buy_price;
                best_quantity = temp_best_qty;
                best_kind = good.borrow().get_kind();
            }
        }

        (best_quantity, best_kind)
    }

    pub fn find_best_sell_quantity(&self, market: &Rc<RefCell<dyn Market>>, goodkind: GoodKind) -> f32 {
        let mut sell_price = 0.0;
        let mut eur_qty = 0.0;

        for market_good in market.borrow().get_goods() {
            if market_good.good_kind == GoodKind::EUR {
                eur_qty = market_good.quantity;
            }
        }
        let mut best_quantity = self.get_trader_goodquantity(goodkind);
        if best_quantity > 0.0 {
            sell_price = market.borrow().get_sell_price(goodkind, best_quantity).expect("Error in find_best_sell_quantity function");
            while eur_qty < sell_price && best_quantity > 0.1 {
                best_quantity = best_quantity * 0.5;
                sell_price = market.borrow().get_sell_price(goodkind, best_quantity).expect("Error in find_best_sell_quantity function");
            }
        }

        best_quantity
    }

    pub async fn buy_from_market(&mut self, market_name: &str, goodkind: GoodKind, quantity: f32, price: f32, trader_name: String){

        let client = reqwest::Client::new();

        let market = match market_name{
            "RCNZ" => &mut self.rcnz,
            "ZSE" => &mut self.zse,
            _ => &mut self.bfb
        };

        let mut cash = Good::new(GoodKind::EUR, price);
        let token = match market.borrow_mut().lock_buy(goodkind, quantity, price, trader_name) {
                Ok(token) => token,
                Err(e) => {
                    let e_string = format!("{:?}",e);
                    self.register.push(craft_log_event(self.time, CustomEventKind::LockedBuy, goodkind, quantity, price, market_name.to_string(), false, Some(e_string.clone())));
                    let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedBuy, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
                    panic!("Error in lock_buy in {}: {:?}", market_name, e);
                }
            };

        self.register.push(craft_log_event(self.time, CustomEventKind::LockedBuy, goodkind, quantity, price, market_name.to_string(), true, None));
        let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedBuy, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;

        //self.update_time();
        self.time += 1;

        //use the token to buy the good
        let increase = match market.borrow_mut().buy(token, &mut cash) {
            Ok(increase) => increase,
            Err(e) => {
                let e_string = format!("{:?}",e);
                self.register.push(craft_log_event(self.time, CustomEventKind::Bought, goodkind, quantity, price, market_name.to_string(), false, Some(e_string.clone())));
                let _res = client.post("http://http://127.0.0.1:8000//log").json(&craft_log_event(self.time, CustomEventKind::Bought, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
                panic!("Error in buy in bfb: {:?}", e);
            }
        };

        self.register.push(craft_log_event(self.time, CustomEventKind::Bought, goodkind, quantity, price, market_name.to_string(), true, None));
        let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Bought, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
        //self.update_time();
        self.time += 1;

        //now that we have bought the good from the market, now we have to change
        //the quantities inside the trader
        self.cash -= price;
        for good in self.goods.iter_mut() {
            if good.borrow().get_kind() == goodkind {
                match good.borrow_mut().merge(increase.clone()) {
                    Ok(_) => (),
                    Err(e) => println!("Error in merge {:?}", e),
                }
            }
        }
    }

    pub async fn sell_from_market(&mut self, market_name: &str, goodkind: GoodKind, quantity: f32, price: f32, trader_name: String){

        let client = reqwest::Client::new();

        let market = match market_name{
            "RCNZ" => &mut self.rcnz,
            "ZSE" => &mut self.zse,
            _ => &mut self.bfb
        };

        let mut bool_sell_error = false;

        //get the token from lock_sell
        let token_sell = match market.borrow_mut().lock_sell(goodkind, quantity, price, trader_name) {
            Ok(token_sell) => token_sell,
            Err(LockSellError::MaxAllowedLocksReached) => {
                bool_sell_error = true;
                let e_string = format!("LockSellError::MaxAllowedLocksReached");
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
                "LockSellError::MaxAllowedLocksReached".to_string()
            },
            Err(LockSellError::InsufficientDefaultGoodQuantityAvailable{ .. }) => {
                bool_sell_error = true;
                let e_string = format!("LockSellError::InsufficientDefaultGoodQuantityAvailable");
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
                "LockSellError::InsufficientDefaultGoodQuantityAvailable".to_string()
            },
            Err(e) => {
                let e_string = format!("{:?}",e);
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), false, Some(e_string.clone()))).send().await;
                self.register.push(craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), false, Some(e_string)));
                panic!("Error in lock_sell: {:?} in {}, since we are locking {} at {} with offer {}", e, market_name, goodkind, quantity, price);
            }
        };



        if !bool_sell_error {
            self.register.push(craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), true, None));
            let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
            //self.update_time();
            self.time += 1;
            //get the cash from the market
            let mut sold_good = Good::new(goodkind, quantity);
            //println!("{}",sold_good.get_qty());
            let increase_eur = match market.borrow_mut().sell(token_sell, &mut sold_good) {
                Ok(increase_eur) => increase_eur,
                Err(e) => {
                    let e_string = format!("{:?}",e);
                    let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Sold, goodkind, quantity, price, market_name.to_string(), false, Some(e_string.clone()))).send().await;
                    self.register.push(craft_log_event(self.time, CustomEventKind::Sold, goodkind, quantity, price, market_name.to_string(), false, Some(e_string)));
                    panic!("Error in sell in {:?}", e);
                }
            };

            self.register.push(craft_log_event(self.time, CustomEventKind::Sold, goodkind, quantity, price, market_name.to_string(), true, None));
            let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Sold, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
            //self.update_time();
            self.time += 1;

            self.cash += increase_eur.get_qty();
            for good in self.goods.iter_mut() {
                if good.borrow().get_kind() == goodkind {
                    match good.borrow_mut().split(quantity) {
                        Ok(_) => (),
                        Err(e) => panic!("Error in split {:?}, best_qty: {} in {}", e, quantity, market_name),
                    }
                }
            }
        } else {
            self.wait();
        }
    }


    // BOT METHOD: apply bot strategy for i loop interactions
    //and returns the string with all the actions done by the trader
    //with this function we only interact with one market
    pub fn strategy_bfb(&mut self, mut i: i32) {
        loop {
            //loop until i reaches 0
            if i <= 0 {
                break;
            }

            let original_budget = self.cash;

            let (best_quantity_bfb, kind_quantity_bfb) = self.find_best_buy_quantity(&self.bfb);


            if best_quantity_bfb > 1.0 {
                let price = match self.bfb.borrow().get_buy_price(kind_quantity_bfb, best_quantity_bfb) {
                    Ok(price) => price,
                    Err(e) => {
                        panic!(
                            "Error in get_buy_price in bfb: {:?}",
                            e
                        );
                    }
                };

                self.buy_from_market("BFB",kind_quantity_bfb,best_quantity_bfb,price, self.get_trader_name());
            } else {
                self.wait();
            }

            let best_quantity_bfb_sell = self.find_best_sell_quantity(&self.bfb, kind_quantity_bfb.clone());

            if best_quantity_bfb_sell > 1.0 {
                //we repeat the same procedure we did for the buy part, but now we consider variables for selling
                let price_sell = match self.bfb.borrow().get_sell_price(kind_quantity_bfb, best_quantity_bfb_sell) {
                    Ok(price_sell) => price_sell,
                    Err(e) => {
                        panic!(
                            "Error in get_sell_price in bfb: {:?}",
                            e
                        );
                    }
                };
                let final_budget_pre_sell = self.cash + price_sell;
                println!("Now trader has {} euros", self.cash);
                if (original_budget > final_budget_pre_sell) {
                    break;
                }
                self.sell_from_market("BFB",kind_quantity_bfb,best_quantity_bfb_sell,price_sell, self.get_trader_name());
            } else {
                self.wait();
            }

            i -= 1;
        }
    }

    //RCNZ
    pub fn strategy_rcnz(&mut self, mut i: i32) {
        loop {
            //loop until i reaches 0
            if i <= 0 {
                break;
            }

            let original_budget = self.cash;
            let (best_quantity_rcnz, kind_quantity_rcnz) = self.find_best_buy_quantity(&self.rcnz);
            let mut best_quantity_rcnz = best_quantity_rcnz * 0.8;

            if best_quantity_rcnz > 1.0 {
                let price = match self.rcnz.borrow().get_buy_price(kind_quantity_rcnz, best_quantity_rcnz) {
                    Ok(price) => price,
                    Err(e) => {
                        panic!(
                            "Error in get_buy_price in rcnz: {:?}",
                            e
                        );
                    }
                };
                self.buy_from_market("RCNZ", kind_quantity_rcnz, best_quantity_rcnz, price, self.get_trader_name());
            } else {
                self.wait();
            }

            let best_quantity_rcnz_sell = self.find_best_sell_quantity(&self.rcnz, kind_quantity_rcnz.clone());

            if best_quantity_rcnz_sell > 1.0 {
                //we repeat the same procedure we did for the buy part, but now we consider variables for selling
                let price_sell = match self.rcnz.borrow().get_sell_price(kind_quantity_rcnz, best_quantity_rcnz_sell) {
                    Ok(price_sell) => price_sell,
                    Err(e) => {
                        panic!(
                            "Error in get_sell_price in rcnz: {:?}",
                            e
                        );
                    }
                };
                let final_budget_pre_sell = self.cash + price_sell;
                println!("Now trader has {} euros", self.cash);
                if (original_budget > final_budget_pre_sell) {
                    break;
                }
                self.sell_from_market("RCNZ",kind_quantity_rcnz, best_quantity_rcnz_sell, price_sell, self.get_trader_name());
            } else {
                self.wait();
            }

            i -= 1;
        }
    }

    pub fn strategy_zse(&mut self, mut i: i32) {
        loop {
            //loop until i reaches 0
            if i <= 0 {
                break;
            }

            let original_budget = self.cash;
            let (best_quantity_zse, kind_quantity_zse) = self.find_best_buy_quantity(&self.zse);

            if best_quantity_zse > 1.0 {
                let price = match self.zse.borrow().get_buy_price(kind_quantity_zse, best_quantity_zse) {
                    Ok(price) => price,
                    Err(e) => {
                        panic!(
                            "Error in get_buy_price in rcnz: {:?}",
                            e
                        );
                    }
                };
                self.buy_from_market("ZSE", kind_quantity_zse, best_quantity_zse, price, self.get_trader_name());
            } else {
                self.wait();
            }

            let best_quantity_zse_sell = self.find_best_sell_quantity(&self.zse, kind_quantity_zse.clone());

            if best_quantity_zse_sell > 1.0 {
                //we repeat the same procedure we did for the buy part, but now we consider variables for selling
                let price_sell = match self.zse.borrow().get_sell_price(kind_quantity_zse, best_quantity_zse_sell) {
                    Ok(price_sell) => price_sell,
                    Err(e) => {
                        panic!(
                            "Error in get_sell_price in zse: {:?}",
                            e
                        );
                    }
                };
                let final_budget_pre_sell = self.cash + price_sell;
                println!("Now trader has {} euros", self.cash);
                if (original_budget > final_budget_pre_sell) {
                    break;
                }
                self.sell_from_market("ZSE",kind_quantity_zse, best_quantity_zse_sell, price_sell, self.get_trader_name());
            } else {
                self.wait();
            }

            i -= 1;
        }
    }


    // BOT METHOD: apply bot strategy for i loop interactions
    //and returns the string with all the actions done by the trader
    //with this function we only interact with all markets
    pub fn strategy(&mut self, mut i: i32) {
        loop {
            //loop until i reaches 0
            if i <= 0 {
                break;
            }

            let original_budget = self.cash;

            let (best_quantity_bfb, kind_quantity_bfb) = self.find_best_buy_quantity(&self.bfb);
            let (best_quantity_rcnz, kind_quantity_rcnz) = self.find_best_buy_quantity(&self.rcnz);
            let (best_quantity_zse, kind_quantity_zse) = self.find_best_buy_quantity(&self.zse);

            let price_bfb = self.bfb.borrow_mut().get_buy_price(kind_quantity_bfb, best_quantity_bfb);
            let price_rcnz = self.rcnz.borrow_mut().get_buy_price(kind_quantity_rcnz, best_quantity_rcnz * 0.75);
            let price_zse = self.zse.borrow_mut().get_buy_price(kind_quantity_zse, best_quantity_zse);

            let mut min_buy_price = f32::MAX;

            let mut best_market = &self.bfb;
            let mut best_kind = kind_quantity_bfb;
            let mut best_quantity = best_quantity_bfb;
            let mut market_name = "BFB";


            match price_bfb{
                Ok(_) => {
                    if min_buy_price > price_bfb.clone().unwrap(){
                        min_buy_price = price_bfb.unwrap();
                        best_market = &self.bfb;
                        best_kind = kind_quantity_bfb;
                        best_quantity = best_quantity_bfb;
                        market_name = "BFB";
                    }
                }
                Err(_) => {}
            };

            match price_rcnz{
                Ok(_) => {
                    if min_buy_price > price_rcnz.clone().unwrap(){
                        min_buy_price = price_rcnz.unwrap();
                        best_market = &self.rcnz;
                        best_kind = kind_quantity_rcnz;
                        best_quantity = best_quantity_rcnz * 0.75;
                        market_name = "RCNZ";
                    }
                }
                Err(_) => {}
            };

            match price_zse{
                Ok(_) => {
                    if min_buy_price > price_zse.clone().unwrap(){
                        min_buy_price = price_zse.unwrap();
                        best_market = &self.zse;
                        best_kind = kind_quantity_zse;
                        best_quantity = best_quantity_zse;
                        market_name = "ZSE";
                    }}
                Err(_) => {}
            };

            if best_quantity > 1.0 && min_buy_price < f32::MAX{
                let price = match best_market.borrow().get_buy_price(best_kind, best_quantity) {
                    Ok(price) => price,
                    Err(e) => {
                        panic!("Error in get_buy_price: {:?}", e);
                    }
                };
                block_on(self.buy_from_market(market_name, best_kind, best_quantity, price, self.get_trader_name()));
            } else {
                self.wait();
            }

            let best_quantity_bfb_sell = self.find_best_sell_quantity(&self.bfb, best_kind.clone());
            let best_quantity_rcnz_sell = self.find_best_sell_quantity(&self.rcnz, best_kind.clone());
            let best_quantity_zse_sell = self.find_best_sell_quantity(&self.zse, best_kind.clone());

            let price_sell_bfb = self.bfb.borrow_mut().get_sell_price(best_kind, best_quantity_bfb_sell);
            let price_sellrcnz = self.rcnz.borrow_mut().get_sell_price(best_kind, best_quantity_rcnz_sell * 0.7);
            let price_sell_zse = self.zse.borrow_mut().get_sell_price(best_kind, best_quantity_zse_sell);

            let mut max_sell_price = 0.0;
            let mut best_market_sell = &self.bfb;
            let mut best_quantity_sell = best_quantity_bfb_sell;
            let mut market_name_sell = "BFB";

            match price_sell_bfb{
                Ok(_) => {
                    if max_sell_price < price_sell_bfb.clone().unwrap(){
                        max_sell_price = price_sell_bfb.unwrap();
                        best_market_sell = &self.bfb;
                        best_quantity_sell = best_quantity_bfb_sell;
                        market_name_sell = "BFB";
                    }
                }
                Err(_) => {}
            };


            match price_sellrcnz{
                Ok(_) => {

                    if max_sell_price < price_sellrcnz.clone().unwrap(){
                        max_sell_price = price_sellrcnz.unwrap();
                        best_market_sell = &self.rcnz;
                        best_quantity_sell = best_quantity_rcnz_sell * 0.75;
                        market_name_sell = "RCNZ";
                    }
                }
                Err(_) => {}
            };

            match price_sell_zse{
                Ok(_) => {
                    if max_sell_price < price_sell_zse.clone().unwrap(){
                        max_sell_price = price_sell_zse.unwrap();
                        best_market_sell = &self.zse;
                        best_quantity_sell = best_quantity_zse_sell;
                        market_name_sell = "ZSE";
                    }}
                Err(_) => {}
            };



            if best_quantity_sell > 1.0 && max_sell_price > 0.0{
                //we repeat the same procedure we did for the buy part, but now we consider variables for selling
                let price_sell = match best_market_sell.borrow().get_sell_price(best_kind, best_quantity_sell) {
                    Ok(price_sell) => price_sell,
                    Err(e) => {
                        panic!("Error in get_sell_price in zse: {:?}", e);
                    }
                };

                let final_budget_pre_sell = self.cash + price_sell;
                //println!("Now trader has {} euros", self.cash);
                /*if (original_budget > final_budget_pre_sell) {
                    break;
                }*/
                block_on(self.sell_from_market(market_name_sell,best_kind, best_quantity_sell, price_sell, self.get_trader_name()));

            } else {
                self.wait();
            }
            println!("Now trader has {} euros", self.cash);
            i -= 1;
        }
    }
}



#[cfg(test)]
mod test {
    use crate::common::markets::new_with_quantities;
    use super::*;

    #[test]
    fn test_get_money_by_kind() {
        let _trader_name = "Trader_RAST_SA".to_string();
        let (bfb, rcnz, zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);
        let mut trader_sa = Trader_SA::new(_trader_name,10000.0, bfb.clone(), rcnz.clone(), zse.clone());
        assert_eq!(trader_sa.get_trader_goodquantity(GoodKind::EUR), 10000.0);
        assert_eq!(trader_sa.get_trader_goodquantity(GoodKind::USD), 0.0);
        assert_eq!(trader_sa.get_trader_goodquantity(GoodKind::YUAN), 0.0);
        assert_eq!(trader_sa.get_trader_goodquantity(GoodKind::YEN), 0.0);
    }
/*
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
*/
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
