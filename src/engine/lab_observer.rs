use crate::strategy::{
    market_data::MarketData,
    position::{Position, TradePosition},
    signal::Signal,
    strategy_factory::StrategyFactory,
    strategy_trait::Strategy,
};

use super::{
    backtest_result::{BacktestResult, Balance, Trade, TradeResultType, TradeType},
    parameters::RunLabStrategy,
};

pub struct LabObserver {
    pub run_lab_strategy: RunLabStrategy,
    pub strategy: Box<dyn Strategy>,
    pub initial_capital: f64,
    pub capital: f64,
    pub position: Option<TradePosition>,
    pub trade_log: Vec<(String, Signal, f64, Option<f64>, f64)>, // (Timestamp, Signal, Price, Profit,Capital after trade)
    pub returns: Vec<(String, f64)>,                             // (Timestamp,return)
    max_drawdown: f64,
    pub finish_running: bool,
}

impl LabObserver {
    pub fn new(run_lab_strategy: RunLabStrategy) -> Self {
        println!(
            "Initializing LabObserver with strategy: {:?}",
            &run_lab_strategy
        );
        let mut strategy = StrategyFactory::create(
            "observe".to_string(),
            &run_lab_strategy.r#type,
            &run_lab_strategy.sub_type.clone().unwrap(),
            run_lab_strategy.strategy_run_params.fast_period.unwrap() as u32,
            run_lab_strategy.strategy_run_params.slow_period.unwrap() as u32,
        );
        let capital = run_lab_strategy.initial_capital;
        let position = None;
        let trade_log = Vec::new();
        let returns = Vec::new();
        let finish_running = false;

        strategy.apply_parameters(
            run_lab_strategy.strategy_run_params.entry_threshold,
            run_lab_strategy.strategy_run_params.exit_threshold,
        );

        Self {
            run_lab_strategy,
            strategy,
            capital,
            initial_capital: capital,
            position,
            trade_log,
            returns,
            max_drawdown: 0.0,
            finish_running,
        }
    }

    /// Run backtest over (timestamp, close_price) series
    pub fn run(&mut self, market_data: Vec<MarketData>) {
        let mut peak_capital = self.capital;
        let mut prev_capital = self.capital;
        let params = &self.run_lab_strategy.strategy_run_params;
        // Extract parameters
        let stop_loss = params.stop_loss.unwrap_or(0.0) / 100.0;
        let take_profit = params.take_profit.unwrap_or(0.0) / 100.0;
        let risk_trade = params.risk_per_trade.map(|v| v / 100.0);
        let pos_size = params.position_size.map(|v| v / 100.0);
        let max_pos = params.max_concurrent_positions.unwrap_or(1);
        let slippage = params.slippage.unwrap_or(0.0) / 100.0;
        let commission = params.commission.unwrap_or(0.0) / 100.0;
        let mode = self.run_lab_strategy.position_type.clone();
        let entry_threshold = params.entry_threshold.unwrap_or(0.0) / 100.0;
        let exit_threshold = params.exit_threshold.unwrap_or(0.0) / 100.0;

        println!(
            "Parameters - Stop Loss: {}, Take Profit: {}, Entry Threshold: {}, Exit Threshold: {}",
            stop_loss, take_profit, entry_threshold, exit_threshold
        );

        println!(
            "Market data range: {} to {}, Price range: {} to {}",
            market_data.first().unwrap().timestamp,
            market_data.last().unwrap().timestamp,
            market_data
                .iter()
                .map(|d| d.close_price)
                .min_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap(),
            market_data
                .iter()
                .map(|d| d.close_price)
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap()
        );

        for data in market_data {
            let (ts, price) = (data.timestamp.clone(), data.close_price);

            println!(
                "Price: {}, Stop Loss Trigger: {}, Take Profit Trigger: {}",
                price,
                price * (1.0 - stop_loss),
                price * (1.0 + take_profit)
            );

            // let signal = self.strategy.generate_signal(&data);

            // println!("Signal: {:?}, Position: {:?}", signal, self.position);

            // println!(
            //     "Checking position creation - Signal: {:?}, Position: {:?}, Max Positions: {}, Position Size: {:?}",
            //     signal, self.position, max_pos, pos_size
            // );

            // if (mode == "long" || mode == "both") && signal == Signal::Buy {
            //     println!("Processing long entry...");
            // }
            // if (mode == "short" || mode == "both") && signal == Signal::Sell {
            //     println!("Processing short entry...");
            // }

            // stop-loss / take-profit for open position
            if let Some(pos) = &self.position {
                match pos {
                    TradePosition::Long {
                        quantity,
                        entry_price,
                        ..
                    } => {
                        if price <= entry_price * (1.0 - stop_loss) && stop_loss > 0.0 {
                            // stop-loss long
                            self.close_long(
                                &ts,
                                price,
                                *quantity,
                                *entry_price,
                                slippage,
                                commission,
                            );
                            self.position = None;
                            continue;
                        }
                        if price >= entry_price * (1.0 + take_profit) && take_profit > 0.0 {
                            // take-profit long
                            self.close_long(
                                &ts,
                                price,
                                *quantity,
                                *entry_price,
                                slippage,
                                commission,
                            );
                            self.position = None;
                            continue;
                        }
                    }
                    TradePosition::Short {
                        quantity,
                        entry_price,
                        ..
                    } => {
                        if price >= entry_price * (1.0 + stop_loss) && stop_loss > 0.0 {
                            // stop-loss short
                            self.close_short(
                                &ts,
                                price,
                                *quantity,
                                *entry_price,
                                slippage,
                                commission,
                            );
                            self.position = None;
                            continue;
                        }
                        if price <= entry_price * (1.0 - take_profit) && take_profit > 0.0 {
                            // take-profit short
                            self.close_short(
                                &ts,
                                price,
                                *quantity,
                                *entry_price,
                                slippage,
                                commission,
                            );
                            self.position = None;
                            continue;
                        }
                    }
                }
            }

            // long entry
            if (mode == "long" || mode == "both")
                // && signal == Signal::Buy
                && self.position.is_none()
                && max_pos >= 1
            {
                self.open_long(
                    &ts, price, risk_trade, pos_size, stop_loss, slippage, commission,
                );
            }

            // short entry
            if (mode == "short" || mode == "both")
                // && signal == Signal::Sell
                && self.position.is_none()
                && max_pos >= 1
            {
                self.open_short(
                    &ts, price, risk_trade, pos_size, stop_loss, slippage, commission,
                );
            }

            // // exit long
            // if (mode == "long" || mode == "both")
            //     && signal == Signal::Sell
            //     && matches!(self.position, Some(TradePosition::Long { .. }))
            // {
            //     if let Some(TradePosition::Long {
            //         quantity,
            //         entry_price,
            //         ..
            //     }) = self.position.take()
            //     {
            //         self.close_long(&ts, price, quantity, entry_price, slippage, commission);
            //     }
            // }

            // exit short
            if (mode == "short" || mode == "both")
                // && signal == Signal::Buy
                && matches!(self.position, Some(TradePosition::Short { .. }))
            {
                if let Some(TradePosition::Short {
                    quantity,
                    entry_price,
                    ..
                }) = self.position.take()
                {
                    self.close_short(&ts, price, quantity, entry_price, slippage, commission);
                }
            }

            if self.capital <= 0.0 {
                println!("Capital depleted. Stopping backtest.");
                break;
            }

            // record return per bar
            let ret = if prev_capital > 0.0 {
                (self.capital - prev_capital) / prev_capital
            } else {
                0.0
            };
            self.returns.push((ts.clone(), ret));
            prev_capital = self.capital;

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

    fn position_size(
        &self,
        price: f64,
        stop_loss: f64,
        risk_pct: Option<f64>,
        size_pct: Option<f64>,
    ) -> f64 {
        if let (Some(risk), sl) = (risk_pct, stop_loss) {
            if sl > 0.0 {
                return ((self.capital * risk) / (price * sl))
                    .min(self.capital / price)
                    .floor();
            }
        }
        if let Some(sz) = size_pct {
            return ((self.capital * sz) / price)
                .min(self.capital / price)
                .floor();
        }
        (self.capital / price).floor()
    }

    fn open_long(
        &mut self,
        ts: &str,
        price: f64,
        risk_pct: Option<f64>,
        size_pct: Option<f64>,
        stop_loss: f64,
        slippage: f64,
        commission: f64,
    ) {
        let qty = self.position_size(price, stop_loss, risk_pct, size_pct);
        if qty <= 0.0 {
            return;
        }

        let exec = price * (1.0 + slippage);
        let cost = qty * exec;
        let fee = cost * commission;

        // self.capital -= cost + fee;
        // self.position = Some(TradePosition::Long {
        //     quantity: qty,
        //     entry_price: exec,
        //     held_bars: 0,
        // });
        // self.trade_log
        //     .push((ts.to_string(), Signal::Buy, exec, None, self.capital));

        // println!(
        //     "Trade Log - Timestamp: {}, Signal: {:?}, Price: {}, Profit: {:?}, Capital: {}",
        //     ts,
        //     Signal::Buy,
        //     exec,
        //     None::<f64>,
        //     self.capital
        // );
    }

    fn close_long(
        &mut self,
        ts: &str,
        price: f64,
        qty: f64,
        entry: f64,
        slippage: f64,
        commission: f64,
    ) {
        let exec = price * (1.0 - slippage);
        let proceeds = qty * exec;
        let fee = proceeds * commission;
        let pnl = proceeds - fee - qty * entry;

        // self.capital += proceeds - fee;
        // self.trade_log
        //     .push((ts.to_string(), Signal::Sell, exec, Some(pnl), self.capital));
        // self.position = None;

        // println!(
        //     "Trade Log - Timestamp: {}, Signal: {:?}, Price: {}, Profit: {:?}, Capital: {}",
        //     ts,
        //     Signal::Sell,
        //     exec,
        //     Some(pnl),
        //     self.capital
        // );
    }

    fn open_short(
        &mut self,
        ts: &str,
        price: f64,
        risk_pct: Option<f64>,
        size_pct: Option<f64>,
        stop_loss: f64,
        slippage: f64,
        commission: f64,
    ) {
        let qty = self.position_size(price, stop_loss, risk_pct, size_pct);
        if qty <= 0.0 {
            return;
        }

        let exec = price * (1.0 - slippage); // 卖空成交价
        let proceeds = exec * qty;

        let fee = proceeds * commission;

        // // ---- FIXED: 卖空开仓不计入盈利，只扣手续费
        // self.capital -= fee;

        // self.position = Some(TradePosition::Short {
        //     quantity: qty,
        //     entry_price: exec,
        //     held_bars: 0,
        // });
        // self.trade_log
        //     .push((ts.to_string(), Signal::Sell, exec, None, self.capital));

        // println!(
        //     "Trade Log - Timestamp: {}, Signal: {:?}, Price: {}, Profit: {:?}, Capital: {}",
        //     ts,
        //     Signal::Sell,
        //     exec,
        //     None::<f64>,
        //     self.capital
        // );
    }

    fn close_short(
        &mut self,
        ts: &str,
        price: f64,
        qty: f64,
        entry: f64,
        slippage: f64,
        commission: f64,
    ) {
        let exec = price * (1.0 + slippage); // 买回平仓价
        let pnl = (entry - exec) * qty; // ----  盈亏公式
        let fee = exec * qty * commission;

        // 只把净盈亏加入资本
        // self.capital += pnl - fee;
        // self.trade_log
        //     .push((ts.to_string(), Signal::Buy, exec, Some(pnl), self.capital));
        // self.position = None;

        // println!(
        //     "Trade Log - Timestamp: {}, Signal: {:?}, Price: {}, Profit: {:?}, Capital: {}",
        //     ts,
        //     Signal::Buy,
        //     exec,
        //     Some(pnl),
        //     self.capital
        // );
    }

    pub fn get_backtest_result(&self) -> Option<BacktestResult> {
        if !self.finish_running {
            return None;
        }

        let mut balances = Vec::new();
        let mut trades = Vec::new();
        for (ts, signal, _price, profit_opt, capital_after) in &self.trade_log {
            // if Signal::Sell == *signal {
            //     balance.push(Balance {
            //         date: date.to_string(),
            //         capital: *cur_captial,
            //         // market: 100.0,
            //         trades: 1,
            //     });
            // }
            // let trade = match signal {
            //     Signal::Buy => None,
            //     Signal::Sell => profit.map(|p| Trade {
            //         date: date.to_string(),
            //         trade_type: TradeType::Sell,
            //         profit: p,
            //         result: if p > 0.0 {
            //             TradeResultType::Win
            //         } else {
            //             TradeResultType::Loss
            //         },
            //     }),
            //     _ => None, // Ignore other signals
            // };

            // if let Some(trade) = trade {
            //     trades.push(trade);
            // }

            // === ① 记录 balance：凡是平仓（profit 有值）就 push
            if let Some(_) = profit_opt {
                balances.push(Balance {
                    date: ts.clone(),
                    capital: *capital_after,
                    trades: 1,
                });
            }

            // === ② 记录 trade 详情，用 profit 正负判断胜负
            // if let Some(pnl) = profit_opt {
            //     trades.push(Trade {
            //         date: ts.clone(),
            //         trade_type: match signal {
            //             Signal::Buy => TradeType::Buy,
            //             Signal::Sell => TradeType::Sell,
            //             _ => TradeType::Sell, // fallback
            //         },
            //         profit: *pnl,
            //         result: if *pnl > 0.0 {
            //             TradeResultType::Win
            //         } else {
            //             TradeResultType::Loss
            //         },
            //     });
            // }
        }

        Some(BacktestResult {
            total_return: self.total_return(),
            cagr: self.cagr(1.0),
            max_drawdown: self.max_drawdown(),
            sharpe_ratio: self.sharpe_ratio(0.05),
            profit_factor: self.profit_factor(),
            win_rate: self.win_rate(),
            final_capital: self.capital,
            balances,
            trades,
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
        let (mut profit, mut loss) = (0.0, 0.0);
        for (_, _, _, pnl_opt, _) in &self.trade_log {
            if let Some(p) = pnl_opt {
                if *p > 0.0 {
                    profit += p
                } else {
                    loss += p.abs()
                }
            }
        }
        if loss == 0.0 {
            f64::INFINITY
        } else {
            profit / loss
        }
    }

    /// **Win Rate (%)**
    pub fn win_rate(&self) -> f64 {
        // let mut total_trades = 0.0;
        // let mut winning_trades = 0.0;

        // let mut last_entry: Option<f64> = None;

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

        // if total_trades == 0.0 {
        //     return 0.0;
        // }

        // (winning_trades / total_trades) * 100.0

        let mut wins = 0.0;
        let mut total = 0.0;

        for (_, _, _, profit_opt, _) in &self.trade_log {
            if let Some(pnl) = profit_opt {
                total += 1.0;
                if *pnl > 0.0 {
                    wins += 1.0;
                }
            }
        }

        if total == 0.0 {
            0.0
        } else {
            wins / total * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        data::coin_market::CoinsMarket,
        engine::parameters::{StrategyRunParameters, StrategyRunParams},
        indicators::moving_average::MovingAverageType,
        strategy::strategy_type::StrategyType,
    };

    use super::*;

    #[test]
    fn test_lab_backtest() {
        let params = StrategyRunParameters {
            ma_type: None,
            fast_period: Some(15),             // 增加快线周期，减少噪声
            slow_period: Some(50),             // 增加慢线周期，捕捉更长期趋势
            entry_threshold: Some(0.02),       // 提高入场阈值，减少噪声信号
            exit_threshold: Some(0.01),        // 保持较低的退出阈值，快速止损
            position_size: Some(0.2),          // 每笔交易使用 20% 的资本
            max_concurrent_positions: Some(1), // 限制同时持仓数量为 1
            slippage: Some(0.001),             // 滑点保持为 0.1%
            commission: Some(0.001),           // 手续费保持为 0.1%
            entry_delay: Some(0),              // 无入场延迟
            min_holding_period: Some(1),       // 最小持仓时间为 1 个周期
            max_holding_period: Some(20),      // 最大持仓时间增加到 20 个周期
            risk_per_trade: Some(0.01),        // 每笔交易风险降低到 1%
            stop_loss: Some(0.05),             // 止损比例降低到 5%
            take_profit: Some(0.1),            // 保持止盈比例为 10%
            band_multiplier: None,
            cooldown_period: None,
            entry_z_score: None,
            exit_z_score: None,
            position_type: None,
            mean_type: None,
            reversion_style: None,
            lookback_period: None,
        };
        let run_lab_strategy = RunLabStrategy {
            r#type: "ma".to_string(),
            sub_type: Some("sma".to_string()),
            strategy_run_params: params,
            initial_capital: 1000000.0,
            position_type: "short".to_string(),
        };
        let mut backtester = LabObserver::new(run_lab_strategy);

        let ohlcv_datas = CoinsMarket::get_coin_ohlcv("bitcoin");
        if let Ok(ohlcv) = ohlcv_datas {
            let market_data: Vec<MarketData> = ohlcv
                .iter()
                .map(|ohlc| MarketData {
                    timestamp: ohlc.timestamp.to_rfc3339(),
                    close_price: ohlc.close,
                })
                .collect();
            assert!(!market_data.is_empty(), "Market data should not be empty");

            backtester.run(market_data);
            backtester.report();
            backtester.print_summary(1.0, 0.05);

            let result = backtester.get_backtest_result();
            println!(
                "Backtest result: {}",
                serde_json::to_string_pretty(&result).unwrap()
            );

            assert!(
                backtester.capital > 0.0,
                "Final capital should be greater than 0"
            );
            assert!(
                !backtester.trade_log.is_empty(),
                "Trade log should not be empty"
            );
        } else {
            panic!("Failed to fetch OHLCV data");
        }
    }
}
