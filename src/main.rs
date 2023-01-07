use bfb::bfb_market::Bfb;
use RCNZ::RCNZ;
use ZSE::market::ZSE;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{Market, market_test, SellError};

fn main() {

    //just testing if everything is ok

    let market_1 = RCNZ::new_random();

    println!("Name is {}", market_1.borrow().get_name());

    let market_2 = ZSE::new_random();

    println!("Name is {}", market_2.borrow().get_name());

    let market_3 = Bfb::new_random();

    println!("Name is {}", market_3.borrow().get_name());
}
