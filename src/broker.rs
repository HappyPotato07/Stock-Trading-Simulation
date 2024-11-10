
use std::sync::{Arc, Mutex, atomic::{AtomicUsize, Ordering, AtomicBool}};
use chrono::Local;

use crate::stock_object::{MarketFactors, Stock};
use crate::rmq::consume;
use std::sync::mpsc::Receiver;
use crate::trader::{NUM_TRADERS, ORDERS_PER_TRADER};

pub struct Broker {
    stocks: Arc<Mutex<Vec<Stock>>>,
    order_count: Arc<AtomicUsize>,
    stop_signal: Arc<AtomicBool>,
    market_rx: Receiver<MarketFactors>,
}

impl Broker {
    pub fn new(stocks: Arc<Mutex<Vec<Stock>>>, order_count: Arc<AtomicUsize>, stop_signal: Arc<AtomicBool>, market_rx: Receiver<MarketFactors>) -> Self {
        Broker { stocks, order_count, stop_signal, market_rx }
    }

    pub fn process_orders(&self) {
        let current_time = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        loop {
            if self.stop_signal.load(Ordering::SeqCst) {
                break;
            }

            let order = consume("stock_order");
            if order.is_empty() {
                if self.order_count.load(Ordering::SeqCst) >= NUM_TRADERS * ORDERS_PER_TRADER {
                    break;
                }
                continue;
            }

            // Deserialize the JSON to a Stock object
            match serde_json::from_str::<Stock>(&order) {
                Ok(stock) => {
                    println!("* Received order: stock_name: {}, current_price: ${:.2}", stock.stock_name.trim(), 
                    stock.current_price);
                    
                    let mut current_stocks = self.stocks.lock().unwrap();
                    if let Some(existing_stock) = current_stocks.iter_mut().find(|s| 
                        s.stock_name == stock.stock_name) {
                        existing_stock.current_price = stock.current_price;

                        println!("{}, Order processing... {} share prices updated at ${:.2}", current_time, 
                        existing_stock.stock_name.trim(), existing_stock.current_price);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to deserialize stock: {}", e);
                }
            }
        }
        println!("\nBroker has finished processing all orders.");
    }
}

 


