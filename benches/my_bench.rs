// BENCHMARK - OVERALL SIMULATION ----------------------------------------------------------------
use criterion::{criterion_group, criterion_main, Criterion};
use rand::Rng;
use rts_stockv3::
    stock_object::{MarketFactors, Stock}
;
use std::sync::mpsc::{channel, Sender};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize},
    Arc, RwLock,
};
use std::thread;

pub fn benchmark_simulation(c: &mut Criterion) {
    c.bench_function("overall_simulation", |b| {
        b.iter(|| {
            // Initialize stocks
            let stocks = vec![
                Stock::new("NIKE", 1500.0),
                Stock::new("ADIDAS", 2500.0),
                Stock::new("PUMA", 3300.0),
                Stock::new("YONEX", 3000.0),
                Stock::new("LINING", 4500.0),
            ];

            let stocks = Arc::new(RwLock::new(stocks));
            let market_factors = Arc::new(RwLock::new(MarketFactors::new(6.0, 2.5)));
            let order_count = Arc::new(AtomicUsize::new(0));
            let stop_signal = Arc::new(AtomicBool::new(false));
            let (tx, rx) = channel();

            let broker_handle = {
                let order_count = Arc::clone(&order_count);
                let stop_signal = Arc::clone(&stop_signal);
                thread::spawn(move || {
                    while !stop_signal.load(std::sync::atomic::Ordering::SeqCst) {
                        if order_count.load(std::sync::atomic::Ordering::SeqCst) >= 5 * 20 {
                            stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
                        }
                    }
                    println!("\nBroker has finished processing all orders.");
                })
            };

            // Start traders
            let mut handles = vec![];
            for id in 0..5 {
                let trader = Trader::new(
                    id,
                    Arc::clone(&stocks),
                    Arc::clone(&market_factors),
                    Arc::clone(&order_count),
                    Arc::clone(&stop_signal),
                    tx.clone(),
                );
                let handle = thread::spawn(move || {
                    let mut rng = rand::thread_rng();
                    let mut orders_generated = 0;

                    while orders_generated < 20 {
                        if rng.gen_bool(0.2) {
                            let mut market_factors = trader.market_factors.write().unwrap();
                            market_factors.unemployment_rate = rng.gen_range(3.0..10.0);
                            market_factors.gdp_growth = rng.gen_range(-1.0..4.0);
                            trader.market_tx.send(market_factors.clone()).unwrap();
                        }

                        let market_factors = trader.market_factors.read().unwrap();
                        let market_news = market_factors.determine_market_news();

                        let mut stocks = trader.stocks.write().unwrap();
                        let stock_index = rng.gen_range(0..stocks.len());
                        let stock = &mut stocks[stock_index];
                        let original_price = stock.current_price;
                        let price_change: f64 = rng.gen_range(-0.2..0.2);

                        stock.adjust_price(&market_news);
                        let activity = if price_change < 0.0 { "buy" } else { "sell" };

                        if activity == "buy" {
                            stock.current_price += original_price * (price_change + 0.05);
                        // Example logic for buying
                        } else {
                            stock.current_price += original_price * (price_change - 0.05);
                            // Example logic for selling
                        }

                        orders_generated += 1;
                        trader
                            .order_count
                            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                        if trader.order_count.load(std::sync::atomic::Ordering::SeqCst) >= 5 * 20 {
                            trader
                                .stop_signal
                                .store(true, std::sync::atomic::Ordering::SeqCst);
                            break;
                        }
                    }
                });
                handles.push(handle);
            }

            for handle in handles {
                handle.join().unwrap();
            }

            broker_handle.join().unwrap();
        })
    });
}

struct Trader {
    id: usize,
    stocks: Arc<RwLock<Vec<Stock>>>,
    market_factors: Arc<RwLock<MarketFactors>>,
    order_count: Arc<AtomicUsize>,
    stop_signal: Arc<AtomicBool>,
    market_tx: Sender<MarketFactors>,
}

impl Trader {
    fn new(
        id: usize,
        stocks: Arc<RwLock<Vec<Stock>>>,
        market_factors: Arc<RwLock<MarketFactors>>,
        order_count: Arc<AtomicUsize>,
        stop_signal: Arc<AtomicBool>,
        market_tx: Sender<MarketFactors>,
    ) -> Self {
        Trader {
            id,
            stocks,
            market_factors,
            order_count,
            stop_signal,
            market_tx,
        }
    }
}

criterion_group!(benches, benchmark_simulation);
criterion_main!(benches);

// --------------------------------------------------------------------------------------
// Determine Market Factor ---------------------------------------------------------------
// use criterion::{criterion_group, criterion_main, Criterion};
// use std::sync::{Arc, RwLock};
// use rand::Rng;
// use rts_stockv3::stock_object::MarketFactors;

// fn update_market_factors(market_factors: Arc<RwLock<MarketFactors>>) {
//     let mut rng = rand::thread_rng();
//     let mut factors = market_factors.write().unwrap();
//     factors.unemployment_rate = rng.gen_range(3.0..10.0);
//     factors.gdp_growth = rng.gen_range(-1.0..4.0);
// }

// fn criterion_benchmark(c: &mut Criterion) {
//     let market_factors = Arc::new(RwLock::new(MarketFactors::new(6.0, 2.5)));
//     c.bench_function("update_market_factors", |b| {
//         b.iter(|| update_market_factors(Arc::clone(&market_factors)))
//     });
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);

// ----------------------------------------------------------------------------------------
// Determine Market News ------------------------------------------------------------------
// use criterion::{criterion_group, criterion_main, Criterion};
// use std::sync::{Arc, RwLock};
// use rand::Rng;
// use rts_stockv3::stock_object::MarketFactors;

// fn update_market_factors(market_factors: Arc<RwLock<MarketFactors>>) {
//     let mut rng = rand::thread_rng();
//     let mut factors = market_factors.write().unwrap();
//     factors.unemployment_rate = rng.gen_range(3.0..10.0);
//     factors.gdp_growth = rng.gen_range(-1.0..4.0);
// }

// fn determine_market_news_benchmark(market_factors: Arc<RwLock<MarketFactors>>) {
//     let factors = market_factors.read().unwrap();
//     factors.determine_market_news();
// }

// fn criterion_benchmark(c: &mut Criterion) {
//     let market_factors = Arc::new(RwLock::new(MarketFactors::new(6.0, 2.5)));

//     c.bench_function("update_market_factors", |b| {
//         b.iter(|| update_market_factors(Arc::clone(&market_factors)))
//     });

//     c.bench_function("determine_market_news", |b| {
//         b.iter(|| determine_market_news_benchmark(Arc::clone(&market_factors)))
//     });
// }

// criterion_group!(benches, criterion_benchmark);
// criterion_main!(benches);

// ----------------------------------------------------------------------------------------
// Simulation per Trader (20 orders) ------------------------------------------------------

// use std::{sync::{atomic::{AtomicBool, AtomicUsize}, mpsc::channel, Arc, RwLock}, thread};

// use criterion::{criterion_group, criterion_main, Criterion};
// use rand::Rng;
// use rts_stockv3::stock_object::{MarketFactors, Stock};

// pub const ORDERS_PER_TRADER: usize = 20;

// pub fn benchmark_orders_per_trader(c: &mut Criterion) {
//     c.bench_function("orders_per_trader", |b| b.iter(|| {
//         let stocks = Arc::new(RwLock::new(vec![
//             Stock::new("NIKE", 1500.0),
//             Stock::new("ADIDAS", 2500.0),
//             Stock::new("PUMA", 3300.0),
//             Stock::new("YONEX", 3000.0),
//             Stock::new("LINING", 4500.0),
//         ]));

//         let market_factors = Arc::new(RwLock::new(MarketFactors::new(6.0, 2.5)));
//         let order_count = Arc::new(AtomicUsize::new(0));
//         let stop_signal = Arc::new(AtomicBool::new(false));
//         let (tx, rx) = channel();

//         // Broker thread to stop when enough orders are processed
//         let broker_handle = {
//             let order_count = Arc::clone(&order_count);
//             let stop_signal = Arc::clone(&stop_signal);
//             thread::spawn(move || {
//                 while !stop_signal.load(std::sync::atomic::Ordering::SeqCst) {
//                     if order_count.load(std::sync::atomic::Ordering::SeqCst) >= ORDERS_PER_TRADER {
//                         stop_signal.store(true, std::sync::atomic::Ordering::SeqCst);
//                     }
//                 }
//             })
//         };

//         // Trader thread to generate orders
//         let trader_handle = {
//             let stocks = Arc::clone(&stocks);
//             let market_factors = Arc::clone(&market_factors);
//             let order_count = Arc::clone(&order_count);
//             let stop_signal = Arc::clone(&stop_signal);
//             thread::spawn(move || {
//                 let mut rng = rand::thread_rng();
//                 for _ in 0..ORDERS_PER_TRADER {
//                     if stop_signal.load(std::sync::atomic::Ordering::SeqCst) {
//                         break;
//                     }

//                     if rng.gen_bool(0.4) {
//                         let mut factors = market_factors.write().unwrap();
//                         factors.unemployment_rate = rng.gen_range(3.0..10.0);
//                         factors.gdp_growth = rng.gen_range(-1.0..4.0);
//                         tx.send(factors.clone()).unwrap();
//                     }

//                     let factors = market_factors.read().unwrap();
//                     let news = factors.determine_market_news();
//                     let mut stocks = stocks.write().unwrap();
//                     let stock = &mut stocks[rng.gen_range(0..4)];
//                     stock.adjust_price(&news);
//                     order_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
//                 }
//             })
//         };
//         trader_handle.join().unwrap();
//         broker_handle.join().unwrap();
//     }));
// }

// criterion_group!(benches, benchmark_orders_per_trader);
// criterion_main!(benches);

// ----------------------------------------------------------------------------------------
// Stock Generator ------------------------------------------------------------------------
// use criterion::{criterion_group, criterion_main, Criterion};
// use rand::Rng;
// use rts_stockv3::stock_object::Stock;

// const STOCK_NAMES: [&str; 5] = ["NIKE", "ADIDAS", "PUMA", "YONEX", "LINING"];
// const BASE_PRICES: [f64; 5] = [1500.0, 2500.0, 3300.0, 3000.0, 4500.0];

// fn generate_stocks() -> Vec<Stock> {
//     let mut rng = rand::thread_rng();
//     STOCK_NAMES.iter().zip(BASE_PRICES.iter()).map(|(&name, &base_price)| {
//         let price_variation: f64 = rng.gen_range(-100.0..100.0);
//         Stock::new(name, base_price + price_variation)
//     }).collect()
// }

// fn benchmark_stock_generator(c: &mut Criterion) {
//     c.bench_function("stock_generator", |b| b.iter(|| {
//         generate_stocks()
//     }));
// }

// criterion_group!(benches, benchmark_stock_generator);
// criterion_main!(benches);

// -----------------------------------------------------------------------------------------
// Affect Price Benchmark ------------------------------------------------------------------

// use criterion::{criterion_group, criterion_main, Criterion};
// use rand::Rng;
// use rts_stockv3::stock_object::{Stock, MarketNews};

// const STOCK_NAME: &str = "NIKE";
// const INITIAL_PRICE: f64 = 1500.0;

// fn adjust_stock_price(stock: &mut Stock, news: &MarketNews) {
//     stock.adjust_price(news);
// }

// fn benchmark_adjust_price(c: &mut Criterion) {
//     // Create a stock instance
//     let mut stock = Stock::new(STOCK_NAME, INITIAL_PRICE);

//     // Different market news scenarios
//     let news_scenarios = [
//         MarketNews::Good,
//         MarketNews::Bad,
//         MarketNews::Neutral,
//     ];

//     let mut rng = rand::thread_rng();

//     c.bench_function("adjust_price", |b| {
//         b.iter(|| {
//             // Select a random market news scenario
//             let news = &news_scenarios[rng.gen_range(0..news_scenarios.len())];
//             adjust_stock_price(&mut stock, news);
//         });
//     });
// }

// criterion_group!(benches, benchmark_adjust_price);
// criterion_main!(benches);

// -----------------------------------------------------------------------------------------
// Trader's Decision (Buy/Sell) ------------------------------------------------------------

// use criterion::{criterion_group, criterion_main, Criterion};
// use rand::Rng;
// use rts_stockv3::stock_object::{Stock, MarketFactors, MarketNews};

// const STOCK_NAME: &str = "NIKE";
// const INITIAL_PRICE: f64 = 1500.0;

// fn trader_decision(stock: &mut Stock, market_news: &MarketNews) {
//     let mut rng = rand::thread_rng();
//     let original_price = stock.current_price;
//     let price_change: f64 = rng.gen_range(-0.2..0.2);

//     // Adjust price based on market news
//     stock.adjust_price(market_news);

//     // Determine buy or sell based on the price change
//     let activity = if price_change < 0.0 {
//         "buy"
//     } else {
//         "sell"
//     };

//     // Adjust stock price based on activity
//     if activity == "buy" {
//         stock.current_price += original_price * (price_change + 0.05); // Example logic for buying
//     } else {
//         stock.current_price += original_price * (price_change - 0.05); // Example logic for selling
//     }
// }

// fn benchmark_trader_decision(c: &mut Criterion) {
//     // Create a stock instance
//     let mut stock = Stock::new(STOCK_NAME, INITIAL_PRICE);

//     // Different market news scenarios
//     let market_factors = MarketFactors::new(6.0, 2.5);
//     let market_news = market_factors.determine_market_news();

//     c.bench_function("trader_decision", |b| {
//         b.iter(|| {
//             trader_decision(&mut stock, &market_news)
//         });
//     });
// }

// criterion_group!(benches, benchmark_trader_decision);
// criterion_main!(benches);

// -----------------------------------------------------------------------------------------
// Broker Process Order --------------------------------------------------------------------
// use criterion::{criterion_group, criterion_main, Criterion};
// use std::sync::{atomic::AtomicUsize, Arc, Mutex};
// use rts_stockv3::stock_object::Stock;
// use serde_json::to_string;

// const STOCK_LIST: [(&str, f64); 5] = [
//     ("NIKE", 1500.0),
//     ("ADIDAS", 2500.0),
//     ("PUMA", 3300.0),
//     ("YONEX", 3000.0),
//     ("LINING", 4500.0),
// ];

// fn generate_order(stock: &Stock) -> String {
//     to_string(stock).unwrap()
// }

// fn process_order_benchmark(c: &mut Criterion) {
//     let stocks: Vec<Stock> = STOCK_LIST.iter().map(|&(name, price)| Stock::new( name, price)).collect();
//     let stocks = Arc::new(Mutex::new(stocks));
//     let order_count = Arc::new(AtomicUsize::new(0));

//     // Generate dummy orders
//     let orders: Vec<String> = stocks.lock().unwrap().iter().map(|stock| generate_order(stock)).collect();

//     // Create a mock function to process orders directly
//     fn process_orders_mock(stocks: Arc<Mutex<Vec<Stock>>>, orders: Vec<String>, order_count: Arc<AtomicUsize>) {
//         for order in orders {
//             let stock: Stock = serde_json::from_str(&order).unwrap();
//             let mut current_stocks = stocks.lock().unwrap();
//             if let Some(existing_stock) = current_stocks.iter_mut().find(|s| 
//                 s.stock_name == stock.stock_name) {
//                 existing_stock.current_price = stock.current_price;
//                 order_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
//             }
//         }
//     }

//     c.bench_function("process_order", |b| {
//         b.iter(|| {
//             process_orders_mock(Arc::clone(&stocks), orders.clone(), 
//             Arc::clone(&order_count));
//         });
//     });
// }

// criterion_group!(benches, process_order_benchmark);
// criterion_main!(benches);


// -----------------------------------------------------------------------------------------
