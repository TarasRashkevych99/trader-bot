mod tr_traders;
mod sa_traders;
mod common;


use crate::common::markets::{new_random, new_with_quantities, print_markets, print_results};
use crate::sa_traders::sa_trader_1::{Trader_SA};


fn main() {
    let client = reqwest::Client::new();

    let _trader_name = "RAST".to_string();

    // the random initialization of the markets
    let (bfb, rcnz, zse) = new_random();

    print_markets("Markets with random quantities", &bfb, &rcnz, &zse);

    // the initialization of the markets with the fixed quantity
    //let (bfb, rcnz, zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);

    print_markets("Markets with fixed quantities", &bfb, &rcnz, &zse);

    let mut trader_sa = Trader_SA::new(_trader_name,1000.0, bfb.clone(), rcnz.clone(), zse.clone());

    trader_sa.strategy(10);

    //println!("{:?}", trader_sa.register);

    //print_results(result);

    for i in trader_sa.register.iter(){
        println!("{:?}",i);
    }


    print_markets("Markets after with fixed quantities", &bfb, &rcnz, &zse);

    //let _res = client.post("http://localhost:8000/log").json(&craft_log_event(&event, "BFB".to_string(), true, None)).send().await;


}
