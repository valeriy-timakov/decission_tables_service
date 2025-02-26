extern crate core;


use calamine::{DataType, Reader};
use decision_tables_service::decession_table::{DecisionTable, DecisionTableSource, RuleTrait};
use decision_tables_service::xlsx_dt_data_source::{ParsePreferences, XlsxDTDataSource};
use simd_json::borrowed::Value as JsonValue;
use std::time::Instant;


fn main() {
    let mut data = br#"{
        "parameters": {
            "salesProgramCode": 600004,  					
            "acquisitionType": "NEW",    
            "creditContractYear": "Default", 
            "signatory": "AGENT", 
            "signingOption": "ELECTRONIC", 
            "salesman": "AGENT_NETWORK"
        }
    }"#.to_vec();

    // Парсимо JSON
    let mut parsed: JsonValue = simd_json::from_slice(&mut data).expect("Failed to parse JSON");


    let now = Instant::now();
        
    let parse_preferences = ParsePreferences::new(
        ',', DataType::String("*".to_string()), "%d.%m.%YT%H:%M:%S".to_string(), Some(6));
    let mut loader = XlsxDTDataSource::new(
        "D:\\Downloads\\1\\EnterPrintouts_FullRights.xlsx".to_string(), parse_preferences);
    loader.init().unwrap();
    
    let dt = DecisionTable::create(Box::new(loader)).unwrap();


    let nanos = now.elapsed().as_nanos(); // Час, що минув з моменту створення `now`

    println!("Initializing, nanoseconds: {}", nanos);

    let now = Instant::now();
    
    match dt.check_all(&parsed["parameters"]) {
        Ok(res) => {
            print!("Results: (");
            res.iter().for_each(|x| print!("'{}', ", x));
            println!(")");
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    let nanos = now.elapsed().as_nanos(); // Час, що минув з моменту створення `now`

    println!("Processing, nanoseconds: {}", nanos);
}
