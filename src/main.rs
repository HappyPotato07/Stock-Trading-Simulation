
use std::sync::{atomic::{AtomicBool, AtomicUsize}, Arc, Mutex, RwLock};
use std::sync::mpsc::channel;
use rts_stockv3::stock_object::{Stock, MarketFactors};
use rts_stockv3::trader::start_traders;
use rts_stockv3::broker::Broker;
use std::thread;

fn main() {
    // Initialize stocks
    let stocks = vec![
        Stock::new("NIKE", 1500.0),
        Stock::new("ADIDAS", 2500.0),
        Stock::new("PUMA", 3300.0),
        Stock::new("YONEX", 3000.0),
        Stock::new("LINING", 4500.0),
    ];
    
    let stocks = Arc::new(RwLock::new(stocks));
    
    // Initialize market factors
    let market_factors = Arc::new(RwLock::new(MarketFactors::new(6.0, 2.5))); // Example values for unemployment rate and GDP growth
    
    // Initialize order counter
    let order_count = Arc::new(AtomicUsize::new(0));
    
    // Initialize stop signal
    let stop_signal = Arc::new(AtomicBool::new(false));
    
    // Create channel for market factors updates
    let (tx, rx) = channel();

    println!("\nMARKET OPENS.....");

    // Start RabbitMQ processing thread
    let _broker_stocks = Arc::clone(&stocks);
    let broker = Broker::new(Arc::new(Mutex::new((*stocks.read().unwrap()).clone())), 
    Arc::clone(&order_count), Arc::clone(&stop_signal), rx);
    let broker_handle = thread::spawn(move || {
        broker.process_orders();
    });
    
    // Start traders
    start_traders(Arc::clone(&stocks), Arc::clone(&market_factors), 
    Arc::clone(&order_count), Arc::clone(&stop_signal), tx);
    
    // Wait for broker to finish processing
    broker_handle.join().unwrap();

    println!("MARKET CLOSED...");
}

