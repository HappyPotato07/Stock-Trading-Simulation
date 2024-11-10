
use std::sync::{Arc, RwLock, atomic::{AtomicUsize, Ordering, AtomicBool}};
use std::thread;
use std::time::Duration;
use chrono::Local;
use rand::Rng;
use serde_json::to_string;
use crate::stock_object::{Stock, MarketFactors, MarketNews};
use crate::rmq::send;
use std::sync::mpsc::Sender;

pub const NUM_TRADERS: usize = 5;
pub const ORDERS_PER_TRADER: usize = 20;

pub struct Trader {
    id: usize,
    stocks: Arc<RwLock<Vec<Stock>>>,
    market_factors: Arc<RwLock<MarketFactors>>,
    order_count: Arc<AtomicUsize>,
    stop_signal: Arc<AtomicBool>,
    market_tx: Sender<MarketFactors>,
}

impl Trader {
    fn new(id: usize, stocks: Arc<RwLock<Vec<Stock>>>, market_factors: Arc<RwLock<MarketFactors>>, order_count: Arc<AtomicUsize>, stop_signal: Arc<AtomicBool>, market_tx: Sender<MarketFactors>) -> Self {
        Trader { id, stocks, market_factors, order_count, stop_signal, market_tx }
    }

    fn generate_order(&self) {
        let mut rng = rand::thread_rng();
        let mut orders_generated = 0;

        while orders_generated < ORDERS_PER_TRADER {
            // Introduce a random delay between operations
            let delay = rng.gen_range(100..500);
            thread::sleep(Duration::from_millis(delay));

            // Randomly update market factors
            if rng.gen_bool(0.4) { //40% chance to update market factors
                let mut market_factors = self.market_factors.write().unwrap();
                market_factors.unemployment_rate = rng.gen_range(3.0..10.0);
                market_factors.gdp_growth = rng.gen_range(-1.0..4.0);
                market_factors.print_factors();
                self.market_tx.send(market_factors.clone()).unwrap();

                let market_news = market_factors.determine_market_news();
                match market_news {
                    MarketNews::Good => println!("\x1b[32m!!! NEWS: Stock share prices are expected to rise.\x1b[0m"),
                    MarketNews::Bad => println!("\x1b[31m!!! NEWS: Stock share prices are expected to fall.\x1b[0m"),
                    MarketNews::Neutral => println!("\x1b[38;5;230m!!! NEWS: No significant changes in stock share prices are expected.\x1b[0m")
                }
            }

            let market_factors = self.market_factors.read().unwrap();
            let market_news = market_factors.determine_market_news();

            let mut stocks = self.stocks.write().unwrap();
            let stock_index = rng.gen_range(0..stocks.len());
            let stock = &mut stocks[stock_index];
            let original_price = stock.current_price;
            let price_change: f64 = rng.gen_range(-0.2..0.2);

            // Adjust price based on market news
            stock.adjust_price(&market_news);

            // Determine buy or sell based on the price change
            let activity = if price_change < 0.0 {
                "buy"
            } else {
                "sell"
            };

            // Adjust stock price based on activity
            if activity == "buy" {
                stock.current_price += original_price * (price_change + 0.05); // Example logic for buying
            } else {
                stock.current_price += original_price * (price_change - 0.05); // Example logic for selling
            }
            

            // Serialize the stock to JSON
            match to_string(&*stock) {
                Ok(order) => {
                    let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    println!("{}, Trader {}: {} {} shares at ${:.2}", current_time, self.id + 1, activity, 
                    stock.stock_name, stock.current_price);

                    if let Err(e) = send(order, "stock_order") {
                        eprintln!("Trader {}: Failed to send order: {}", self.id + 1, e);
                    }
                },
                Err(e) => {
                    eprintln!("Trader {}: Failed to serialize stock: {}", self.id + 1, e);
                }
            }

            orders_generated += 1;
            self.order_count.fetch_add(1, Ordering::SeqCst);

            if self.order_count.load(Ordering::SeqCst) >= NUM_TRADERS * ORDERS_PER_TRADER {
                self.stop_signal.store(true, Ordering::SeqCst);
                break;
            }
        }
        println!("Trader {} has completed {} orders and is now stopping.", self.id + 1, ORDERS_PER_TRADER);
    }
}

pub fn start_traders(stocks: Arc<RwLock<Vec<Stock>>>, market_factors: Arc<RwLock<MarketFactors>>,
     order_count: Arc<AtomicUsize>, stop_signal: Arc<AtomicBool>,market_tx: Sender<MarketFactors>) {
    let mut handles = vec![];

    for id in 0..NUM_TRADERS {
        let trader = Trader::new(id, Arc::clone(&stocks), 
        Arc::clone(&market_factors), Arc::clone(&order_count), 
        Arc::clone(&stop_signal), market_tx.clone());
        let handle = thread::spawn(move || {
            trader.generate_order();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

