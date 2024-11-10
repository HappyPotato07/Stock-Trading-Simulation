use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Stock {
    pub stock_name: String,
    pub current_price: f64,
}

impl Stock {
    pub fn new(stock_name: &str, current_price: f64) -> Self {
        Stock {
            stock_name: stock_name.to_string(),
            current_price,
        }
    }

    pub fn adjust_price(&mut self, market_news: &MarketNews) {
        let adjustment = match market_news {
            MarketNews::Good => 0.05,
            MarketNews::Bad => -0.05,
            MarketNews::Neutral => 0.0,
        };
        self.current_price *= 1.0 + adjustment;
    }
}

#[derive(Debug, Clone)]
pub struct MarketFactors {
    pub unemployment_rate: f64,
    pub gdp_growth: f64,
}

impl MarketFactors {
    pub fn new(unemployment_rate: f64, gdp_growth: f64) -> Self {
        MarketFactors {
            unemployment_rate,
            gdp_growth,
        }
    }

    pub fn determine_market_news(&self) -> MarketNews {
        if self.unemployment_rate < 6.0 && self.gdp_growth > 2.0 {
            MarketNews::Good
        } else if self.unemployment_rate > 8.0 || self.gdp_growth < 0.0 {
            MarketNews::Bad
        } else {
            MarketNews::Neutral
        }
    }

    pub fn print_factors(&self) {
        println!(
            "\x1b[34mMARKET: Unemployment Rate is {:.2}% & GDP Growth is {:.2}%\x1b[0m",
            self.unemployment_rate, self.gdp_growth
        );
    }
}

#[derive(Debug)]
pub enum MarketNews {
    Good,
    Bad,
    Neutral,
}
