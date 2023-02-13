# Trader-bot project 
Trader project that contains trader bots that interact with different markets developed for Advanced Programming course project written in Rust. Each one of these traders implements a strategy used to interact with the markets in order to buy and sell goods in a more or less efficient way.

## Index
* Traders
  * Buy low - sell high bot (Sabin Andone)
  * Trader number 2 
  * Trader number 3
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

Now we check if a buy operation is possible. More specifically, we check the quantity that we want to buy (if it is too low, we don't buy, but just wait), if at least one buy operation is possible (by checking if the minimum price is a reasonable number, i.e. not f32::MAX) and that the buy operation doesn't take too much from the trader's budget. 
            
```
 if best_quantity > 1.0 && min_buy_price < f32::MAX
```

If all checks, are alright, then we proceed by taking the three variables (the best kind of good, the best buy quantity and the best market to buy the good from) that we have obtained previously and do the lock_buy() and buy() functions. We execute these two functions in the following way:
            
```
 //do the lock_buy
 let delay = rt.block_on(self.get_delay_in_milliseconds());
 wait_before_calling_api(delay);
 let token = rt.block_on(self.lock_buy_from_market(market_name, best_kind, best_quantity, price, self.get_trader_name()));

 if let Ok(token) = token{
     //buy
     rt.block_on(self.send_labels());
     rt.block_on(self.send_trader_goods());
     let delay = rt.block_on(self.get_delay_in_milliseconds());
     wait_before_calling_api(delay);
     rt.block_on(self.buy_from_market(market_name, best_kind, best_quantity, price, token));
     rt.block_on(self.send_labels());
     rt.block_on(self.send_trader_goods());
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
 continue;            
```
             
The selling part follows an approach similar to the buying part, except for the fact that now we have to sell the good the trader just bought and we sell it at the highest price possible, by computing the highest quantity of EURs possibly attainable from all markets and we try to sell the highest amount of that good in order to get the highest amount of EURs possible. Repeat again for i interactions to gain more profit. 


## Trader number 2 (Taras Rashkevych)

## Trader number 3 (Alfredo Bombace)

## Data Visualizer (Roberto Cornacchiari)
All the details regarding the data visualizer are located into this [link](https://github.com/RobertoCornacchiari/DataVisualizer)
