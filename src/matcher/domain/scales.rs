use super::{price_ticks::PriceTicks, qty_lots::QtyLots};

#[derive(Debug, Clone, Copy)]
pub struct Scales {
    pub tick_size: i64,
    pub lot_size: i64,
}

impl Scales {
    pub const EPS: f64 = 1e-9;
    pub fn new(tick_size: i64, lot_size: i64) -> Self {
        Scales {
            tick_size,
            lot_size,
        }
    }
    pub fn to_ticks(&self, px: f64) -> PriceTicks {
        PriceTicks((px * self.tick_size as f64).round() as i64)
    }
    pub fn to_lots(&self, q: f64) -> QtyLots {
        QtyLots((q * self.lot_size as f64).round() as i64)
    }
    pub fn ticks_to_f64(&self, t: PriceTicks) -> f64 {
        t.0 as f64 / self.tick_size as f64
    }
    pub fn lots_to_f64(&self, l: QtyLots) -> f64 {
        l.0 as f64 / self.lot_size as f64
    }

    pub fn to_ticks_strict_str(&self, px_str: &str) -> Result<PriceTicks, String> {
        let px: f64 = px_str
            .parse()
            .map_err(|e| format!("bad price '{}': {e}", px_str))?;
        self.to_ticks_strict(px)
    }
    pub fn to_lots_strict_str(&self, q_str: &str) -> Result<QtyLots, String> {
        let q: f64 = q_str
            .parse()
            .map_err(|e| format!("bad qty '{}': {e}", q_str))?;
        self.to_lots_strict(q)
    }

    pub fn to_ticks_strict(&self, px: f64) -> Result<PriceTicks, String> {
        if !self.is_price_aligned(px) {
            return Err(format!("price {} not aligned to 1/{}", px, self.tick_size));
        }
        Ok(PriceTicks((px * self.tick_size as f64).round() as i64))
    }
    pub fn to_lots_strict(&self, q: f64) -> Result<QtyLots, String> {
        if !self.is_qty_aligned(q) {
            return Err(format!("qty {} not aligned to 1/{}", q, self.lot_size));
        }
        Ok(QtyLots((q * self.lot_size as f64).round() as i64))
    }

    pub fn is_price_aligned(&self, px: f64) -> bool {
        let scaled = px * self.tick_size as f64;
        (scaled - scaled.round()).abs() <= Self::EPS
    }

    pub fn is_qty_aligned(&self, q: f64) -> bool {
        let scaled = q * self.lot_size as f64;
        (scaled - scaled.round()).abs() <= Self::EPS
    }
}
