# Trader-bot project 
Trader project that contains trader bots that interact with different markets developed for Advanced Programming course project written in Rust. Each one of these traders implements a strategy used to interact with the markets in order to buy and sell goods in a more or less efficient way.

## Index
* Traders
  * Buy low - sell high bot (Sabin Andone) (WIP)
  * Trader number 2 
  * Trader number 3
* Data visualizer (WIP)

## Buy low - sell high bot (Sabin Andone) (WIP)
The trader bot which will be discussed in this section adopts a simple yet effective approach to commercialize goods with the market this bot interacts with. The strategy consists of buying goods at the lowest price possible from all markets, and then selling the goods we just bought at the highest price possible, in order to get the highest amount of EURs possible (i.e. get profit).   

It works as following: in every i-th interaction, first it is necessary to find the best price at which we can buy goods using trader's EURs at its disposal. To do so, we call the how_much_buy() function on the three markets. This function computes which is the best good to buy and how much of that good from a given market we can buy considering the trader's cash. For each one of these markets we get the best kind of good we can buy from that market and at which quantity we can buy that good with trader's money. Then we compute the prices for each one of these values using the get_buy_price() method using as parameters the values we obtained in the how_much_buy() method. Now, we compare the prices of the three markets and we decide which one is the mininum. In this way we spend less and we can potentailly get more goods from the market the trader chooses to trade with. As a result we get the best kind of good, the best buy quantity and the best market to buy the good from. All of these three variables will be useful into the next step, where the trader buys the good we have decided to buy through the lock_buy() and buy() functions. When executing the buy() function, the good in return is saved in a variable used for the merge() function in order to increase the quantity of that good inside the trader's good list. Meanwhile, in this part we also reduce the EURs from the trader's budget.

The selling part follows an approach similar to the buying part, except for the fact that now we have to sell the good the trader just bought at the highest price possible, by computing the highest quantity of EURs possibly attainable from all markets and we try to sell the highest amount of that good in order to get the highest amount of EURs possible. Repeat again for i interactions to gain more profit. 


## Trader number 2 
## Trader number 3

## Data visualizer (WIP)
