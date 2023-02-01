use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
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

    pub fn trade_with_one_market(&self, market: &mut ChosenMarket) {
        // loop {
        //     let result = market.borrow_mut().buy(&mut Good::new(GoodKind::EUR, 10.0));
        //     match result {
        //         Ok(token) => {
        //             println!("Transaction successful");
        //             market.borrow().sell(token, &mut Good::new(GoodKind::EUR, 10.0));
        //         }
        //         Err(e) => {
        //             println!("Transaction failed");
        //         }
        //     }
        // }
    }

    pub fn trade_with_all_markets(&self, bfb: &mut ChosenMarket, rcnz: &mut ChosenMarket, zse: &mut ChosenMarket) {
        loop {
            // trader logic
        }
    }
}