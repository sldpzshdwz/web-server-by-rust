use std::sync::{Mutex, OnceLock};

pub struct 用户数据{
    
}
pub struct 数据库连接池{

}
impl 数据库连接池{
    pub fn get_instance()->&'static Mutex<mysql::Pool>{
        static INSTANCE:OnceLock<Mutex<mysql::Pool>>=OnceLock::new();
        INSTANCE.get_or_init(|| Mutex::new(mysql::Pool::new(crate::数据库连接url).unwrap()))
    }
}