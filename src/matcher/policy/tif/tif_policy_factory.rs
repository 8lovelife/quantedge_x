use chrono::Utc;

use crate::matcher::{
    book::book_ops::OrderBookOps,
    domain::{
        price_ticks::PriceTicks, qty_lots::QtyLots, tif_result::TifResult,
        time_in_force::TimeInForce,
    },
    policy::tif::{
        fok_policy::FokPolicy, gtc_policy::GtcPolicy, gtt_policy::GttPolicy, ioc_policy::IocPolicy,
        tif_policy::TifPolicy,
    },
};

pub enum AnyTifPolicy {
    FOK(FokPolicy),
    IOC(IocPolicy),
    GTC(GtcPolicy),
    GTT(GttPolicy),
}

impl TifPolicy for AnyTifPolicy {
    fn execute_buy<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        match self {
            AnyTifPolicy::IOC(p) => p.execute_buy(book, limit, want),
            AnyTifPolicy::FOK(p) => p.execute_buy(book, limit, want),
            AnyTifPolicy::GTC(p) => p.execute_buy(book, limit, want),
            AnyTifPolicy::GTT(p) => p.execute_buy(book, limit, want),
        }
    }

    fn execute_sell<T: OrderBookOps>(
        &self,
        book: &mut T,
        limit: Option<PriceTicks>,
        want: QtyLots,
    ) -> anyhow::Result<TifResult> {
        match self {
            AnyTifPolicy::IOC(p) => p.execute_sell(book, limit, want),
            AnyTifPolicy::FOK(p) => p.execute_sell(book, limit, want),
            AnyTifPolicy::GTC(p) => p.execute_sell(book, limit, want),
            AnyTifPolicy::GTT(p) => p.execute_sell(book, limit, want),
        }
    }
}

pub fn obtain_tif_policy(tif: TimeInForce) -> AnyTifPolicy {
    match tif {
        TimeInForce::IOC => AnyTifPolicy::IOC(IocPolicy),
        TimeInForce::FOK => AnyTifPolicy::FOK(FokPolicy),
        TimeInForce::GTC => AnyTifPolicy::GTC(GtcPolicy),
        TimeInForce::GTT(_exp) => AnyTifPolicy::GTT(GttPolicy {
            expires_at: Utc::now(),
        }),
    }
}
