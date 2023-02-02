mod tr_traders;
mod sa_traders;
mod common;


use crate::common::markets::{new_random, new_with_quantities, print_results};
use crate::sa_traders::sa_trader_1::{Trader_SA};


fn main() {
    let _trader_name = "RAST".to_string();

    // the random initialization of the markets
    let (bfb, rcnz, zse) = new_random();

    print_results("Markets with random quantities", &bfb, &rcnz, &zse);

    // the initialization of the markets with the fixed quantity
    //let (bfb, rcnz, zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);

    print_results("Markets with fixed quantities", &bfb, &rcnz, &zse);

    let mut trader_sa = Trader_SA::new(10000.0, bfb.clone(), rcnz.clone(), zse.clone());

    let result = trader_sa.strategy(3);

    println!("{:?}", result);

    print_results("Markets after with fixed quantities", &bfb, &rcnz, &zse);

}
