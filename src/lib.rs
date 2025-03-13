use std::sync::{LazyLock, Mutex};

use thread_pool::线程池;
use log::日志级别;
pub mod network;
pub mod thread_pool;
pub mod api;
pub mod database;
pub mod log;

static 线程池线程数:u32=5;
static 数据库连接url: &str = "mysql://root:aA147741@localhost:3306/rust91";
static 日志显示级别:日志级别=日志级别::INFO;