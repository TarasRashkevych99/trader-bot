use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::market::Market;
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use ZSE::market::ZSE;

// function for generating the three markets with random good quantities
pub fn new_random() -> (Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>) {
    (BFB::new_random(),
     RCNZ::new_random(),
     ZSE::new_random())
}

// function for generating the three markets with fixed good quantities
pub fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> (Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>, Rc<RefCell<dyn Market>>) {
    (BFB::new_with_quantities(eur, yen, usd, yuan),
     RCNZ::new_with_quantities(eur, yen, usd, yuan),
     ZSE::new_with_quantities(eur, yen, usd, yuan))
}

pub fn print_markets(title: &str, bfb: &Rc<RefCell<dyn Market>>, rcnz: &Rc<RefCell<dyn Market>>, zse: &Rc<RefCell<dyn Market>>) {
    println!("{}", title);
    println!("BFB:");
    print_quantities(&bfb);
    println!("RCNZ:");
    print_quantities(&rcnz);
    println!("ZSE:");
    print_quantities(&zse);
    println!(" ");
}

// function that prints the quantities of each good for each market
fn print_quantities(market: &Rc<RefCell<dyn Market>>) {
    let market = market.borrow_mut();
    let goods = market.get_goods();

    //print the quantities in good labels
    for i in 0..goods.len() {
        println!("{:?}", goods[i]);
    }
}
