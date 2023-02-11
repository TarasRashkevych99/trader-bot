mod tr_traders;
mod sa_traders;
mod common;


use crate::common::markets::{new_random, new_with_quantities, print_markets, print_results};
use crate::sa_traders::sa_trader_1::{Trader_SA};
use crate::tr_traders::tr_trader::Trader_TR;


fn main() {
    let (mut bfb, mut rcnz, mut zse) = new_random();

    // ==================== TR ====================
    let mut trader_tr = Trader_TR::new("RAST".to_string(), 10000.0);

    trader_tr.print_wallet_per_kind();
    trader_tr.print_register();
    print_markets("Markets with random quantities", &bfb, &rcnz, &zse);

    trader_tr.trade_with_all_markets(&mut bfb, &mut rcnz, &mut zse, 100);

    trader_tr.print_wallet_per_kind();
    trader_tr.print_register();
    print_markets("Markets with random quantities", &bfb, &rcnz, &zse);



    // ==================== SA ====================
    let mut trader_sa = Trader_SA::new(10000.0, bfb.clone(), rcnz.clone(), zse.clone());

    let result = trader_sa.strategy(3);

    print_results(result);

    print_markets("Markets after with fixed quantities", &bfb, &rcnz, &zse);

}
