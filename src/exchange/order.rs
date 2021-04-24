#[derive(Clone)]
pub enum OrderKind {
    Market,
    Limit,
}

impl std::fmt::Display for OrderKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            OrderKind::Market => format!("market"),
            OrderKind::Limit => format!("limit")
        })
    }
}

#[derive(Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            OrderSide::Buy => format!("buy"),
            OrderSide::Sell => format!("sell")
        })
    }
}

#[derive(Clone)]
pub struct Order {
    pub symbol: String,
    pub order_id: String,
    pub kind: Option<OrderKind>,
    pub side: Option<OrderSide>,
    pub health: i8,
    pub alive: bool,
}

impl Order {
    pub fn lower_health(&mut self) {
        self.health -= 1;
        if self.health <= 0 {
            self.alive = false;
        }
    }
}
