use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{LockBuyError, LockSellError, Market, MarketGetterError};
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use unitn_market_2022::wait_one_day;
use ZSE::market::ZSE;
use crate::common::visualizer::{craft_log_event, CustomEventKind, LogEvent, wait_before_calling_api};
use futures::executor::block_on;
use tokio::runtime::Runtime;
use serde::{Deserialize, Serialize};
use crate::common;

//TraderGood struct, necessary for sending data to visualizer
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TraderGood{
    kind: GoodKind,
    quantity: f32
}


//the struct for the trader
//it has as attributes its name, the budget at its disposal in terms of quantity of EURs
//the list of other goods it has at its disposal, the three markets with which it interacts,
//and the time, used both for events and for indicating the number of interaction
pub struct Trader_SA {
    pub name: String,
    pub cash: f32,
    pub goods: Vec<Rc<RefCell<Good>>>,
    pub bfb: Rc<RefCell<dyn Market>>,
    pub rcnz: Rc<RefCell<dyn Market>>,
    pub zse: Rc<RefCell<dyn Market>>,
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

    //ASYNC METHODS:
    //These methods are called when we need to call the API for the visualizer

    //use this function when it is necessary to send the vec of TraderGoods to the visualizer
    async fn send_trader_goods(&self){
        let client = reqwest::Client::new();

        let mut tradergoods = vec![];
        tradergoods.push(TraderGood{kind: GoodKind::EUR, quantity: self.cash});
        for goodkind in &self.goods{
            tradergoods.push(TraderGood{kind: goodkind.borrow().get_kind().clone(), quantity: goodkind.borrow().get_qty()});
        }
        //println!("{:?}",tradergoods);
        wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
        let _res = client
            .post("http://localhost:8000/traderGoods")
            .json(&tradergoods)
            .send()
            .await;
    }

    //wait function, called on wait events
    async fn wait(&mut self, goodkind: GoodKind, quantity: f32, price: f32, market_name: &str){
        let client = reqwest::Client::new();

        wait_one_day!(self.bfb, self.rcnz, self.zse);

        wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
        let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Wait, goodkind, 0.0, 0.0, market_name.to_string(), true, None)).send().await;
        self.time += 1;
    }

    //use this function to send the GoodLabels of each market to the visualizer
    async fn send_labels(&mut self){
        let client = reqwest::Client::new();
        let labels_bfb = self.bfb.borrow().get_goods();
        let labels_rcnz = self.rcnz.borrow().get_goods();
        let labels_zse = self.zse.borrow().get_goods();
        wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
        let _res = client
            .post("http://localhost:8000/currentGoodLabels/".to_string() + "BFB")
            .json(&labels_bfb)
            .send()
            .await;
        let _res = client
            .post("http://localhost:8000/currentGoodLabels/".to_string() + "RCNZ")
            .json(&labels_rcnz)
            .send()
            .await;
        let _res = client
            .post("http://localhost:8000/currentGoodLabels/".to_string() + "ZSE")
            .json(&labels_zse)
            .send()
            .await;
    }

    //METHODS USED FOR STRATEGY:

    //function for finding how much of any good i can buy from a market given euros at disposal
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

    //function for finding how much of a good i can sell to a market given the quantity of that good at disposal
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

    //lock buy function
    pub async fn lock_buy_from_market(&mut self, market_name: &str, goodkind: GoodKind, quantity: f32, price: f32, trader_name: String) -> Result<String, LockBuyError>{
        let market = match market_name{
            "RCNZ" => &mut self.rcnz,
            "ZSE" => &mut self.zse,
            _ => &mut self.bfb
        };

        let client = reqwest::Client::new();

        match market.borrow_mut().lock_buy(goodkind, quantity, price, trader_name) {
            Ok(token) => {
                wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedBuy, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
                self.time += 1;
                Ok(token)
            },
            Err(e) => {
                let e_string = format!("{:?}",e);
                wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedBuy, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
                Err(e)
            }
        }
    }


    //function for buying a good from a market
    pub async fn buy_from_market(&mut self, market_name: &str, goodkind: GoodKind, quantity: f32, price: f32, token: String){

        let client = reqwest::Client::new();

        let market = match market_name{
            "RCNZ" => &mut self.rcnz,
            "ZSE" => &mut self.zse,
            _ => &mut self.bfb
        };

        //use the token to buy the good
        match market.borrow_mut().buy(token, &mut Good::new(GoodKind::EUR, price)) {
            Ok(increase) => {
                wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Bought, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
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
            },
            Err(e) => {
                let e_string = format!("{:?}",e);
                wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                let _res = client.post("http://http://127.0.0.1:8000//log").json(&craft_log_event(self.time, CustomEventKind::Bought, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
            }
        };
    }

    //lock sell function
    pub async fn lock_sell_to_market(&mut self, market_name: &str, goodkind: GoodKind, quantity: f32, price: f32, trader_name: String) -> Result<String, LockSellError>{
        let market = match market_name{
            "RCNZ" => &mut self.rcnz,
            "ZSE" => &mut self.zse,
            _ => &mut self.bfb
        };

        let client = reqwest::Client::new();

        match market.borrow_mut().lock_sell(goodkind, quantity, price, trader_name) {
            Ok(token) => {
                wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
                self.time += 1;
                Ok(token)
            },
            Err(e) => {
                let e_string = format!("{:?}",e);
                wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::LockedSell, goodkind, quantity, price, market_name.to_string(), false, Some(e_string))).send().await;
                Err(e)
            }
        }
    }



    //function for selling a good to a market
    pub async fn sell_to_market(&mut self, market_name: &str, goodkind: GoodKind, quantity: f32, price: f32, token: String){
            let client = reqwest::Client::new();

            let market = match market_name{
                "RCNZ" => &mut self.rcnz,
                "ZSE" => &mut self.zse,
                _ => &mut self.bfb
            };

            //use the token to sell the good
            match market.borrow_mut().sell(token, &mut Good::new(goodkind.clone(), quantity)) {
                Ok(increase_eur) => {
                    wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                    let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Sold, goodkind, quantity, price, market_name.to_string(), true, None)).send().await;
                    self.time += 1;
                    //now that we have sold the good to the market, now we have to change
                    //the quantities inside the trader
                    self.cash += increase_eur.get_qty();
                    for good in self.goods.iter_mut() {
                        if good.borrow().get_kind() == goodkind {
                            match good.borrow_mut().split(quantity) {
                                Ok(_) => (),
                                Err(e) => panic!("Error in split {:?}, best_qty: {} in {}", e, quantity, market_name),
                            }
                        }
                    }
                },
                Err(e) => {
                    let e_string = format!("{:?}",e);
                    wait_before_calling_api(common::trader_config::get_trader_config().get_delay_in_milliseconds());
                    let _res = client.post("http://localhost:8000/log").json(&craft_log_event(self.time, CustomEventKind::Sold, goodkind, quantity, price, market_name.to_string(), false, Some(e_string.clone()))).send().await;
                }
            };
    }


    // BOT METHOD: apply bot strategy for i loop interactions
    //with this function we interact with all markets
    pub fn strategy(&mut self, mut i: u32) {
        //define Runtime object used for calling async functions
        let rt  = Runtime::new().unwrap();
        //initial call to the API, to visualize our initial wallet and labels
        rt.block_on(self.send_labels());
        rt.block_on(self.send_trader_goods());
        loop {
            //loop until i reaches 0
            if i < self.time {
                break;
            }

            //for each market get the best kind and quantity of good which could maximize profit
            let (best_quantity_bfb, kind_quantity_bfb) = self.find_best_buy_quantity(&self.bfb);
            let (best_quantity_rcnz, kind_quantity_rcnz) = self.find_best_buy_quantity(&self.rcnz);
            let (best_quantity_zse, kind_quantity_zse) = self.find_best_buy_quantity(&self.zse);

            //define prices with kinds and quantity obtained previously
            let price_bfb = self.bfb.borrow_mut().get_buy_price(kind_quantity_bfb, best_quantity_bfb);
            let price_rcnz = self.rcnz.borrow_mut().get_buy_price(kind_quantity_rcnz, best_quantity_rcnz * 0.75);
            let price_zse = self.zse.borrow_mut().get_buy_price(kind_quantity_zse, best_quantity_zse);

            //define variables for deciding the best buy operation
            //in this case we prefer to trade with BFB market but if
            //another market has better prices, we will buy from that market
            let mut min_buy_price = f32::MAX;
            let mut best_market = &self.bfb;
            let mut best_kind = kind_quantity_bfb;
            let mut best_quantity = best_quantity_bfb;
            let mut market_name = "BFB";

            //choose the best market, based on the price we obtained
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

            //check if the quantity is bigger than 1 and if at least one price is "ok"
            //(i.e. it doesn't have an error as its output)
            if best_quantity > 1.0 && min_buy_price < f32::MAX{
                //get the buy_price
                let price = match best_market.borrow().get_buy_price(best_kind, best_quantity) {
                    Ok(price) => price,
                    Err(e) => {
                        panic!("Error in get_buy_price: {:?}", e);
                    }
                };

                //do the lock_buy
                let token = rt.block_on(self.lock_buy_from_market(market_name, best_kind, best_quantity, price, self.get_trader_name()));

                if let Ok(token) = token{
                    //buy
                    rt.block_on(self.send_labels());
                    rt.block_on(self.send_trader_goods());
                    rt.block_on(self.buy_from_market(market_name, best_kind, best_quantity, price, token));
                    rt.block_on(self.send_labels());
                    rt.block_on(self.send_trader_goods());
                }

            } else {
                //wait
                rt.block_on(self.wait(best_kind, 0.0, 0.0, market_name));
                rt.block_on(self.send_labels());
                rt.block_on(self.send_trader_goods());
            }

            //for each market get the best quantity of good which could maximize profit
            //in this case we will sell the same good, hoping that we can do profit
            let best_quantity_bfb_sell = self.find_best_sell_quantity(&self.bfb, best_kind.clone());
            let best_quantity_rcnz_sell = self.find_best_sell_quantity(&self.rcnz, best_kind.clone());
            let best_quantity_zse_sell = self.find_best_sell_quantity(&self.zse, best_kind.clone());

            //define prices with kinds and quantity obtained previously
            let price_sell_bfb = self.bfb.borrow_mut().get_sell_price(best_kind, best_quantity_bfb_sell);
            let price_sellrcnz = self.rcnz.borrow_mut().get_sell_price(best_kind, best_quantity_rcnz_sell * 0.7);
            let price_sell_zse = self.zse.borrow_mut().get_sell_price(best_kind, best_quantity_zse_sell);

            //define variables for deciding the best sell operation
            //in this case we prefer to trade with BFB market but if
            //another market has better prices, we will buy from that market
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

            //check if the quantity is bigger than 1 and if at least one price is "ok"
            //(i.e. it doesn't have an error as its output)
            if best_quantity_sell > 1.0 && max_sell_price > 0.0{
                //we repeat the same procedure we did for the buy part, but now we consider variables for selling
                let price_sell = match best_market_sell.borrow().get_sell_price(best_kind, best_quantity_sell) {
                    Ok(price_sell) => price_sell,
                    Err(e) => {
                        panic!("Error in get_sell_price in zse: {:?}", e);
                    }
                };

                //do the lock_sell
                let token = rt.block_on(self.lock_sell_to_market(market_name_sell,best_kind, best_quantity_sell, price_sell, self.get_trader_name()));

                if let Ok(token) = token{
                    //sell
                    rt.block_on(self.send_labels());
                    rt.block_on(self.send_trader_goods());
                    rt.block_on(self.sell_to_market(market_name_sell,best_kind, best_quantity_sell, price_sell, token));
                    rt.block_on(self.send_labels());
                    rt.block_on(self.send_trader_goods());
                }
            } else {
                //wait
                rt.block_on(self.wait(best_kind, 0.0, 0.0, market_name_sell));
                rt.block_on(self.send_labels());
                rt.block_on(self.send_trader_goods());
            }

        }
    }
}
