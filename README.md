# Trader-bot project 
Trader project that contains trader bots that interact with different markets developed for Advanced Programming course project written in Rust. Each one of these traders implements a strategy used to interact with the markets in order to buy and sell goods in a more or less efficient way.

## Index
* Traders
  * Buy low - sell high bot (Sabin Andone)
  * Trader number 2 (Taras Rashkevych)
  * Trader number 3 (Alfredo Bombace)
* Data visualizer (Roberto Cornacchiari)

## Buy low - sell high bot (Sabin Andone)
The trader bot which will be discussed in this section adopts a simple yet effective approach to commercialize goods with the market this bot interacts with. The strategy consists of buying goods at the lowest price possible from all markets, and then selling the goods we just bought at the highest price possible, in order to get the highest amount of EURs possible (i.e. get profit).   

It works as following: first, since this bot needs to communicate its data to the Data Visualizer, we need to do an initial call to send its initial data, necessary to see tha starting point of our simulation. To do so, we also define the Runtime variable, as in the following case:

```
 let rt  = Runtime::new().unwrap();
 rt.block_on(self.send_labels());
 rt.block_on(self.send_trader_goods());
```

The send_labels() and send_trader_goods() functions are two functions that we use to do calls on the API for sending, respectively the GoodLabels of each market and the list of goods the market has at that moment in the form of Vec<TraderGood>, where TraderGood is a struct of form:
 
```
 struct TraderGood{
    kind: GoodKind,
    quantity: f32
 }
```

Then we start with the loop. At each interaction, we first check if the number interactions to do are over or not, and if not then continue to the next step. Then, it is necessary to find the best price at which we can buy goods using trader's EURs at its disposal. To do so, we call the find_best_buy_quantity() function on each of the three markets. 

```
 pub fn find_best_buy_quantity(&self, market: &Rc<RefCell<dyn Market>>) -> (f32, GoodKind) {
        let mut best_quantity = 0.0;
        let mut best_kind = GoodKind::USD;
        let mut lowest_price = -1.0;
        for good in &self.goods {
            let mut temp_best_qty = 0.0;
            for market_good in market.borrow().get_goods() {
                if good.borrow().get_kind() == market_good.good_kind {
                    temp_best_qty = market_good.quantity;
                }
            }
            let mut buy_price = f32::MAX;
            if temp_best_qty > 0.0 {
                buy_price = market.borrow().get_buy_price(good.borrow().get_kind(), temp_best_qty).expect("Error in find_best_buy_quantity function");
                while self.cash < buy_price && temp_best_qty > 0.01 {
                    temp_best_qty = temp_best_qty * 0.5;
                    buy_price = market.borrow().get_buy_price(good.borrow().get_kind(), temp_best_qty).expect("Error in find_best_buy_quantity function");
                }
            }
            if (lowest_price > buy_price) || (lowest_price < 0.0) {
                lowest_price = buy_price;
                best_quantity = temp_best_qty;
                best_kind = good.borrow().get_kind();
            }
        }
        (best_quantity, best_kind)
    }
```

This function computes which is the best good to buy and how much of that good we can buy from a given market and considering the trader's cash. For each one of these markets we get the best kind of good we can buy from that market and at which quantity we can buy that good with trader's money. Then we compute the prices for each market using the get_buy_price() method using as parameters the values we obtained in the find_best_buy_quantity() method. Now, we compare the prices of the three markets and we decide which one is the mininum. In this way we spend less and we can potentailly get more goods from the market the trader chooses to trade with. As a result we get the best kind of good, the best buy quantity and the best market to buy the good from.

Now we check if a buy operation is possible. More specifically, we check the quantity that we want to buy (if it is too low, we don't buy, but just wait), if at least one buy operation is possible (by checking if the minimum price is a reasonable number, i.e. not f32::MAX) and that the buy operation doesn't take too much from the trader's budget. For the last check, we consider as the max_budget the maximum budget that the trader got during the simulation. This because sometimes the trader reaches the highest budget value possible too early and then it starts losing euros, thus value from its budget. The 10% limit is there to prevent having trader's budget going too low.
            
```
 if best_quantity > 1.0 && min_buy_price < f32::MAX && (self.cash - min_buy_price >= max_budget * 0.10)
```

If all checks are fine, then we proceed by taking the three variables (the best kind of good, the best buy quantity and the best market to buy the good from) that we have obtained previously and do the lock_buy() and buy() functions. We execute these two functions in the following way:
            
```
 //do the lock_buy
 let delay = rt.block_on(self.get_delay_in_milliseconds());
 wait_before_calling_api(delay);
 let token = rt.block_on(self.lock_buy_from_market(market_name, best_kind, best_quantity, price, self.get_trader_name()));

 if let Ok(token) = token{
     rt.block_on(self.send_labels());
     rt.block_on(self.send_trader_goods());
     //loop until i is reached
     if i < self.time {
          break;
     }
     let delay = rt.block_on(self.get_delay_in_milliseconds());
     wait_before_calling_api(delay);
     //buy
     rt.block_on(self.buy_from_market(market_name, best_kind, best_quantity, price, token));
     rt.block_on(self.send_labels());
     rt.block_on(self.send_trader_goods());
     //loop until i is reached
     if i < self.time {
          break;
     }
 }else{
     continue;
 }
```

Before doing each operation we get the delay through an API call (see Data Visualizer section for more details). For the lock_buy function, we get the token and if we get no errors, then we can proceed to buy the good, otherwise skip the interaction and pass to the next one. When executing the buy() function, the good in return is saved in a variable used for the merge() function in order to increase the quantity of that good inside the trader's good list. Meanwhile, in this part we also reduce the EURs from the trader's budget, which means that now the trader has less EURs but it is available to sell that good to get profit.
             
If the checks that we have done for the if part result being false, then we need to wait a day and pass to the next interaction.           
        
```
 //wait
 let delay = rt.block_on(self.get_delay_in_milliseconds());
 wait_before_calling_api(delay);
 rt.block_on(self.wait(best_kind, 0.0, 0.0, market_name));
 rt.block_on(self.send_labels());
 rt.block_on(self.send_trader_goods());
 //loop until i is reached
 if i < self.time {
      break;
 }
 continue;          
```
             
The selling part follows an approach similar to the buying part, except for the fact that now we have to sell the good the trader just bought and we sell it at the highest price possible, by computing the highest quantity of EURs possibly attainable from all markets and we try to sell the highest amount of that good in order to get the highest amount of EURs possible. The operation for getting the best quantity for selling is similar to the one for the buy part:

```
 pub fn find_best_sell_quantity(&self, market: &Rc<RefCell<dyn Market>>, goodkind: GoodKind) -> f32 {
        let mut sell_price = 0.0;
        let mut eur_qty = 0.0;
        for market_good in market.borrow().get_goods() {
            if market_good.good_kind == GoodKind::EUR {
                eur_qty = market_good.quantity;
            }
        }
        let mut best_quantity = self.get_trader_goodquantity(goodkind);
        if best_quantity > 0.0 {
            sell_price = market.borrow().get_sell_price(goodkind, best_quantity).expect("Error in find_best_sell_quantity function");
            while eur_qty < sell_price && best_quantity > 0.1 {
                best_quantity = best_quantity * 0.5;
                sell_price = market.borrow().get_sell_price(goodkind, best_quantity).expect("Error in find_best_sell_quantity function");
            }
        }
        best_quantity
    }
```

Also, before proceed with lock_sell and sell operations, we need to check if the sell operation would make sense or not (i.e. we can get profit from it) we do the following check:

```
 if best_quantity_sell > 1.0 && max_sell_price > 0.0 && (self.cash + max_sell_price >= initial_budget * 0.9) {
```

Repeat again for i interactions to gain more profit. 

![esempio grafico_2](https://user-images.githubusercontent.com/58253647/218355172-92629eb3-0194-4c00-b270-6b93e30875d4.png)

Example of a chart that shows the cash flow of the trader during an instance of the simulation of the strategy. This chart has been created using our data visualizer.


## Trader number 2 (Taras Rashkevych)

## Trader number 3 (Alfredo Bombace)
> Trading of ab_trader::Trader starts with a call to method `trade(...)` over a newly instanced `&mut Self`.

Firstly, the trader designates the best market to which buy the most convenient good according to its `exchange_rate_buy` and `quantity` owned by that market; hence the trader, at the first step, will buy that good from the market which offers the cheapest price and the maximum availability. The quantity of bought good, and the quantity of each trader's transaction, is carried out by the `self.get_qty(...)` method which applies a self-stabilizing curve working as follows:
> The trigonometric curve is an arctangent with an horizontal asymptote to:
> - $\frac{3}{2}$ of the trader's capital ($c$), if the trader is buying;
> - the current quantity of good owned by the trader ($|g|:g\in G$), if the trader is selling.
> Moreover, in the former case the quantity decreases as the market minimum rate (`exchange_rate_buy`,$f_b$) increases with the number of locks for sells already bargained since the trader is alive (`self.lock_sells`,$|L_s|$). On the other hand, when the trader sells: the trader's offered quantity raises with the maximum rate at which the market accepts goods (`exchange_rate_sell`,$f_s$) times the quantity of buyouts already negotiated with other markets (`self.lock_buyouts`,$|L_b|$).

To sum up, the method performs:
$$
\begin{subequations}
\begin{flalign}
y=\frac{c}{3\pi}\tan^{-1}\left(\frac{1}{f_b|L_s|}\right)
\end{flalign}
\begin{flalign}
y=\frac{2|g|}{\pi}\tan^{-1}\left(f_s|L_b|\right)
\end{flalign}
\end{subequations}
$$
### Trading
At each iteration, the method matches optional content of local variable `last_lock`: it is a tuple of `EventKind` and `GoodKind` used to manage trader's decisions on the basis of the previous (last) concluded transaction:
- if the content is `None` then the trader will try to perform a buy transaction according to the most convenient parameters thanks to `get_best_buy` (newly designated market to perform the cheapest buyout)···
  - *after both the success and the failure due insufficient good quantity, `last_lock` will trigger a sell transaction for the same good (`Bought` event)*
  - *after the failure due locks limit, `last_lock`'s event is set to `Sold` in order to trigger a retry of buy without that market*
  - *after other failures, `last_lock` will trigger a fresh new cheapest market search (`None`)*
,
- if the content matches the `Bought` event, the trader will try to sell the specified good kind according to the quantity computed thanks to the above function···
  - *after success, the method will try to buy the same kind (`Sold` event)*
  - *after failure, the method will try to buy a fresh new cheapest kind (`None`)*
, 
- if the content matches the `Sold` event, along with the respective kind, the trader will try to purchase the already sold kind at a (hopefully) better price from either the same market or one of the others···
  - *trader will then try to perform a fresh new buy*
### Termination
The trade procedure terminates in the following cases:
- all the markets reached their max allowed lock quantity regarding trades with the current trader instance (`Err(LockBuyError::MaxAllowedLocksReached)`)

## Data Visualizer (Roberto Cornacchiari)
All the details regarding the data visualizer are located into this [link](https://github.com/RobertoCornacchiari/DataVisualizer)
