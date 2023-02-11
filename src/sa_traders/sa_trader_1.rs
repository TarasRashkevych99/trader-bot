use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{Market, MarketGetterError};
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use ZSE::market::ZSE;

// **Sabin Andone**
//the struct for the trader
#[derive(Clone)]
pub struct Trader_SA {
    pub(crate) name: String,
    pub cash: f32,
    pub goods: Vec<Rc<RefCell<Good>>>,
    pub bfb: Rc<RefCell<dyn Market>>,
    pub rcnz: Rc<RefCell<dyn Market>>,
    pub zse: Rc<RefCell<dyn Market>>,
}

// **Sabin Andone**
impl Trader_SA {

    //the constructor for the trader
    pub(crate) fn new(
        cash: f32,
        bfb: Rc<RefCell<dyn Market>>,
        rcnz: Rc<RefCell<dyn Market>>,
        zse: Rc<RefCell<dyn Market>>,
    ) -> Self {
        //create the list of goods, we start with only euros, which means all other goods will have quantity 0.0
        let mut goods: Vec<Rc<RefCell<Good>>> = vec![];
        goods.push(Rc::new(RefCell::new(Good::new(GoodKind::USD, 0.0))));
        goods.push(Rc::new(RefCell::new(Good::new(GoodKind::YEN, 0.0))));
        goods.push(Rc::new(RefCell::new(Good::new(GoodKind::YUAN, 0.0))));

        //return the new trader
        Trader_SA {
            name: "Trader_SA".to_string(),
            cash,
            goods,
            bfb,
            rcnz,
            zse
        }
    }

    //GETTER METHODS:

    //get trader name
    fn get_trader_name(&self) -> String{
        self.name.clone()
    }

    //get trader cash at its disposal
    fn get_trader_cash(&self) -> f32{
        self.cash
    }

    //get trader goods at its disposal
    fn get_trader_goods(&self) -> Vec<Rc<RefCell<Good>>>{
        self.goods.clone()
    }

    //get the quantity of a certain good, EUR included
    fn get_trader_goodquantity(&self, goodkind: GoodKind) -> f32{
        match goodkind{
            GoodKind::EUR => {
                self.get_trader_cash()
            }

            _ => {
                let mut result = 0.0;
                for good in &self.goods{
                    if good.borrow().get_kind() == goodkind {
                        result = good.borrow().get_qty();
                    }
                }
                result
            }
        }
    }

    //OTHER METHODS USED FOR STRATEGY:

    //function for finding how much of a product i can buy from a market given euros at disposal
    pub fn how_much_buy(&self, market: &Rc<RefCell<dyn Market>>) -> (f32, GoodKind){
        let mut best_quantity = 0.0;
        let mut best_kind = GoodKind::USD;
        let mut lowest_price = -1.0;
        let mut buy_price = 0.0;

        let mut temp_best_qty = 0.0;

        for good in &self.goods{

            for market_good in market.borrow().get_goods(){
                if good.borrow().get_kind() == market_good.good_kind{
                    temp_best_qty = market_good.quantity;
                }
            }

            if temp_best_qty > 0.0{
                buy_price = market.borrow()
                    .get_buy_price(good.borrow().get_kind(), temp_best_qty)
                    .expect("Error in get_buy_price in the max_buy_quantity function");
                while self.cash < buy_price && temp_best_qty > 0.0 {
                    temp_best_qty = temp_best_qty * 0.9;
                    buy_price = market.borrow()
                        .get_buy_price(good.borrow().get_kind(), temp_best_qty)
                        .expect("Error in get_buy_price in the max_buy_quantity function");
                }
            }

            if (lowest_price > buy_price) || (lowest_price < 0.0){
                lowest_price = buy_price;
                best_quantity = temp_best_qty;
                best_kind = good.borrow().get_kind();
            }

        }

        (best_quantity, best_kind)

    }

    pub fn how_much_sell(&self, market: &Rc<RefCell<dyn Market>>, goodkind: GoodKind) -> f32{
        let mut sell_price = 0.0;

        let mut eur_qty = 0.0;

        for market_good in market.borrow().get_goods(){
            if market_good.good_kind == GoodKind::EUR{
                eur_qty = market_good.quantity;
            }
        }

        let mut best_quantity = self.get_trader_goodquantity(goodkind);

        if best_quantity > 0.0{
            sell_price = market.borrow()
                .get_sell_price(goodkind, best_quantity)
                .expect("Error in get_buy_price in the max_buy_quantity function");
            while eur_qty < sell_price && best_quantity > 0.0 {
                best_quantity = best_quantity * 0.9;
                sell_price = market.borrow()
                    .get_sell_price(goodkind, best_quantity)
                    .expect("Error in get_buy_price in the max_buy_quantity function");
            }
        }

        best_quantity
    }


    // BOT METHOD: apply bot strategy for i loop interactions
    //and returns the string with all the actions done by the trader
    pub fn strategy(&mut self, mut i: i32) -> Vec<String>{
        let mut result: Vec<String> = vec![];

        loop {
            //loop until i reaches 0
            if i <= 0{
                break;
            }

            //first buy other goods using all the euros at our disposal
            //we need to find the cheapest buy price for a certain good
            //in order to do so, we need to do a research among all the markets
            //and their prices for each good, and see where we can get the most
            //from them

            let (best_quantity_bfb, kind_quantity_bfb) = self.how_much_buy(&self.bfb);
            let (best_quantity_rcnz, kind_quantity_rcnz) = self.how_much_buy(&self.rcnz);
            let (best_quantity_zse, kind_quantity_zse) = self.how_much_buy(&self.zse);

            let bfb = self.bfb.borrow_mut().get_buy_price(kind_quantity_bfb, best_quantity_bfb);
            let rcnz = self.rcnz.borrow_mut().get_buy_price(kind_quantity_rcnz, best_quantity_rcnz);
            let zse = self.zse.borrow_mut().get_buy_price(kind_quantity_zse, best_quantity_zse);

            let mut min = bfb.unwrap();
            let mut best_market = &self.bfb;
            let mut best_kind = kind_quantity_bfb;
            let mut best_qty = best_quantity_bfb;

            match rcnz{
                Ok(_) => {
                    if min > rcnz.clone().unwrap(){
                        min = rcnz.unwrap();
                        best_market = &self.rcnz;
                        best_kind = kind_quantity_rcnz;
                        best_qty = best_quantity_rcnz;
                    }
                }
                Err(_) => {}
            };

            match zse{
                Ok(_) => {
                    if min > zse.clone().unwrap(){
                        min = zse.unwrap();
                        best_market = &self.zse;
                        best_kind = kind_quantity_zse;
                        best_qty = best_quantity_zse;
                    }}
                Err(_) => {}
            };

            let (best_market, best_kind, best_quantity) = (best_market, best_kind, best_qty);



            //once we have found what is the best buy trade we can make, start to do it
            //first we define the price of the trade inside the strategy function
            let price = match best_market.borrow().get_buy_price(best_kind, best_quantity) {
                Ok(price) => price,
                Err(e) => {
                    panic!(
                        "Error in get_buy_price in {}: {:?}",
                        best_market.borrow().get_name(),
                        e
                    );
                }
            };

            //now we start with locking the good and getting the token for the buy function
            let mut cash = Good::new(GoodKind::EUR, price);
            let token =
                match best_market
                    .borrow_mut()
                    .lock_buy(best_kind, best_quantity, price, self.get_trader_name())
                {
                    Ok(token) => token,
                    Err(e) => {
                        panic!("Error in lock_buy in {}: {:?}", best_market.borrow().get_name(), e);
                    }
                };

            //use the token to buy the good
            let increase = match best_market.borrow_mut().buy(token, &mut cash) {
                Ok(increase) => increase,
                Err(e) => {
                    panic!("Error in buy in {}: {:?}", best_market.borrow().get_name(), e);
                }
            };

            //now that we have bought the good from the market, now we have to change
            //the quantities inside the trader
            self.cash -= price;
            for good in self.goods.iter_mut() {
                if good.borrow().get_kind() == best_kind {
                    match good.borrow_mut().merge(increase.clone()) {
                        Ok(_) => (),
                        Err(e) => println!("Error in merge {:?}", e),
                    }
                }
            }

            result.push(format!(
                "Buy GoodKind: {} Quantity: {} Market Name: {}",
                best_kind.to_string(),
                best_quantity.to_string(),
                best_market.borrow().get_name()
            ));

            //once we have used all our euros, we need to sell the good we just bought
            // at the highest price in order to get profit

            //let (best_market_sell, best_quantity_sell) = &self.find_best_sell_price(best_kind.clone());


            let best_quantity_bfb_sell = self.how_much_sell(&self.bfb, best_kind.clone());
            let best_quantity_rcnz_sell= self.how_much_sell(&self.rcnz, best_kind.clone());
            let best_quantity_zse_sell = self.how_much_sell(&self.zse, best_kind.clone());

            let bfb = self.bfb.borrow_mut().get_sell_price(best_kind, best_quantity_bfb_sell);
            let rcnz = self.rcnz.borrow_mut().get_sell_price(best_kind, best_quantity_rcnz_sell);
            let zse = self.zse.borrow_mut().get_sell_price(best_kind, best_quantity_zse_sell);

            let mut max = bfb.unwrap();
            let mut best_market = &self.bfb;
            let mut best_kind = kind_quantity_bfb;
            let mut best_qty_sell = best_quantity_bfb_sell;

            match rcnz{
                Ok(_) => {
                    if max < rcnz.clone().unwrap(){
                        max = rcnz.unwrap();
                        best_market = &self.rcnz;
                        best_qty_sell = best_quantity_rcnz_sell;
                    }
                }
                Err(_) => {}
            };

            match zse{
                Ok(_) => {
                    if max < zse.clone().unwrap(){
                        max = zse.unwrap();
                        best_market = &self.zse;
                        best_qty_sell = best_quantity_zse_sell;

                    }}
                Err(_) => {}
            };


            let (best_market_sell, best_quantity_sell) = (best_market, best_qty_sell);

            let market_name = best_market_sell.borrow().get_name();

            //we repeat the same procedure we did for the buy part, but now we consider variables for selling
            let price_sell = match best_market_sell.borrow().get_sell_price(best_kind, best_quantity_sell) {
                Ok(price_sell) => price_sell,
                Err(e) => {
                    panic!(
                        "Error in get_sell_price in {}: {:?}",
                        best_market_sell.borrow().get_name(),
                        e
                    );
                }
            };

            //get the token from lock_sell
            let token_sell = match best_market_sell
                .borrow_mut()
                .lock_sell(best_kind, best_quantity_sell, price_sell, self.get_trader_name())
            {
                Ok(token_sell) => token_sell,
                Err(e) => {
                    panic!("Error in lock_sell: {:?} in {:?}", e, market_name);
                }
            };

            //get the cash from the market
            let mut sold_good = Good::new(best_kind, best_quantity_sell);
            let increase_eur = match best_market_sell.borrow_mut().sell(token_sell, &mut sold_good) {
                Ok(increase_eur) => increase_eur,
                Err(e) => {
                    panic!("Error in sell in {}: {:?}", best_market_sell.borrow().get_name(), e);
                }
            };

            self.cash += increase_eur.get_qty();
            for good in self.goods.iter_mut() {
                if good.borrow().get_kind() == best_kind {
                    match good.borrow_mut().split(best_quantity_sell) {
                        Ok(_) => (),
                        Err(e) => panic!("Error in split {:?}, best_qty: {}", e, best_quantity_sell ),
                    }
                }
            }
            result.push(format!(
                "Sell GoodKind: {} Quantity: {} Market Name: {}",
                best_kind.to_string(),
                best_quantity_sell.to_string(),
                best_market_sell.borrow().get_name()
            ));

            i -= 1;
        }

        result.push(
            format!("Trader goods: EUR: {}, USD: {}, YEN: {}, YUAN: {}",
                  self.cash,
                  self.get_trader_goodquantity(GoodKind::USD),
                  self.get_trader_goodquantity(GoodKind::YEN),
                  self.get_trader_goodquantity(GoodKind::YUAN)));

        result
    }
}

