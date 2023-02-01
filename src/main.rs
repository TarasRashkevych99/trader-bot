mod tr_traders;
mod common;

use crate::common::markets::{new_random, new_with_quantities, print_results};
use crate::tr_traders::tr_trader_1::Trader;


fn main() {
    let mut trader = Trader::new("RAST".to_string());

    // the random initialization of the markets
    let (mut bfb, mut rcnz, mut zse) = new_random();

    trader.trade_with_one_market(&mut bfb);

    print_results("Markets with random quantities", &bfb, &rcnz, &zse);

    // the initialization of the markets with the fixed quantity
    // let (bfb, rcnz, zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);
    //
    // print_results("Markets with fixed quantities", &bfb, &rcnz, &zse);
}
