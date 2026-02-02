use super::backtest_result::{BacktestResult, Balance, Trade, TradeResultType, TradeType};
use crate::domain::trade_observer::{TradeObserver, TradeRecord};
use crate::strategy::direction::Direction;
use chrono::{DateTime, Utc};
use std::cell::RefCell;
use std::rc::Rc;

/// 单线程版性能观察者：内部使用 Rc<RefCell<…>> 共享数据
#[derive(Clone, Debug)]
pub struct PerformanceObserver {
    trades: Rc<RefCell<Vec<Trade>>>,
    balances: Rc<RefCell<Vec<Balance>>>,
    current_capital: Rc<RefCell<f64>>,
}

impl PerformanceObserver {
    /// 初始化时传入初始资金，会记录第一条余额快照
    pub fn new(initial_capital: f64) -> Self {
        let trades = Rc::new(RefCell::new(Vec::new()));
        let mut init = Vec::new();
        // init.push(Balance {
        //     date: Utc::now().to_rfc3339(),
        //     capital: initial_capital,
        //     trades: 0,
        // });
        let balances = Rc::new(RefCell::new(init));
        let current_capital = Rc::new(RefCell::new(initial_capital));
        Self {
            trades,
            balances,
            current_capital,
        }
    }

    /// 回测结束后调用，计算并返回所有绩效指标
    pub fn finalize(&self) -> BacktestResult {
        // 1. 克隆出交易和余额数据
        let trades: Vec<Trade> = (*self.trades.borrow()).clone();
        let balances: Vec<Balance> = (*self.balances.borrow()).clone();
        let cap = *self.current_capital.borrow();
        // // 如果没有任何余额快照，则添加一条初始快照
        // if balances.is_empty() {
        //     balances.push(Balance {
        //         date: Utc::now().to_rfc3339(),
        //         capital: cap,
        //         trades: trades.len() as u32,
        //     });
        // }
        // 2. 计算总收益
        let start = balances.first().map(|b| b.capital).unwrap_or(cap);
        let end = balances.last().unwrap().capital;
        assert!(start > 0.0, "Initial capital must be positive");
        let total_return = if start > 0.0 {
            (end / start) - 1.0
        } else {
            0.0
        };

        // let total_return = (end / start) - 1.0;

        // 3. 年化收益（CAGR）
        let first_date = DateTime::parse_from_rfc3339(&balances.first().unwrap().date)
            .unwrap()
            .with_timezone(&Utc);
        let last_date = DateTime::parse_from_rfc3339(&balances.last().unwrap().date)
            .unwrap()
            .with_timezone(&Utc);
        let years = (last_date - first_date).num_days() as f64 / 365.0;
        let cagr = if years > 0.0 {
            (1.0 + total_return).powf(1.0 / years) - 1.0
        } else {
            0.0
        };

        // 4. 最大回撤
        let mut peak = start;
        let max_drawdown = balances.iter().fold(0.0_f64, |md, b| {
            if b.capital > peak {
                peak = b.capital;
            }
            let dd = if peak > 0.0 {
                (peak - b.capital) / peak
            } else {
                0.0
            };
            md.max(dd)
        });

        // 5. 胜率 & 盈亏比
        let wins: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.result == TradeResultType::Win)
            .collect();
        let losses: Vec<&Trade> = trades
            .iter()
            .filter(|t| t.result == TradeResultType::Loss)
            .collect();
        let win_rate = if !trades.is_empty() {
            wins.len() as f64 / trades.len() as f64
        } else {
            0.0
        };
        let gross_win: f64 = wins.iter().map(|t| t.profit).sum();
        let gross_loss: f64 = losses.iter().map(|t| t.profit.abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_win / gross_loss
        } else {
            0.0
        };

        // 6. 夏普比率（按日收益）
        let daily_returns: Vec<f64> = balances
            .windows(2)
            .filter_map(|w| {
                let prev = w[0].capital;
                let curr = w[1].capital;
                if prev > 0.0 {
                    Some((curr / prev) - 1.0)
                } else {
                    None
                }
            })
            .collect();
        let avg = if !daily_returns.is_empty() {
            daily_returns.iter().sum::<f64>() / daily_returns.len() as f64
        } else {
            0.0
        };
        let std = if daily_returns.len() > 1 {
            (daily_returns.iter().map(|r| (r - avg).powi(2)).sum::<f64>()
                / (daily_returns.len() - 1) as f64)
                .sqrt()
        } else {
            0.0
        };
        let sharpe_ratio = if std > 0.0 {
            avg * (daily_returns.len() as f64).sqrt() / std
        } else {
            0.0
        };

        BacktestResult {
            total_return,
            cagr,
            max_drawdown,
            sharpe_ratio,
            profit_factor,
            win_rate,
            final_capital: end,
            trades,
            balances,
        }
    }
}

impl TradeObserver for PerformanceObserver {
    fn on_trade(&mut self, record: &TradeRecord) {
        // 更新资金
        *self.current_capital.borrow_mut() += record.pnl;
        // 记录交易
        self.trades.borrow_mut().push(Trade {
            date: record.exit_time.clone(),
            trade_type: if record.direction == Direction::Long {
                TradeType::Buy
            } else {
                TradeType::Sell
            },
            result: if record.pnl >= 0.0 {
                TradeResultType::Win
            } else {
                TradeResultType::Loss
            },
            profit: record.pnl,
        });
        // 记录余额快照
        let cap = *self.current_capital.borrow();
        let count = self.trades.borrow().len() as u32;
        self.balances.borrow_mut().push(Balance {
            date: record.exit_time.clone(),
            capital: cap,
            trades: count,
        });
    }
}
