use serde::{Deserialize, Serialize};

pub type CustomerId = u64;
pub type SkuId = u64;
pub type OrderId = u64;

pub type MoneyCents = i64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderLine {
    pub sku_id: SkuId,
    pub qty: u32,
    pub price_cents: MoneyCents,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OrderPlaced {
    pub order_id: OrderId,
    pub customer_id: CustomerId,
    pub lines: Vec<OrderLine>,
    pub ts_ms: u64,
}

impl OrderPlaced {
    pub fn total_cents(&self) -> MoneyCents {
        self.lines
            .iter()
            .map(|l| l.price_cents.saturating_mul(l.qty as i64))
            .sum()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RetailEvent {
    OrderPlaced(OrderPlaced),
    InventoryAdjusted { sku_id: SkuId, delta_qty: i32, ts_ms: u64 },
}

