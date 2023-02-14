
use std::cell::{RefCell, RefMut};
use std::cmp::Ordering;
use std::collections::{HashSet, HashMap, hash_map};
use std::f32::consts::PI;
use std::ops::Rem;
use std::rc::Rc;

use unitn_market_2022::event::event::EventKind;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::good::good_kind::GoodKind::{USD,YEN,YUAN,EUR};
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::{Market, MarketGetterError, SellError, LockSellError, BuyError, LockBuyError};
use bfb::bfb_market::Bfb as BFB;
use RCNZ::RCNZ;
use ZSE::market::ZSE;
use unitn_market_2022::wait_one_day;
use crate::common::visualizer::{craft_log_event, CustomEventKind, LogEvent, wait_before_calling_api, TraderGood};
use futures::executor::block_on;
use tokio::runtime::Runtime;
use serde::{Deserialize, Serialize};
use crate::common;


type MarketRef=Rc<RefCell<dyn Market>>;
#[derive(Clone)]
pub struct Trader{
    name: String,
    ttl: u32,
    cash: f32,
    goods: Rc<RefCell<HashMap<GoodKind,Good>>>,
    markets: HashMap<String,Rc<RefCell<dyn Market>>>,
    lock_sells: HashMap<GoodKind,Vec<String>>,
    lock_purchases: HashMap<GoodKind,Vec<String>>,
}

impl Trader{
    pub fn new(name:String,ttl:u32,cash:f32,markets: Vec<Rc<RefCell<dyn Market>>>)->Self{
        Self{
            name,
            ttl,
            cash,
            goods: Rc::new(RefCell::new(HashMap::from([
                // (EUR,Good::new(EUR, 0.0)),
                (USD,Good::new(USD,0.0)),
                (YEN,Good::new(YEN,0.0)),
                (YUAN,Good::new(YUAN,0.0)),
            ]))),
            markets: markets.into_iter().map(|m| (String::from(m.borrow().get_name()),m.clone())).collect(),
            lock_sells: HashMap::from([(USD,vec![]),(YEN,vec![]),(YUAN,vec![])]),
            lock_purchases: HashMap::from([(USD,vec![]),(YEN,vec![]),(YUAN,vec![])]),
        }
    }
    pub fn best_sell(&self,gk:GoodKind)->Option<(String, Rc<RefCell<dyn Market>>)>{
        match self.markets.iter().max_by(|(_,x),(_,y)| match (x.borrow().get_sell_price(gk, 1.0),y.borrow().get_sell_price(gk, 1.0)){
            (Ok(a),Ok(b))=>a.total_cmp(&b),
            _=>Ordering::Greater
        }){
            Some((s,rc))=>Some((s.clone(),rc.clone())),
            _=>None
        }
    }
    pub fn best_buy(&self,gk:GoodKind)->Option<(String, Rc<RefCell<dyn Market>>)>{
        match self.markets.iter().min_by(|(_,x),(_,y)| match (x.borrow().get_buy_price(gk, 1.0),y.borrow().get_buy_price(gk, 1.0)){
            (Ok(a),Ok(b))=>a.total_cmp(&b),
            (_,Ok(_))=>Ordering::Less,
            _=>Ordering::Greater
        }){
            Some((s,rc))=>Some((s.clone(),rc.clone())),
            _=>None
        }
    }
    pub fn get_best_buy(&self)->Option<(String, GoodLabel)>{
        let labels:Vec<(String,Vec<GoodLabel>)>=self.markets.iter().map(|(s,m)| (String::from(s),m.borrow().get_goods())).collect();
        let mut bestsells:Vec<(String,GoodLabel)>=vec![];
        for (market,l) in labels.into_iter(){
            match l.into_iter().max_by(|x,y| x.quantity.total_cmp(&y.quantity)){
                Some(a)=>if a.good_kind!=EUR{bestsells.push((market.clone(),a))}else{()},
                _=>()
            }
        }
        bestsells.into_iter().min_by(|(_,x),(_,y)| x.quantity.total_cmp(&y.quantity))
    }
    pub fn get_qty(&self,k: GoodKind,qty: f32,price: f32,e: EventKind)->Option<f32>{
        match e{
            // the value ever remains under the half of the amount of sold goods and it is modulated according to the quantity
            // of the good that the trader is owning during the lock buy
            EventKind::LockedBuy=>{
                Some((price.abs()/
                match self.lock_sells.get(&k){
                    Some(a)=>a.len() as f32,
                    None=>1.0
                }).atan()*self.cash/PI)
            },
            // the trader can sell a quantity
            EventKind::LockedSell=>{
                Some((price/
                match self.lock_purchases.get(&k){
                    Some(a)=>a.len() as f32,
                    None=>1.
                }).atan()*2.*qty/PI)
            },
            _=>None
        }
    }

    pub fn lock_sell(&mut self, m: Rc<RefCell<dyn Market>>,k: GoodKind,qty: f32,offer:f32)->Result<(f32,f32,String), unitn_market_2022::market::LockSellError>{
        match m.borrow_mut().lock_sell(k, qty, offer, self.name.clone()){
            Ok(t)=>{
                match self.lock_sells.get_mut(&k){
                    Some(a)=>a.push(t.clone()),
                    None=>println!("not found {:?}",k)
                };
                Ok((qty,offer,t))
            },
            Err(lse)=>Err(lse)
        }
    }
    // pub fn notify()
    pub fn lock_buy(&mut self, m: Rc<RefCell<dyn Market>>,k: GoodKind,qty: f32,price:f32)->Result<(f32,f32,String), unitn_market_2022::market::LockBuyError>{
        match m.borrow_mut().lock_buy(k, qty, price, self.name.clone()){
            Ok(t)=>{
                match self.lock_purchases.get_mut(&k){
                    Some(a)=>a.push(t.clone()),
                    None=>println!("not found {:?}",k)
                }
                Ok((qty,price,t))
            },
            Err(lbe)=>Err(lbe)
        }
    }
    pub fn buy(&mut self,m: Rc<RefCell<dyn Market>>,token: String,qty:f32)->Result<Good, BuyError>{
        m.borrow_mut().buy(token, &mut Good::new(DEFAULT_GOOD_KIND, qty))
    }
    pub fn sell(&mut self,m: Rc<RefCell<dyn Market>>,token: String,k:GoodKind,qty:f32)->Result<Good, SellError>{
        m.borrow_mut().sell(token, &mut Good::new(k, qty))
    }
    pub async fn log(&self,client: &reqwest::Client,time:&mut u32, kind:CustomEventKind, good_kind:GoodKind, quantity:f32, price:f32, market:String,result:bool,error:Option<String>){
        wait_before_calling_api(Self::get_delay(client).await);
        let _res=client.post("http://localhost:8000/log").json(&craft_log_event(*time, kind, good_kind, quantity, price, if market=="Baku stock exchange".to_string(){ "BFB".to_string() }else{market}, result, error)).send().await;
        *time+=1;
    }
    pub async fn post_currentGoodLabels(&self,client:&reqwest::Client){
        wait_before_calling_api(Self::get_delay(client).await);
        for (k,m) in self.markets.iter(){
            let _res=client
                .post("http://localhost:8000/currentGoodLabels/".to_string()+if k=="Baku stock exchange"{ "BFB" }else{&k})
                .json(&m.borrow().get_goods()).send().await;
        }
    }
    pub async fn post_traderGoods(&self,client: &reqwest::Client){
        let mut body:Vec<TraderGood>=self.goods
                     .borrow()
                     .iter()
                     .map(|(kind,good)| TraderGood{ kind:*kind, quantity:good.get_qty()})
                     .collect();
        body.push(TraderGood { kind: GoodKind::EUR, quantity: self.cash });
        wait_before_calling_api(Self::get_delay(client).await);
        let _res=client.post("http://localhost:8000/traderGoods").json(&body).send().await;
    }
    pub async fn get_delay(client: &reqwest::Client)->u64{
        client.get("http://localhost:8000/delay").send().await.unwrap().json::<u64>().await.unwrap()
    }
    pub fn trade(&mut self,client: reqwest::Client){
        let r=Runtime::new().unwrap();
        let mut last_lock:Option<(EventKind,GoodKind)>=None;
        r.block_on(self.post_traderGoods(&client));
        r.block_on(self.post_currentGoodLabels(&client));
        for mut i in 0..self.ttl{
            match last_lock.clone(){
                // tries to resell the just bought good
                Some((EventKind::Bought,k))=>{
                    // let k=gl_sell.good_kind;
                    // let market=self.markets.get(&m_sell).unwrap().clone();
                    let Some((m_sell,market))=self.best_sell(k) else { todo!() };
                    let Ok(rate_sell)=market.borrow().get_sell_price(k, 1.) else { todo!() };
                    let Some(qty)=self.get_qty(k,match self.goods.borrow().get(&k){Some(g)=>g.get_qty(),_=>todo!()}, rate_sell, EventKind::LockedSell) else { todo!() };
                    r.block_on(self.post_traderGoods(&client));
                    r.block_on(self.post_currentGoodLabels(&client));
                    let Ok(offer)=market.borrow().get_sell_price(k, qty) else { todo!() };
                    match self.lock_sell(market.clone(), k, qty,offer){
                        Ok((qty,offer,token))=>{
                            r.block_on(self.log(&client,&mut i, CustomEventKind::LockedSell, k, qty, offer, m_sell.clone(), true, None));
                            r.block_on(self.post_traderGoods(&client));
                            r.block_on(self.post_currentGoodLabels(&client));
                            match self.sell(market.clone(), token.clone(),k, qty){
                                Ok(good)=>{
                                    self.cash+=offer;
                                    self.lock_purchases.clear();
                                    r.block_on(self.log(&client, &mut i, CustomEventKind::Sold, k, qty, offer, m_sell.clone(), true, None));
                                    r.block_on(self.post_traderGoods(&client));
                                    r.block_on(self.post_currentGoodLabels(&client));
                                    last_lock=Some((EventKind::Sold,k));
                                },
                                Err(se)=>{
                                    r.block_on(self.log(&client, &mut i, CustomEventKind::Sold, k, qty, offer, m_sell, false, Some(format!("{:?}",se))));
                                }
                            }
                        },
                        Err(LockSellError::InsufficientDefaultGoodQuantityAvailable{
                            offered_good_kind, offered_good_quantity, available_good_quantity
                        })=>{
                            r.block_on(self.log(&client, &mut i, CustomEventKind::LockedSell, k, qty, offer, m_sell.clone(), false, Some(format!("{:?}",LockSellError::InsufficientDefaultGoodQuantityAvailable{
                                offered_good_kind, offered_good_quantity, available_good_quantity
                            }))));
                            self.markets.remove(&m_sell);
                        },
                        Err(lse)=>{
                            r.block_on(self.log(&client, &mut i, CustomEventKind::LockedSell, k, qty, offer,m_sell, false, Some(format!("{:?}",lse))));
                            last_lock=None;
                        }
                    }
                },
                a=>{
                    let (m_buy,market,rate_buy,qty,k)=match a{
                        Some((EventKind::Sold,k))=>{
                            last_lock=None;
                            let Some((m_buy,market))=self.best_buy(k) else { todo!() };
                            let Ok(rate_buy)=market.borrow().get_buy_price(k, 1.) else { todo!() };
                            let goods=market.borrow().get_goods();
                            let Some(gl)=goods.iter().find(|gl| gl.good_kind==k) else { todo!() };
                            (m_buy,market,rate_buy,gl.quantity,k)
                        },
                        _=>{
                            let Some((m_buy,gl))=self.get_best_buy() else { return; };
                            let market=self.markets.get(&m_buy).unwrap().clone();
                            last_lock=Some((EventKind::Bought,gl.good_kind));
                            (m_buy.clone(),market,gl.exchange_rate_buy,gl.quantity,gl.good_kind)
                        }
                    };
                    match self.get_qty(k, qty, rate_buy, EventKind::LockedBuy){
                        Some(qty)=>{
                            let price=match market.borrow().get_buy_price(k, qty){ Ok(price)=>price,Err(e)=>{println!("{:?}",e); todo!()} };
                            match self.lock_buy(market.clone(), k, qty,price){
                                Ok((qty,price,token))=>{
                                    r.block_on(self.log(&client,&mut i, CustomEventKind::LockedBuy, k, qty, price, m_buy.clone(), true, None));
                                    r.block_on(self.post_traderGoods(&client));
                                    r.block_on(self.post_currentGoodLabels(&client));
                                    match self.buy(market, token.clone(), if m_buy=="ZSE".to_string(){ price }else {qty} ){
                                        Ok(good)=>{
                                            self.cash-=price;
                                            let _=self.goods.borrow_mut().get_mut(&k).unwrap().merge(good);
                                            self.lock_sells.clear();
                                            r.block_on(self.log(&client, &mut i, CustomEventKind::Bought, k, qty, price, m_buy.clone(), true, None));
                                            r.block_on(self.post_traderGoods(&client));
                                            r.block_on(self.post_currentGoodLabels(&client));
                                        },
                                        Err(BuyError::InsufficientGoodQuantity { contained_quantity, pre_agreed_quantity })=>{
                                            r.block_on(self.log(&client, &mut i, CustomEventKind::Bought, k, qty, price, m_buy.to_string(), false, Some(format!("{:?}",BuyError::InsufficientGoodQuantity { contained_quantity, pre_agreed_quantity }))));
                                            last_lock=Some((EventKind::Bought,k))
                                        }
                                        Err(be)=>{
                                            r.block_on(self.log(&client, &mut i, CustomEventKind::Bought, k, qty, price, m_buy.to_string(), false, Some(format!("{:?}",be))));
                                            last_lock=Some((EventKind::Bought,k));
                                        }
                                    }
                                },
                                Err(LockBuyError::MaxAllowedLocksReached)=>{
                                    r.block_on(self.log(&client, &mut i, CustomEventKind::LockedBuy, k, qty, price, m_buy.to_string(), false, Some(format!("{:?}",LockBuyError::MaxAllowedLocksReached))));
                                    self.markets.remove(&m_buy);
                                    i+=1;
                                },
                                Err(lbe)=>{
                                    r.block_on(self.log(&client, &mut i, CustomEventKind::LockedBuy, k, qty, price, m_buy.to_string(), false, Some(format!("{:?}",lbe))));
                                    last_lock=None;
                                    panic!();
                                    i+=1;
                                }
                            }
                        },
                        None=>{todo!()}
                    }
                }
            }
        }
    }
}
