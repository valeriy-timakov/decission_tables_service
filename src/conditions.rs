pub trait Condition<T>: Send + Sync
where 
    T: PartialEq + PartialOrd, 
{
    fn check(&self, value1: &T, value2: &T) -> bool;
}

pub struct EqualCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for EqualCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 == value2
    }
}

pub struct GreaterThanCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for GreaterThanCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 > value2
    }
}

pub struct GreaterThanOrEqualCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for GreaterThanOrEqualCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 >= value2
    }
}

pub struct LessThanCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for LessThanCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 < value2
    }
}

pub struct LessThanOrEqualCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for LessThanOrEqualCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 <= value2
    }
} 