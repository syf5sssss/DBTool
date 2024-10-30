use sqlx::FromRow;
// 定义一个结构体来映射用户表的行
#[derive(FromRow, serde::Serialize, serde::Deserialize)]
pub struct User {
    pub id: u32,
    pub username: String,
    pub password: String,
    pub nickname: String,
}
impl User {
    pub fn new(id: u32, username: String, password: String, nickname: String) -> Self {
        User { id, username, password, nickname }
    }
}

#[derive(FromRow, serde::Serialize, serde::Deserialize)]
pub struct StringDB {
    pub name: String,
}
impl StringDB {
    pub fn new(name: String) -> Self {
        StringDB { name }
    }
}

#[derive(FromRow, serde::Serialize, serde::Deserialize)]
pub struct ColumnInfo {
    pub column_name: String,
    pub data_type: String,
}
impl ColumnInfo {
    pub fn new(column_name: String, data_type: String) -> Self {
        ColumnInfo { column_name, data_type }
    }
}
