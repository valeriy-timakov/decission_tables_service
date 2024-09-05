
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ConditionType {
    Equal,
    MoreThan,
    MoreEqualThan,
    LessThan,
    LessEqualThan,
    Interval,
}

#[derive(PartialEq, Debug)]
pub struct Condition<T: PartialEq + PartialOrd> {
    condition_type: ConditionType,
    value: T
}

impl <T: PartialEq + PartialOrd> Condition<T> {
    pub fn new(condition_type: ConditionType, value: T) -> Self {
        Self {condition_type, value}
    }

    pub fn check(&self, value: T) -> bool {
        match self.condition_type {
            ConditionType::Equal => self.value == value,
            ConditionType::MoreThan => { self.value > value },
            ConditionType::MoreEqualThan => { self.value >= value }
            ConditionType::LessThan => { self.value < value }
            ConditionType::LessEqualThan => { self.value <= value }
            ConditionType::Interval => { false }
        }
    }
}

pub struct Rule<T: PartialEq + PartialOrd> {
    condition: Condition<T>,
    result_id: usize,
}