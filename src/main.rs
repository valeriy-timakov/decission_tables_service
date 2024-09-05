fn main() {
    let conditions: Vec<Box<dyn Condition>> = vec![
        Box::new(GreaterThan { threshold: 10 }),
        Box::new(Equals { target: 20 }),
    ];

    let parser = Parser::new(|s| ParsedType::Integer(s.parse().unwrap()), conditions);

    let input = "20";
    let result = parser.parse_and_check(input);

    println!("Result: {}", result); // Output: true, якщо всі умови задоволені
}


use std::str::FromStr;

trait Condition {
    fn check(&self, value: &dyn ToString) -> bool;
}

struct GreaterThan<T: PartialOrd> {
    threshold: T,
}

impl<T: PartialOrd + ToString + FromStr> Condition for GreaterThan<T> {
    fn check(&self, value: &dyn ToString) -> bool {
        value.to_string().parse::<T>().map_or(false, |v| v > self.threshold)
    }
}

struct Equals<T: PartialEq> {
    target: T,
}

impl<T: PartialEq + ToString + FromStr> Condition for Equals<T> {
    fn check(&self, value: &dyn ToString) -> bool {
        value.to_string().parse::<T>().map_or(false, |v| v == self.target)
    }
}


enum ParsedType {
    Integer(i32),
    Float(f64),
    Text(String),
}

impl ToString for ParsedType {
    fn to_string(&self) -> String {
        match self {
            ParsedType::Integer(i) => i.to_string(),
            ParsedType::Float(f) => f.to_string(),
            ParsedType::Text(s) => s.clone(),
        }
    }
}

struct Parser {
    conditions: Vec<Box<dyn Condition>>,
    parse_type: fn(&str) -> ParsedType,
}

impl Parser {
    fn new(parse_type: fn(&str) -> ParsedType, conditions: Vec<Box<dyn Condition>>) -> Self {
        Self { parse_type, conditions }
    }

    fn parse_and_check(&self, input: &str) -> bool {
        let parsed_value = (self.parse_type)(input);
        self.conditions.iter().all(|cond| cond.check(&parsed_value))
    }
}












extern crate core;

fn main() {
    println!("Hello, world!");
    let c = C::new();
    let d = D::new();
    c.do_something();
    d.do_something();
    call_do_something(c);
    call_do_something(d);
}

fn call_do_something<T: B>(t: T) {
    t.do_something();
}

trait A {
    fn create() -> Self where Self: Sized;
}

trait B : A {
    fn new() -> Self where Self: Sized {
        let res = Self::create();
        res.do_something();
        res
    }
    fn do_something(&self){
        println!("B");
    }
}

struct C {}

impl C {
}

impl A for C {
    fn create() -> Self {
        C {}
    }
}

impl B for C {
    fn do_something(&self) {
        println!("C");
    }}

struct D {}

impl A for D {
    fn create() -> Self {
        D {}
    }
}

impl B for D {
    fn do_something(&self) {
        println!("D");
    }}

impl D {
}