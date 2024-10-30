pub mod dto;
use std::{
    collections::HashMap,
    fs::File,
    io::{ BufWriter, Write },
    process::Stdio,
    time::SystemTime,
};
use chrono::{ DateTime, NaiveDateTime, Utc };
use dto::StringDB;
use tauri::command;
use std::process::Command;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use serde_json::{ json, Value };
use sqlx::{ mysql::MySqlPool, Row };

// 定义一个全局的、可变且线程安全的数据库连接池
pub static POOL: Lazy<Mutex<Option<MySqlPool>>> = Lazy::new(|| Mutex::new(None));

#[command]
async fn connect_db(dbip: &str, username: &str, password: &str) -> Result<String, String> {
    // mysql://sa:Nanhui-380@localhost:3306/rustblog
    let database_url = format!("mysql://{}:{}@{}:3306/mysql", username, password, dbip);
    let pool = MySqlPool::connect(&database_url).await.map_err(|e| e.to_string())?;
    println!("database_url : {} ", database_url.to_string());
    let res = pool.size() > 0;
    let mut conn = POOL.lock().await;
    *conn = Some(pool);
    Ok(res.to_string())
}

#[command]
async fn get_alldbname() -> Result<String, String> {
    let conn = POOL.lock().await;
    match conn.as_ref() {
        Some(pool) => {
            let select_query = "SELECT schema_name AS 'name' FROM information_schema.schemata";
            let data = sqlx
                ::query_as::<_, StringDB>(select_query)
                .fetch_all(pool).await
                .map_err(|e| e.to_string())?;

            let res = serde_json::to_string(&data).map_err(|e| e.to_string())?;
            Ok(res)
        }
        None => Err("数据库连接未初始化".to_string()),
    }
}
#[command]
async fn get_alltablenamebydbname(dbname: &str) -> Result<String, String> {
    let conn = POOL.lock().await;
    match conn.as_ref() {
        Some(pool) => {
            let select_query =
                format!("SELECT table_name AS `name` FROM information_schema.tables WHERE table_schema =  '{}'", dbname);
            let data = sqlx
                ::query_as::<_, StringDB>(&select_query)
                .fetch_all(pool).await
                .map_err(|e| e.to_string())?;

            let res = serde_json::to_string(&data).map_err(|e| e.to_string())?;
            Ok(res)
        }
        None => Err("数据库连接未初始化".to_string()),
    }
}

#[command]
async fn get_table_columns(db_name: &str, table_name: &str) -> Result<String, String> {
    let conn = POOL.lock().await;
    match conn.as_ref() {
        Some(pool) => {
            let columns_query = format!(
                "SELECT COLUMN_NAME AS name, DATA_TYPE AS type FROM information_schema.columns WHERE table_schema = '{}' AND table_name = '{}'",
                db_name,
                table_name
            );
            let rows = sqlx
                ::query(&columns_query)
                .fetch_all(pool).await
                .map_err(|e| e.to_string())?;

            let mut columns = HashMap::new();
            for row in rows {
                let column_name: String = row.get::<String, &str>("name");
                let data_type: String = row.get::<String, &str>("type");
                columns.insert(column_name, data_type);
            }
            let res = json!({ "columns": columns }).to_string();
            Ok(res)
        }
        None => Err("数据库未初始化".to_string()),
    }
}

#[command]
async fn execute_sql_command(sql: &str) -> Result<String, String> {
    let conn = POOL.lock().await;
    match conn.as_ref() {
        Some(pool) => {
            // 执行 SQL 命令（注意：这里存在 SQL 注入的风险！）
            println!("execute_sql_command sql: {:?}", sql);
            let rows_affected = sqlx
                ::query(&sql)
                .execute(pool).await
                .map_err(|e| e.to_string())?;
            let num = rows_affected.rows_affected();
            // 根据影响的行数构建返回结果
            let res = json!({ "num": num }).to_string();
            Ok(res)
        }
        None => Err("数据库未初始化".to_string()),
    }
}

#[command]
async fn execute_dbtable_command(sql: &str) -> Result<String, String> {
    let conn = POOL.lock().await;
    match conn.as_ref() {
        Some(pool) => {
            println!("execute_sql_command sql: {:?}", sql);
            sqlx
                ::query(&sql)
                .execute(pool).await
                .map_err(|e| e.to_string())?;
            Ok("ok".to_string())
        }
        None => Err("数据库未初始化".to_string()),
    }
}

#[command]
async fn query_table_data(db_name: &str, table_name: &str, sql: &str) -> Result<String, String> {
    println!("query start");
    let start = SystemTime::now();

    let mut jsonres: Value = json!({});
    let mut jsoncolumns: Value = json!([]);
    let mut jsondata: Value = json!([]);
    let conn = POOL.lock().await;
    match conn.as_ref() {
        Some(pool) => {
            let columns_query = format!(
                "SELECT COLUMN_NAME AS name, DATA_TYPE AS type FROM information_schema.columns WHERE table_schema = '{}' AND table_name = '{}'",
                db_name,
                table_name
            );
            let rows = sqlx
                ::query(&columns_query)
                .fetch_all(pool).await
                .map_err(|e| e.to_string())?;

            let mut index = 0;
            for row in rows {
                let column_name: String = row.get::<String, &str>("name");
                let data_type: String = row.get::<String, &str>("type");
                let mut json_obj: Value = json!({});
                json_obj["index"] = json!(index);
                json_obj["name"] = json!(column_name);
                json_obj["type"] = json!(data_type);
                if let Some(array) = jsoncolumns.as_array_mut() {
                    array.push(json_obj);
                }
                index += 1;
            }
            jsonres["columns"] = jsoncolumns;
        }
        None => {
            return Err("数据库连接未初始化".to_string());
        }
    }

    match conn.as_ref() {
        Some(pool) => {
            let mut parts = sql.splitn(2, "FROM");
            let from_part = parts.nth(1).unwrap_or("");
            let mut new_sql = "SELECT COUNT(*) FROM ".to_string() + from_part;
            if let Some(limit_pos) = new_sql.find("LIMIT") {
                new_sql.truncate(limit_pos);
            }
            println!("query count: {:?}", new_sql);
            let count_result: i64 = sqlx
                ::query(&new_sql)
                .fetch_one(pool).await
                .map_err(|e| e.to_string())?
                .get(0);

            jsonres["total"] = json!(count_result);
        }
        None => {
            return Err("数据库未初始化".to_string());
        }
    }

    match conn.as_ref() {
        Some(pool) => {
            let select_query = format!("{}", sql);
            println!("query sql: {:?}", sql);
            let rows = sqlx
                ::query(&select_query)
                .fetch_all(pool).await
                .map_err(|err| err.to_string())?;
            if rows.is_empty() {
                jsonres["data"] = jsondata;
                let res = jsonres.to_string();
                return Ok(res);
            }

            for (i, row) in rows.iter().enumerate() {
                let mut json_obj: Value = json!({});

                if let Some(columns) = jsonres.get("columns") {
                    let mut k = 0;
                    for column in columns.as_array().unwrap() {
                        let c = column["type"].as_str().unwrap();
                        let n = column["name"].as_str().unwrap();
                        if
                            c.eq_ignore_ascii_case("int") ||
                            c.eq_ignore_ascii_case("integer") ||
                            c.eq_ignore_ascii_case("long") ||
                            c.eq_ignore_ascii_case("bigint") ||
                            c.eq_ignore_ascii_case("smallint")
                        {
                            match row.try_get::<i64, _>(k) {
                                Ok(value) => {
                                    json_obj[n] = json!(value.to_string());
                                }
                                Err(..) => {
                                    json_obj[n] = serde_json::Value::Null;
                                }
                            }
                        } else if
                            c.eq_ignore_ascii_case("varchar") ||
                            c.eq_ignore_ascii_case("varstring") ||
                            c.eq_ignore_ascii_case("char") ||
                            c.eq_ignore_ascii_case("text")
                        {
                            match row.try_get::<String, usize>(k) {
                                Ok(value) => {
                                    json_obj[n] = json!(&value.to_string());
                                }
                                Err(..) => {
                                    //试试VARBINARY类型转换一次,数据库编码方式如果是bin
                                    match row.try_get::<Vec<u8>, usize>(k) {
                                        Ok(value) => {
                                            // 这里你可以将 Vec<u8> 转换为 String，但需要注意编码问题
                                            // 例如，如果数据是 UTF-8 编码的，你可以使用 String::from_utf8_lossy
                                            let str_value = String::from_utf8_lossy(&value);
                                            json_obj[n] = json!(str_value);
                                        }
                                        Err(..) => {
                                            json_obj[n] = serde_json::Value::Null;
                                        }
                                    }
                                }
                            }
                        } else if
                            c.eq_ignore_ascii_case("datetime") ||
                            c.eq_ignore_ascii_case("timestamp")
                        {
                            // 通过索引获取 DateTime<Utc>
                            let datetime_utc_by_index: Option<DateTime<Utc>> = match row.try_get(k) {
                                Ok(dt) => Some(dt),
                                Err(..) => { None }
                            };

                            // 根据获取的结果进行处理
                            if let Some(datetime_utc) = datetime_utc_by_index {
                                // 将 DateTime<Utc> 转换为 NaiveDateTime
                                let datetime: NaiveDateTime = datetime_utc.naive_local();
                                json_obj[n] = json!(&datetime.to_string());
                            } else {
                                json_obj[n] = serde_json::Value::Null;
                            }
                        } else if
                            c.eq_ignore_ascii_case("double") ||
                            c.eq_ignore_ascii_case("float")
                        {
                            match row.try_get::<f64, _>(k) {
                                Ok(value) => {
                                    json_obj[n] = json!(value);
                                }
                                Err(..) => {
                                    json_obj[n] = serde_json::Value::Null;
                                }
                            }
                        } else if
                            c.eq_ignore_ascii_case("varbinary") ||
                            c.eq_ignore_ascii_case("blob")
                        {
                            println!("c.eq_ignore_ascii_case(varbinary,blob)");
                            // let value: Vec<u8> = row.try_get::<Vec<u8>, usize>(
                            //     column["index"].as_usize().unwrap()
                            // )?;
                            // println!(
                            //     "row.index:{},{} - decode binary data of length {}",
                            //     i,
                            //     column["type"],
                            //     value.len()
                            // );
                        } else {
                            println!("row.index:{},{}", i, column["type"]);
                        }
                        k += 1;
                    }
                }
                if let Some(array) = jsondata.as_array_mut() {
                    array.push(json_obj);
                }
            }
            jsonres["data"] = jsondata;
            println!("query end");
            let elapsed = start.elapsed().expect("Time went backwards");
            println!("query time: {:?}", elapsed);
            let res = jsonres.to_string();
            Ok(res)
        }
        None => Err("数据库未初始化".to_string()),
    }
}

#[command]
async fn backup_db_command(
    username: &str,
    password: &str,
    dbname: &str,
    filename: &str
) -> Result<String, String> {
    let command = format!("mysqldump -u{} -p{} {}", username, password, dbname);
    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let output = Command::new(shell)
        .arg(if shell == "cmd" { "/c" } else { "-c" })
        .arg(&command)
        .stdout(Stdio::piped())
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        let file = File::create(filename).map_err(|e| e.to_string())?;
        let mut file = BufWriter::new(file);
        file.write_all(&output.stdout).map_err(|e| e.to_string())?;
        Ok("Backup successful".to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn run_command(command: &str) -> Result<String, String> {
    let shell = if cfg!(target_os = "windows") { "cmd" } else { "sh" };
    let output = Command::new(shell)
        .arg(if shell == "cmd" { "/c" } else { "-c" })
        .arg(command)
        .output()
        .map_err(|e| e.to_string())?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}
#[command]
async fn revert_db_command(
    username: &str,
    password: &str,
    dbname: &str,
    filename: &str
) -> Result<String, String> {
    match
        run_command(
            format!("mysql -u {} -p{} {} < {}", username, password, dbname, filename).as_str()
        )
    {
        Ok(_) => println!("Revert DB"),
        Err(e) => {
            return Err(format!("Failed to Revert DB: {}", e));
        }
    }

    Ok("ok".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder
        ::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(
            tauri::generate_handler![
                connect_db,
                get_alldbname,
                get_alltablenamebydbname,
                get_table_columns,
                execute_sql_command,
                query_table_data,
                backup_db_command,
                revert_db_command,
                execute_dbtable_command
            ]
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
