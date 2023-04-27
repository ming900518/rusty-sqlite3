// For better looking JS function.
#![allow(non_snake_case)]
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value};
use sqlx::{SqliteConnection, Connection, query, Row, Column};
use wasm_bindgen::{prelude::*, JsValue};

#[derive(Deserialize)]
struct SqliteConnectionSetting {
    connection_method: SqliteConnectionMethod
}

#[derive(Deserialize)]
enum SqliteConnectionMethod {
    Memory,
    File(SqliteFileConnection)
}

#[derive(Deserialize)]
struct SqliteFileConnection {
    username: Option<String>,
    password: Option<String>,
    filepath: String
}

#[wasm_bindgen]
pub async fn connect(connectInfo: JsValue) -> Result<*mut SqliteConnection, JsError> {
    let Ok(connection_setting) = deserialize_raw_data::<SqliteConnectionSetting>(connectInfo) else {
        return Err(JsError::new("Unable to deserialize connection setting."))
    };

    let connection = if let SqliteConnectionMethod::File(file_connection) = connection_setting.connection_method {
        match (file_connection.username, file_connection.password) {
            (Some(username), Some(password)) => {
                if let Ok(connect) = SqliteConnection::connect(&format!("sqlite://{}:{}@{}", username, password, file_connection.filepath)).await {
                    connect
                } else {
                    return Err(JsError::new("Unable to connect with username/password."))
                }
            }, 
            (None, None) => {
                if let Ok(connect) = SqliteConnection::connect(&format!("sqlite://{}", file_connection.filepath)).await {
                    connect
                } else {
                    return Err(JsError::new("Unable to connect without password."))
                }
            },
            _ => {
                return Err(JsError::new("Not enough parameter provided. Parameters required by authentication: username, password."))
            }
        }
    } else if let Ok(connect) = SqliteConnection::connect("sqlite::memory:").await {
        connect
    } else {
        return Err(JsError::new("Unable to connect in memory database."))
    };
    Ok(Box::into_raw(Box::new(connection))) 
}

#[wasm_bindgen(getter_with_clone)]
pub struct QueryResult {
    pub result: JsValue,
    pub connection: *mut SqliteConnection
}


#[wasm_bindgen]
pub async fn execute(connection: *mut SqliteConnection, sql: &str, parameters: JsValue) -> Result<QueryResult, JsError> {
    let mut db = unsafe { Box::from_raw(connection) };
    let Ok(parameters) = deserialize_raw_data::<Vec<String>>(parameters) else {
        return Err(JsError::new("Unable to deserialize parameters."))
    };
    let mut query = query(sql);

    for i in 0..parameters.len() {
        query = query.bind(parameters.get(i).unwrap());
    }

    match query.fetch_all(&mut *db).await {
        Ok(results) => {
            let result = results.into_iter().enumerate().map(|(index, row)| {
                let column = row.column(index).name().to_owned();
                let field = row.get::<String, usize>(index);
                (column, field)
            }).collect::<Vec<(String, String)>>();
            convert_to_js_value(db, result)
        }, 
        Err(error) => Err(JsError::new(&format!("Unable to fetch data: {}", error)))
    }
}

fn deserialize_raw_data<T: for<'de> Deserialize<'de>>(js_data: JsValue) -> Result<T, JsError> {
    match from_value::<T>(js_data) {
        Ok(raw_result) => Ok(raw_result),
        Err(error) => Err(JsError::new(&format!(
            "Unable to deserialize the input. Detail: {}",
            error
        ))),
    }
}

fn convert_to_js_value<T: Serialize + Debug>(db: Box<SqliteConnection>, data: T) -> Result<QueryResult, JsError> {
    match to_value(&data) {
        Ok(output) => {
            Ok(QueryResult {result: output, connection: Box::into_raw(db) })
        },
        Err(error) => Err(JsError::new(&format!("Error occured when serializing the result: {}.\n\nValue (force displayed with Debug trait): \n{:#?}", error, data)))
    }
}
