#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Date {
    pub year: u32,
    pub month: u8,
    pub day: u8,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Duration {
    pub sign: i8,
    pub weeks: u64,
    pub days: u64,
    pub seconds: u64,
}

impl Default for Duration {
    fn default() -> Self {
        Duration {
            sign: 1,
            weeks: 0,
            days: 0,
            seconds: 0,
        }
    }
}
