extern crate core;

fn main() {
    println!("Hello, world!");
    // let c = C::new();
    // let d = D::new();
    // c.do_something();
    // d.do_something();
    // call_do_something(c);
    // call_do_something(d);
}

trait Condition<T: PartialEq + PartialOrd> {
    fn check(&self, value1: &T, value2: &T) -> bool;
}

struct EqualCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for EqualCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 == value2
    }
}

struct MoreThanCondition {}

impl<T: PartialEq + PartialOrd> Condition<T> for MoreThanCondition {
    fn check(&self, value1: &T, value2: &T) -> bool {
        value1 > value2
    }
}

struct Rule<T: PartialEq + PartialOrd> {
    condition: Box<dyn Condition<T>>,
    values: Box<[T]>,
}

impl<T: PartialEq + PartialOrd> Rule<T> {
    fn check(&self, index: usize, value: &T) -> bool {
        self.condition.check(&self.values[index], &value)
    }
}

struct DecisionTable {}
