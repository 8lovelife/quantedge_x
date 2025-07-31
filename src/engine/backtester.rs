use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    data::{coin_market::MarketOhlcv, csv_data_feed::CsvDataFeed, data_feed::DataFeed},
    domain::PositionSizer,
    executor::backtest_executor::BacktestExecutor,
    processor::SignalProcessor,
    risk::risk_manager_factory::RiskManagerFactory,
    sizer::sizer_factory::SizerFactory,
    strategy::{
        market_data::MarketData,
        position::{Position, PositionType, TradePosition},
        signal::Signal,
        strategy_context::StrategyContext,
        strategy_factory::StrategyFactory,
        strategy_trait::Strategy,
    },
};

use super::{
    backtest_result::{BacktestResult, Balance, Trade, TradeResultType, TradeType},
    observer::PerformanceObserver,
    parameters::{BacktestInput, RunLabStrategy, StrategyRunParams},
    trading_engine::TradingEngine,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetAllocation {
    pub symbol: String,
    pub allocation: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioAsset {
    pub allocations: Vec<AssetAllocation>,
}

impl PortfolioAsset {
    pub fn new(allocations: Vec<AssetAllocation>) -> PortfolioAsset {
        PortfolioAsset { allocations }
    }
    pub fn calculate_portfolio_price(&self, market: &MarketOhlcv) -> Option<f64> {
        // let asset_coin_mapping = get_asset_symbol_mapping();
        let mut portfolio_price = 0.0;
        for asset in &self.allocations {
            // let symbol = asset_coin_mapping
            //     .get(asset.symbol.as_str())
            //     .unwrap_or(&"unknow");
            // println!("symbol {:?}", symbol);
            if let Some(ohlcv) = market.ohlcv.get(asset.symbol.as_str()) {
                portfolio_price += ohlcv.close * (asset.allocation as f64);
            } else {
                return None;
            }
        }
        Some(portfolio_price)
    }

    pub fn to_portfolio_market_data(&self, all_data: Vec<MarketOhlcv>) -> Vec<MarketData> {
        all_data
            .into_iter()
            .filter_map(|entry| {
                self.calculate_portfolio_price(&entry)
                    .map(|portfolio_price| MarketData {
                        timestamp: entry.timestamp,
                        close_price: portfolio_price,
                    })
            })
            .collect()
    }
}

pub struct Backtester {
    strategy: Box<dyn Strategy>,
    capital: f64,
    initial_capital: f64,
    position: Option<TradePosition>,
    trade_log: Vec<(String, Signal, f64, Option<f64>, f64)>, // (Timestamp, Signal, Price, Profit,Capital after trade)
    returns: Vec<(String, f64)>,                             // (Timestamp,return)
    max_drawdown: f64,
    stop_loss: f64,
    take_profit: f64,
    use_trailing_stop: bool,
    trailing_stop_distance: f64,
    risk_per_trade: f64,
    finish_running: bool,
}

impl Backtester {
    pub fn new(params: StrategyRunParams) -> Self {
        Self {
            strategy: StrategyFactory::create_strategy(params.name, params.strategy),
            capital: params.starting_capital,
            initial_capital: params.starting_capital,
            position: None,
            trade_log: Vec::new(),
            returns: Vec::new(),
            max_drawdown: 0.0,
            stop_loss: params.stop_loss,
            take_profit: params.take_profit,
            use_trailing_stop: params.use_trailing_stop,
            trailing_stop_distance: params.trailing_stop_distance,
            risk_per_trade: params.risk_per_trade,
            finish_running: false,
        }
    }

    pub fn run(&mut self, market_data: Vec<MarketData>) {
        let mut peak_capital = self.capital;
        for data in market_data {
            let prev_capital = self.capital;
            // let signal = self.strategy.generate_signal(&data);
            // match signal {
            //     Signal::Buy => {
            //         if self.position.is_none() {
            //             // let risk_amount = self.capital * 0.02; // 2% risk per trade
            //             // let stop_loss_price = data.close_price * 0.98; // 2% Stop Loss
            //             // let size = risk_amount / (data.close_price - stop_loss_price); // Size based on stop loss

            //             let risk_amount = self.capital * (self.risk_per_trade / 100.0);
            //             let stop_loss_price = data.close_price * (1.0 - self.stop_loss / 100.0);
            //             let risk_per_unit = (data.close_price - stop_loss_price).max(0.01); // Avoid division by zero
            //             let size = risk_amount / risk_per_unit; // Adjusted position sizing

            //             let position = TradePosition::Long {
            //                 quantity: size,
            //                 entry_price: data.close_price,
            //                 held_bars: 0,
            //             };

            //             // {
            //             //     position_type: PositionType::Long,
            //             //     entry_price: data.close_price,
            //             //     size: size,
            //             //     stop_loss: stop_loss_price,
            //             //     take_profit: data.close_price * (1.0 + self.take_profit / 100.0),
            //             // };
            //             self.position = Some(position);
            //             self.trade_log.push((
            //                 data.timestamp.clone(),
            //                 Signal::Buy,
            //                 data.close_price,
            //                 None,
            //                 self.capital,
            //             ));
            //         }
            //     }
            //     Signal::Sell => {
            //         if let Some(pos) = &self.position.take() {
            //             let profit = match pos {
            //                 TradePosition::Long {
            //                     quantity,
            //                     entry_price,
            //                     ..
            //                 } => (data.close_price - entry_price) * *quantity as f64,
            //                 TradePosition::Short {
            //                     quantity,
            //                     entry_price,
            //                     ..
            //                 } => (entry_price - data.close_price) * *quantity as f64,
            //                 _ => 0.0,
            //             };
            //             self.capital += profit;
            //             self.trade_log.push((
            //                 data.timestamp.clone(),
            //                 Signal::Sell,
            //                 data.close_price,
            //                 Some(profit),
            //                 self.capital,
            //             ));
            //         }
            //     }
            //     Signal::None => {}
            // }

            // // 错误的以防编译错误
            // if let Some(pos) = &self.position {
            //     if data.close_price >= 0.01 || data.close_price <= 0.05 {
            //         let profit = (data.close_price - 10.0) * 5.0;
            //         self.capital += profit;
            //         self.position = None;
            //         self.trade_log.push((
            //             data.timestamp.clone(),
            //             Signal::Sell,
            //             data.close_price,
            //             Some(profit),
            //             self.capital,
            //         ));
            //     }
            // }

            // Track Returns
            let daily_return = (self.capital - prev_capital) / prev_capital;
            self.returns.push((data.timestamp.clone(), daily_return));

            // Update max drawdown
            if self.capital > peak_capital {
                peak_capital = self.capital;
            }
            let drawdown = (peak_capital - self.capital) / peak_capital;
            if drawdown > self.max_drawdown {
                self.max_drawdown = drawdown;
            }

            self.strategy.update(&data, &self.position);
        }

        self.finish_running = true;
    }

    pub fn get_backtest_result(&self) -> Option<BacktestResult> {
        if !self.finish_running {
            return None;
        }

        let mut balance = Vec::new();
        let mut trades = Vec::new();
        // for (date, signal, _price, profit, cur_captial) in &self.trade_log {
        //     if Signal::Sell == *signal {
        //         balance.push(Balance {
        //             date: date.to_string(),
        //             capital: *cur_captial,
        //             // market: 100.0,
        //             trades: 1,
        //         });
        //     }
        //     let trade = match signal {
        //         Signal::Buy => None,
        //         Signal::Sell => profit.map(|p| Trade {
        //             date: date.to_string(),
        //             trade_type: TradeType::Sell,
        //             profit: p,
        //             result: if p > 0.0 {
        //                 TradeResultType::Win
        //             } else {
        //                 TradeResultType::Loss
        //             },
        //         }),
        //         _ => None, // Ignore other signals
        //     };

        //     if let Some(trade) = trade {
        //         trades.push(trade);
        //     }
        // }

        Some(BacktestResult {
            total_return: self.total_return(),
            cagr: self.cagr(1.0),
            max_drawdown: self.max_drawdown(),
            sharpe_ratio: self.sharpe_ratio(0.05),
            profit_factor: self.profit_factor(),
            win_rate: self.win_rate(),
            final_capital: self.capital,
            balances: balance,
            trades: trades,
        })
    }

    /// **Print Backtest Summary**
    pub fn print_summary(&self, years: f64, risk_free_rate: f64) {
        println!("📈 Backtest Summary:");
        println!("-------------------------------");
        println!("Total Return: {:.2}%", self.total_return());
        println!("CAGR: {:.2}% ({} years)", self.cagr(years), years);
        println!("Max Drawdown: {:.2}%", self.max_drawdown());
        println!("Sharpe Ratio: {:.2}", self.sharpe_ratio(risk_free_rate));
        println!("Profit Factor: {:.2}", self.profit_factor());
        println!("Win Rate: {:.2}%", self.win_rate());
        println!("Final Capital: ${:.2}", self.capital);
        println!("-------------------------------");
    }

    pub fn report(&self) {
        println!("Final Capital: ${:.2}", self.capital);
        println!("Trade Log:");
        for (timestamp, signal, price, _profit, _cur_captial) in &self.trade_log {
            println!(
                "Time: {}, Signal: {:?}, Price: {:.2}",
                timestamp, signal, price
            );
        }
    }

    pub fn total_return(&self) -> f64 {
        ((self.capital - self.initial_capital) / self.initial_capital) * 100.0
    }

    pub fn max_drawdown(&self) -> f64 {
        self.max_drawdown * 100.0
    }

    /// **Sharpe Ratio Calculation**
    /// Measures risk-adjusted return. Higher is better.
    pub fn sharpe_ratio(&self, risk_free_rate: f64) -> f64 {
        if self.returns.is_empty() {
            return 0.0; // No trades = no risk-adjusted return
        }

        // Extract only the return values (ignore timestamps)
        let returns: Vec<f64> = self.returns.iter().map(|(_, r)| *r).collect();

        // Compute the average return (mean return)
        let avg_return = returns.iter().sum::<f64>() / returns.len() as f64;

        // Compute variance (Bessel's correction for unbiased standard deviation)
        let n = returns.len() as f64;
        let variance = if n > 1.0 {
            returns
                .iter()
                .map(|r| (r - avg_return).powi(2))
                .sum::<f64>()
                / (n - 1.0)
        } else {
            0.0
        };

        let std_dev = variance.sqrt();

        // Prevent divide-by-zero error
        if std_dev == 0.0 {
            return 0.0;
        }

        let sharpe = (avg_return - risk_free_rate) / std_dev;

        sharpe
    }

    /// **CAGR (Compounded Annual Growth Rate)**
    pub fn cagr(&self, years: f64) -> f64 {
        ((self.capital / self.initial_capital).powf(1.0 / years) - 1.0) * 100.0
    }

    /// **Profit Factor**
    pub fn profit_factor(&self) -> f64 {
        let mut total_profit = 0.0;
        let mut total_loss = 0.0;

        let mut last_entry: Option<f64> = None;

        // for (_, signal, price, _profit, _) in &self.trade_log {
        //     match signal {
        //         Signal::Buy | Signal::Sell if last_entry.is_none() => {
        //             last_entry = Some(*price);
        //         }
        //         Signal::Sell => {
        //             if let Some(entry_price) = last_entry {
        //                 let pnl = price - entry_price;
        //                 if pnl > 0.0 {
        //                     total_profit += pnl;
        //                 } else {
        //                     total_loss += pnl.abs();
        //                 }
        //                 last_entry = None; // Reset after trade closes
        //             }
        //         }
        //         _ => {}
        //     }
        // }

        if total_loss == 0.0 {
            return f64::INFINITY; // Avoid division by zero
        }

        total_profit / total_loss
    }

    /// **Win Rate (%)**
    pub fn win_rate(&self) -> f64 {
        let mut total_trades = 0.0;
        let mut winning_trades = 0.0;

        let mut last_entry: Option<f64> = None;

        // for (_, signal, price, _profit, _) in &self.trade_log {
        //     match signal {
        //         Signal::Buy | Signal::Sell if last_entry.is_none() => {
        //             last_entry = Some(*price);
        //         }
        //         Signal::Sell => {
        //             if let Some(entry_price) = last_entry {
        //                 total_trades += 1.0;
        //                 if price > &entry_price {
        //                     winning_trades += 1.0;
        //                 }
        //                 last_entry = None;
        //             }
        //         }
        //         _ => {}
        //     }
        // }

        if total_trades == 0.0 {
            return 0.0;
        }

        (winning_trades / total_trades) * 100.0
    }
}

pub fn build_and_run_backtest(
    config: &BacktestInput,
    data_feed: Box<dyn DataFeed>,
) -> BacktestResult {
    let datafeed = CsvDataFeed::from_file("path").unwrap();

    let strategy_factory = StrategyFactory::new();
    let strategy = strategy_factory.build(config.r#type.as_str(), &config.strategy_run_params);

    // 3. Executor：回测专用，传入初始资金与滑点、手续费参数
    let slippage = &config
        .strategy_run_params
        .get("slippage")
        .and_then(Value::as_f64);
    let commission = &config
        .strategy_run_params
        .get("commission")
        .and_then(Value::as_f64);

    let mut executor = BacktestExecutor::new(config.initial_capital)
        .with_slippage(slippage.unwrap_or(0.0))
        .with_commission(commission.unwrap_or(0.0));

    let sizer: Box<dyn PositionSizer> = SizerFactory::build(&config.strategy_run_params);
    let rm = RiskManagerFactory::build(&config.strategy_run_params);

    // 6. SignalProcessor：挂载 executor、sizer 和日志观察者
    let mut processor = SignalProcessor::new(executor, sizer);

    let perf_logger = PerformanceObserver::new(config.initial_capital);
    processor.add_observer(Box::new(perf_logger.clone()));

    // 7. StrategyContext：初始空仓
    // let mut ctx = StrategyContext::new(config.initial_capital);

    // 8. 构造并运行引擎
    let mut engine = TradingEngine::new(
        rm,
        datafeed,
        strategy.unwrap(),
        processor,
        config.initial_capital,
    );
    engine.run();

    // 9. 回测结束后，取出绩效
    // let trades = perf_logger.trades();
    // let balances = perf_logger.balances();
    // perf_logger.finalize(trades, balances)

    perf_logger.finalize()
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use serde_json::json;

    use crate::{
        data::{coin_market::CoinsMarket, data_feed::DataFeed, market_data_feed::MarketDataFeed},
        indicators::moving_average::MovingAverageType,
        strategy::strategy_type::StrategyType,
    };

    use super::*;

    /// DummyDataFeed：只依次产出传入的价格序列
    struct DummyDataFeed {
        idx: usize,
        data: Vec<MarketData>,
    }

    impl DummyDataFeed {
        fn new(prices: &[f64]) -> Self {
            let data = prices
                .iter()
                .enumerate()
                .map(|(i, &p)| MarketData {
                    timestamp: Utc.timestamp(i as i64, 0).to_rfc3339(),
                    close_price: p,
                })
                .collect();
            DummyDataFeed { idx: 0, data }
        }
    }

    impl DataFeed for DummyDataFeed {
        fn next(&mut self) -> Option<MarketData> {
            if self.idx < self.data.len() {
                let m = self.data[self.idx].clone();
                self.idx += 1;
                Some(m)
            } else {
                None
            }
        }
        fn reset(&mut self) {
            self.idx = 0;
        }
    }

    #[test]
    fn test_backtester() {
        let params = StrategyRunParams {
            name: "EMA(5,10)".to_string(),
            strategy: StrategyType::MA {
                short_period: MovingAverageType::EMA(15),
                long_period: MovingAverageType::EMA(30),
            },
            stop_loss: 2.0,
            take_profit: 3.0,
            use_trailing_stop: false,
            trailing_stop_distance: 0.0,
            starting_capital: 10000.0,
            market_data: None,
            risk_per_trade: 2.0,
        };
        let mut backtester = Backtester::new(params);
        let ohlcv_datas = CoinsMarket::get_coin_ohlcv("bitcoin");
        if let Ok(ohlcv) = ohlcv_datas {
            let market_data: Vec<MarketData> = ohlcv
                .iter()
                .map(|ohlc| MarketData {
                    timestamp: ohlc.timestamp.0.to_rfc3339(),
                    close_price: ohlc.close,
                })
                .collect();
            backtester.run(market_data);
            backtester.report();
            backtester.print_summary(1.0, 0.05);
            let result = backtester.get_backtest_result();
            println!(
                "backtest result {}",
                serde_json::to_string_pretty(&result).unwrap()
            );

            println!("trace_log")
        }
        assert!(backtester.capital > 0.0);
    }

    #[test]
    fn test_build_and_run_backtest() {
        let config = BacktestInput {
            r#type: "ma-crossover".to_string(),
            initial_capital: 1_000.0,
            // position_type: "long".to_string(),
            strategy_run_params: json!({
                "maType": "sma",
                "fastPeriod": 5,
                "slowPeriod": 20,
                "positionType":"both"
            }),
        };

        // let datafeed =
        //     CsvDataFeed::from_file("src/data/bitcoin_2018-02-01_2025-04-24.csv").unwrap();

        let datafeed = MarketDataFeed::from_coins_market("bitcoin").unwrap();

        let backtester = BacktestDriver::new(config, datafeed);

        let result: BacktestResult = backtester.build_and_run_backtest();

        println!("result {:?}", serde_json::to_string(&result));
    }
}

/// 把 DataFeed、BacktestInput 和 build_and_run_backtest 串起来
pub struct BacktestDriver<DF: DataFeed> {
    config: BacktestInput,
    datafeed: DF,
}

impl<DF: DataFeed> BacktestDriver<DF> {
    pub fn new(config: BacktestInput, datafeed: DF) -> Self {
        Self { config, datafeed }
    }

    /// 真正触发回测，并返回结果
    pub fn build_and_run_backtest(self) -> BacktestResult {
        let datafeed = self.datafeed;

        let config = &self.config;

        let strategy_factory = StrategyFactory::new();
        let strategy = strategy_factory.build(config.r#type.as_str(), &config.strategy_run_params);

        // 3. Executor：回测专用，传入初始资金与滑点、手续费参数
        let slippage = self
            .config
            .strategy_run_params
            .get("slippage")
            .and_then(Value::as_f64);
        let commission = config
            .strategy_run_params
            .get("commission")
            .and_then(Value::as_f64);

        let executor = BacktestExecutor::new(config.initial_capital)
            .with_slippage(slippage.unwrap_or(0.0))
            .with_commission(commission.unwrap_or(0.0));

        let sizer: Box<dyn PositionSizer> = SizerFactory::build(&config.strategy_run_params);
        let rm = RiskManagerFactory::build(&config.strategy_run_params);

        // 6. SignalProcessor：挂载 executor、sizer 和日志观察者
        let mut processor = SignalProcessor::new(executor, sizer);

        let perf_logger = PerformanceObserver::new(config.initial_capital);
        processor.add_observer(Box::new(perf_logger.clone()));

        // 7. StrategyContext：初始空仓
        // let mut ctx = StrategyContext::new(config.initial_capital);

        // 8. 构造并运行引擎
        let mut engine = TradingEngine::new(
            rm,
            datafeed,
            strategy.unwrap(),
            processor,
            config.initial_capital,
        );
        engine.run();

        // 9. 回测结束后，取出绩效
        // let trades = perf_logger.trades();
        // let balances = perf_logger.balances();
        // perf_logger.finalize(trades, balances)

        perf_logger.finalize()
    }
}
