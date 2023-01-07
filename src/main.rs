use bfb;
use rcnz_market;
use RCNZ;
use ZSE::market::ZSE;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::{Market, market_test, SellError};

fn main() {

    //just testing if everything is ok
    let market_1 = RCNZ::new_random();

    println!("Name is {}", market_1.borrow().get_name());

    let market_2 = ZSE::new_with_quantities(1.0, 1.0, 1.0, 1.0);

    println!("Name is {}", market_2.borrow().get_name());
}
