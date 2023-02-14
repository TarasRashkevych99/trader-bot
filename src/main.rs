mod tr_traders;
mod sa_traders;
mod common;


use crate::common::markets::{new_random, new_with_quantities, print_markets, print_results};
use crate::sa_traders::sa_trader_1::{Trader_SA};
use crate::tr_traders::tr_trader::Trader_TR;


fn main() {
    let trader_config = common::trader_config::get_trader_config();
    let (mut bfb, mut rcnz, mut zse) = new_random();

    if trader_config.is_trader_TR() {
        let mut trader_tr = Trader_TR::new("RAST".to_string(), trader_config.get_budget());

        trader_tr.print_wallet_per_kind();
        trader_tr.print_register();
        print_markets("Markets with random quantities", &bfb, &rcnz, &zse);

        trader_tr.trade_with_all_markets(&mut bfb, &mut rcnz, &mut zse, trader_config.get_trading_days());

        trader_tr.print_wallet_per_kind();
        trader_tr.print_register();
        print_markets("Markets with random quantities", &bfb, &rcnz, &zse);
    } else if trader_config.is_trader_SA() {
        let mut trader_sa = Trader_SA::new("RAST".to_string(), trader_config.get_budget(), bfb.clone(), rcnz.clone(), zse.clone());

        trader_sa.strategy(trader_config.get_trading_days());

        print_markets("Markets after with random quantities", &bfb, &rcnz, &zse);
    } else if trader_config.is_trader_AB() {
        let mut trader_ab=ab_traders::ab_traders::Trader::new("RAST".to_string(), trader_config.get_trading_days(), trader_config.get_budget(), vec![bfb.clone(),rcnz.clone(),zse.clone()]);
        trader_ab.trade(reqwest::Client::new());
        print_markets("Markets after with random quantities", &bfb, &rcnz, &zse);
    }
}
