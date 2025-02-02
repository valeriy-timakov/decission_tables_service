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

impl<T: PartialEq + PartialOrd> Rule<T> {
    fn checkOne(&self, index: usize, value: &T) -> bool {
        self.condition.check(&self.values[index], &value)
    }
}

impl<'a, T: PartialEq + PartialOrd + TryFrom<JsonValue<'a>, Error = String>> Rule<T> {
    fn checkAll(&self, index: usize, value: &JsonValue<'a>) -> Result<Vec<usize>, String> {
        let t_value = T::try_from(value.clone())?;
        let mut result: Vec<usize> = Vec::new();
        for (index, value) in self.values.iter().enumerate() {
            if self.checkOne(index, &t_value) {
                result.push(index);
            }
        }
        Ok(result)
    }
}



struct DecisionTable {}
