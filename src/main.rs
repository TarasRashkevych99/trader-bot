mod tr_traders;

use std::cell::RefCell;
use std::rc::Rc;
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use ZSE::market::ZSE;
use unitn_market_2022::market::{Market};

// function for generating the three markets with random good quantities
// **Sabin Andone**
fn new_random() -> (Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>) {
    (BFB::new_random(),
     RCNZ::new_random(),
     ZSE::new_random())
}


// function for generating the three markets with fixed good quantities
// **Sabin Andone**
fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> (Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>) {
    (BFB::new_with_quantities(eur, yen, usd, yuan),
     RCNZ::new_with_quantities(eur, yen, usd, yuan),
     ZSE::new_with_quantities(eur, yen, usd, yuan))
}


// function that prints the quantities of each good for each market
// **Sabin Andone**
fn print_quantities(market: &Rc<RefCell<dyn Market>>) {

    let market = market.borrow_mut();
    let goods = market.get_goods();

    //print the quantities in good labels
    for i in 0..goods.len() {
        println!("{:?}",goods[i]);
    }
}






fn main() {

    //initialize the trader name
    let _trader_name = "RAST".to_string();

    // the random initialization of the markets
    let (mut bfb, mut rcnz, mut zse) = new_random();

    // print the quantities of goods for each market
    println!("Markets with random quantities");
    println!("BFB:");
    print_quantities(&bfb);
    println!("RCNZ:");
    print_quantities(&rcnz);
    println!("ZSE:");
    print_quantities(&zse);
    println!(" ");

    // the initialization of the markets with the fixed quantity
    (bfb, rcnz, zse) = new_with_quantities(100.0, 100.0, 100.0, 100.0);

    // print the value in good labels for each market
    println!("Markets with fixed quantities");
    println!("BFB:");
    print_quantities(&bfb);
    println!("RCNZ:");
    print_quantities(&rcnz);
    println!("ZSE:");
    print_quantities(&zse);
    println!(" ");
}
