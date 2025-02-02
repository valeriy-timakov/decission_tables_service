extern crate core;

use simd_json::borrowed::Value as JsonValue;
use simd_json::prelude::ValueAsScalar;

fn main() {
    println!("Hello, world!");

    let mut data = br#"{
        "parameters": {
            "name": "someName",  
            "age": 23      
        }
    }"#.to_vec();

    // Парсимо JSON
    let mut parsed: JsonValue = simd_json::from_slice(&mut data).expect("Failed to parse JSON");

    // Дістаємо значення "age"
    if let Some(age) = parsed["parameters"]["age"].as_i64() {
        println!("Age: {}", age);

        let equal_rule = Rule {
            condition: Box::new(EqualCondition {}),
            values: vec![23, 25, 30, 35, 40].into_boxed_slice(),
        };

        let more_than_rule = Rule {
            condition: Box::new(MoreThanCondition {}),
            values: vec![18, 20, 22, 25, 30].into_boxed_slice(),
        };

        match equal_rule.checkAll(0, &parsed["parameters"]["age"]) {
            Ok(indices) => println!("Equal rule matched at indices: {:?}", indices),
            Err(e) => println!("Error: {}", e),
        }

        match more_than_rule.checkAll(0, &parsed["parameters"]["age"]) {
            Ok(indices) => println!("More than rule matched at indices: {:?}", indices),
            Err(e) => println!("Error: {}", e),
        }
    } else {
        println!("Field 'age' not found or not an integer");
    }
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

trait Parser<T: PartialEq + PartialOrd> {
    fn parse(json: &JsonValue) -> Option<T>;
}

impl Parser<i64> for Rule<i64> {
    fn parse(json: &JsonValue) -> Option<i64> {
        json.as_i64()
    }
}

impl<T: PartialEq + PartialOrd> Rule<T> {
    fn checkOne(&self, index: usize, value: &T) -> bool {
        self.condition.check(&self.values[index], &value)
    }
}

impl<'a, T: PartialEq + PartialOrd> Rule<T> {
    fn checkAll(&self, index: usize, value: &JsonValue<'a>) -> Result<Vec<usize>, String> {
        let t_value = parse(value.clone())?;
        let mut result: Vec<usize> = Vec::new();
        for (index, value) in self.values.iter().enumerate() {
            if self.checkOne(index, &t_value) {
                result.push(index);
            }
        }
        Ok(result)
    }
}

impl<'a> TryFrom<JsonValue<'a>> for i64 {
    type Error = &'static str;

    fn try_from(value: JsonValue<'a>) -> Result<Self, Self::Error> {
        todo!()
    }
}

struct DecisionTable {}
