use std::string::ToString;
use calamine::DataType;
use crate::decession_table::{DecisionTable, DecisionTableSource};
use crate::xlsx_dt_data_source::{ParsePreferences, XlsxDTDataSource};

use std::sync::{Mutex, LazyLock, Arc};
use simd_json::borrowed::Value as JsonValue;

static INSTANCES: LazyLock<Mutex<Vec<Arc<DecisionTable>>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
use jni::objects::{JClass, JString, JObject};
use jni::sys::{jint};
use jni::JNIEnv;
use jni::sys::jobjectArray;


#[no_mangle]
pub extern "system" fn Java_net_home_decision_1tables_1service_DecisionTablesService_createDecisionTable<'local>(
    mut env: JNIEnv<'local>,      
    _: JClass<'local>,   
    dt_file_path: JString<'local>, 
    data_start_idx: jint  
) -> jint {
    println!("Start running load dt...");
    let dt_file_path: String = match env.get_string(&dt_file_path) {
        Ok(s) => s.into(),
        Err(e) => {
            env.throw_new("java/lang/RuntimeException", e.to_string().as_str())
                .expect("Error throwing exception!");
            return -1
        }, 
    };
    println!("path parsed: {}", dt_file_path);

    let data_start_idx = if data_start_idx < 1 { None } else { Some(data_start_idx as usize) };
    println!("index parsed: {:?}", data_start_idx);

    let res = match create_dt_inner(dt_file_path, data_start_idx) {
        Ok(idx) => idx as jint,
        Err(e) => {
            env.throw_new("java/lang/RuntimeException", e.as_str())
                .expect("Error throwing exception!");
            return -1
        }, 
    };

    println!("td loaded success! ids: {:?}", res);
    res
}

#[no_mangle]
pub extern "C" fn Java_net_home_decision_1tables_1service_DecisionTablesService_executeRequest<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    dt_index: usize, 
    request:  JString<'local>, 
) -> jobjectArray {
    let mut rust_string: String = env.get_string(&request).expect("Не вдалося отримати JString").into();

    let string_class = match env.find_class("java/lang/String") {
        Ok(class) => class,
        Err(e) => {
            env.throw_new("java/lang/RuntimeException", e.to_string().as_str())
                .expect("Error throwing exception!");
            return std::ptr::null_mut();
        }
    };
    let res = unsafe {
        let request_byes = rust_string.as_bytes_mut();
        match execute_dt_inner(dt_index, request_byes) {
            Ok(res) => res,
            Err(e) => {
                env.throw_new("java/lang/RuntimeException", e.as_str())
                    .expect("Error throwing exception!");
                return std::ptr::null_mut();
            }
        }
    };

    // Створюємо новий масив Java String[]
    let array = match env.new_object_array(res.len() as i32, string_class, JObject::null()) {
        Ok(array) => array,
        Err(e) => {
            env.throw_new("java/lang/RuntimeException", e.to_string().as_str())
                .expect("Error throwing exception!");
            return std::ptr::null_mut();
        }
    };

    for (i, s) in res.iter().enumerate() {
        let jstring = match env.new_string(s) {
            Ok(jstring) => jstring,
            Err(e) => {
                env.throw_new("java/lang/RuntimeException", e.to_string().as_str())
                    .expect("Error throwing exception!");
                return std::ptr::null_mut();
            }
        };
        match env.set_object_array_element(&array, i as i32, jstring) {
            Ok(_) => (),
            Err(e) => {
                env.throw_new("java/lang/RuntimeException", e.to_string().as_str())
                    .expect("Error throwing exception!");
                return std::ptr::null_mut();
            }
        };
    }

    array.into_raw()
}




fn create_dt_inner(dt_file_path: String, data_start_idx: Option<usize>) -> Result<usize, String> {
    let parse_preferences = ParsePreferences::new(
        ',', DataType::String("*".to_string()), "%d.%m.%YT%H:%M:%S".to_string(), data_start_idx);
    let mut loader = XlsxDTDataSource::new(dt_file_path, parse_preferences);
    loader.init()?;
    let dt = DecisionTable::create(Box::new(loader))?;

    let mut instances = INSTANCES.lock()
        .map_err(|e| {format!("Error locking instances! {}", e) })?;
    instances.push(Arc::new(dt));

    Ok(instances.len() - 1)
}

fn get_dt(dt_index: usize) -> Result<Arc<DecisionTable>, String> {
    let instances = INSTANCES.lock()
        .map_err(|e| {format!("Error locking instances! {}", e) })?;
    let dt = instances.get(dt_index)
        .ok_or(format!("No decision table with idx {}!", dt_index))?;
    Ok(Arc::clone(dt))
}

fn execute_dt_inner(dt_index: usize, request:  &mut [u8]) -> Result<Vec<String>, String> {
    let request_parsed: JsonValue = simd_json::from_slice(request).expect("Failed to parse JSON");
    let dt = get_dt(dt_index)?;
    dt.check_all_parallel(&request_parsed)
        .map(|v| v.iter().map(|x| x.to_string()).collect())
}
 