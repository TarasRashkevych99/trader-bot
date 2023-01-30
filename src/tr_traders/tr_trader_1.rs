use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::market::Market;

type ChosenMarket = Rc<RefCell<dyn Market>>;



pub fn trade_with(bfb: ChosenMarket, rcnz: ChosenMarket, zse: ChosenMarket) {
    loop {
        // trader logic
    }
}