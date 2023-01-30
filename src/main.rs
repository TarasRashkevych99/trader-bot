mod tr_traders;
mod common;

use crate::common::markets::{new_random, new_with_quantities, print_results};


fn main() {
    let _trader_name = "RAST".to_string();

    // the random initialization of the markets
    let (bfb, rcnz, zse) = new_random();

    print_results("Markets with random quantities", &bfb, &rcnz, &zse);

    // the initialization of the markets with the fixed quantity
    let (bfb, rcnz, zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);

    print_results("Markets with fixed quantities", &bfb, &rcnz, &zse);
}
