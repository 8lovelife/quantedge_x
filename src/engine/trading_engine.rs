use crate::{
    data::data_feed::DataFeed,
    domain::{RiskManager, executor::OrderExecutor},
    processor::SignalProcessor,
    strategy::{strategy_context::StrategyContext, strategy_trait::Strategy},
};

/// 主交易引擎：驱动数据源、风控、策略决策和信号执行
pub struct TradingEngine<DF, EX>
where
    DF: DataFeed,
    EX: OrderExecutor,
{
    /// 风控管理器（trait object）
    risk_manager: Box<dyn RiskManager>,
    /// 行情或历史数据源
    datafeed: DF,
    /// 策略实例
    strategy: Box<dyn Strategy>,
    /// 信号处理器：负责下单 & 通知 Observer
    processor: SignalProcessor<EX>,
    /// 策略上下文：持仓、资金和当前入场信息
    strategy_context: StrategyContext,
}

impl<DF, EX> TradingEngine<DF, EX>
where
    DF: DataFeed,
    EX: OrderExecutor,
{
    /// 构造交易引擎
    /// 传入已配置好的 SignalProcessor，方便自定义 Observer
    pub fn new(
        risk_manager: Box<dyn RiskManager>,
        datafeed: DF,
        strategy: Box<dyn Strategy>,
        processor: SignalProcessor<EX>,
        initial_equity: f64,
    ) -> Self {
        let engine = Self {
            risk_manager,
            datafeed,
            strategy,
            processor,
            strategy_context: StrategyContext::new(initial_equity),
        };
        engine
    }

    /// 链式注册新的 Observer
    pub fn with_observer(mut self, observer: Box<dyn crate::domain::TradeObserver>) -> Self {
        self.processor.add_observer(observer);
        self
    }

    /// 运行引擎：循环拉取行情，执行风控与策略信号
    pub fn run(&mut self) {
        let ctx = &mut self.strategy_context;
        while let Some(data) = self.datafeed.next() {
            // 1. 同步仓位
            ctx.position = self.processor.sync_positions();

            // 2. 风控优先：止损/止盈检查
            let entry_price: Option<f64> = ctx.current_entry.as_ref().map(|e| e.1);
            if let Some(sig) =
                self.risk_manager
                    .check_risk(data.close_price, ctx.position, entry_price)
            {
                self.processor.process(sig, ctx, &data);
                continue;
            }

            // 3. 策略决策
            let sig = self.strategy.on_tick(ctx, &data);

            // 4. 执行信号
            self.processor.process(sig, ctx, &data);
        }
    }
}
