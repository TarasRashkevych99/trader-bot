use std::cell::RefCell;
use std::collections::{HashSet, HashMap};
use std::rc::Rc;

use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{USD,YEN,YUAN};
use unitn_market_2022::market::{Market, MarketGetterError};
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use ZSE::market::ZSE;

#[derive(Clone)]
pub struct Trader{
    name: String,
    ttl: usize,
    cash: f32,
    goods: Rc<RefCell<HashMap<GoodKind,Good>>>,
    markets: HashSet<Rc<RefCell<dyn Market>>>,
    locks: HashMap<GoodKind,Vec<String>>
}

impl Trader{
    pub fn new(name:String,ttl:usize,cash:f32,markets: HashSet<Rc<RefCell<dyn Market>>>)->Self{
        Self{
            name,
            ttl,
            cash,
            goods: Rc::new(RefCell::new(HashMap::from([
                (USD,Good::new(USD,0.0)),
                (YEN,Good::new(YEN,0.0)),
                (YUAN,Good::new(YUAN,0.0)),
            ]))),
            markets,
            locks: HashMap::from([(USD,vec![]),(YEN,vec![]),(YUAN,vec![])])
        }
    }
    pub fn get_best_markets(&self)->(HashMap<GoodKind,Rc<RefCell<dyn Market>>>,HashMap<GoodKind,Rc<RefCell<dyn Market>>>){
        let mut max_sell=HashMap::new();
        let mut min_buy=HashMap::new();
        // for each good I choose the best market at which buy and the best market in which I can sell
        for (k,good) in self.goods.borrow().iter(){
            match self.markets.iter().max_by(|x,y| match x.borrow().get_sell_price(good.get_kind(), 1.0){
                Ok(a)=>a.total_cmp(
                    match &y.borrow().get_sell_price(good.get_kind(), 1.0){
                        Ok(b)=>b,
                        _=>&0.0
                    }
                ),
                _=>(0.0 as f32).total_cmp(match &y.borrow().get_sell_price(good.get_kind(), 1.0){
                        Ok(b)=>b,
                        _=>&0.0
                    })
            }){
                Some(v)=>max_sell.insert(k.clone(), v.clone()),
                _=>None
            };
        };
        for (k,good) in self.goods.borrow().iter(){
            match self.markets.iter().max_by(|x,y| match x.borrow().get_sell_price(good.get_kind(), 1.0){
                Ok(a)=>a.total_cmp(
                    match &y.borrow().get_sell_price(good.get_kind(), 1.0){
                        Ok(b)=>b,
                        _=>&0.0
                    }
                ),
                _=>(0.0 as f32).total_cmp(match &y.borrow().get_sell_price(good.get_kind(), 1.0){
                        Ok(b)=>b,
                        _=>&0.0
                    })
            }){
                Some(v)=>min_buy.insert(k.clone(), v.clone()),
                _=>None
            };
        };
        (max_sell,min_buy)
    }
    pub fn trade(&mut self){
        let sell_rate=|f:| ;
        for i in 0..self.ttl{
            let (max_sell,min_buy)=self.get_best_markets();
            for (k,rc) in max_sell.iter(){
                match min_buy.get(k){
                    Some(a)=>{
                        a.borrow().
                    },
                    _=>,
                }
            }
        }
    }
}
