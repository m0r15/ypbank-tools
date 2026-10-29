pub enum OpKind {
    Income,
    Expence,
}

pub struct Amount(pub i64);

pub struct Date {
    pub year: u32,
    pub month: u32,
    pub day: u32,
}

struct Operation {
    date: Date,
    category: String,
    kind: OpKind,
    amount: Amount,
}
