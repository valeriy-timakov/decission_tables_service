use std::process::id;
use std::string::ToString;
use calamine::DataType;
use crate::decession_table::{DecisionTable, DecisionTableSource};
use crate::xlsx_dt_data_source::{ParsePreferences, XlsxDTDataSource};

use std::sync::{Mutex, LazyLock, Arc};
use simd_json::borrowed::Value as JsonValue;

static INSTANCES: LazyLock<Mutex<Vec<Arc<DecisionTable>>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static LAST_ERROR: Mutex<Option<String>> = Mutex::new(None);
use jni::objects::{JClass, JString, JObjectArray, JObject};
use jni::sys::{jint};
use jni::JNIEnv;
use jni::sys::jobjectArray;

#[no_mangle]
pub extern "system" fn Java_org_example_Main_test<'local>(_: JNIEnv<'local>, _: JClass<'local>, a: jint, b: jint) -> jint {
    a + b
}

#[no_mangle]
pub extern "system" fn Java_org_example_Main_createDecisionTable<'local>(
    mut env: JNIEnv<'local>,      
    _: JClass<'local>,   
    dt_file_path: JString<'local>, 
    data_start_idx: jint  
) -> jint {
    println!("Start running load dt...");
    let dt_file_path: String = match env.get_string(&dt_file_path) {
        Ok(s) => s.into(),
        Err(e) => {
            throw_java_exception(env, e.to_string().as_str());
            return -1
        }, 
    };
    println!("path parsed: {}", dt_file_path);

    let data_start_idx = if data_start_idx < 1 { None } else { Some(data_start_idx as usize) };
    println!("index parsed: {:?}", data_start_idx);

    let res = match create_dt_inner(dt_file_path, data_start_idx) {
        Ok(idx) => idx as jint,
        Err(e) => {
            throw_java_exception(env, e.as_str());
            return -1
        }, 
    };

    println!("td loaded success! ids: {:?}", res);
    res
}
#[no_mangle]
pub extern "C" fn Java_org_example_Main_ExecuteRequest<'local>(
    mut env: JNIEnv<'local>,
    _: JClass<'local>,
    dt_index: usize, 
    request:  &mut [u8]
) -> Vec<String> {
    match execute_dt_inner(dt_index, request) {
        Ok(res) => res,
        Err(error) => {
            LAST_ERROR.lock().map(|mut buf| {
                *buf = Some(error);
            }).unwrap();
            vec![]
        }
    }
// }#[no_mangle]
// pub extern "C" fn Java_org_example_Main_getStrings<'local>(
// mut env: JNIEnv<'local>, _: JClass<'local>) -> jobjectArray {
//     let strings = vec![
//         "Hello".to_string(),
//         "from".to_string(),
//         "Rust!".to_string(),
//     ];
// 
//     // Отримуємо клас Java String
//     let string_class = env.find_class("java/lang/String").unwrap();
// 
//     // Створюємо новий масив Java String[]
//     let array = env.new_object_array(strings.len() as i32, string_class, JObject::null()).unwrap();
// 
//     for (i, s) in strings.iter().enumerate() {
//         let jstring = env.new_string(s).unwrap();
//         env.set_object_array_element(array, i as i32, jstring).unwrap();
//     }
// 
//     array.into_inner() // Повертаємо jobjectArray
// }

fn throw_java_exception(mut env: JNIEnv, err_message: &str) {
    env.throw_new("java/lang/RuntimeException", err_message)
        .expect("Error throwing exception!");
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
 