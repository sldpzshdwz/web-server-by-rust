use std::error::Error;

pub mod diary_work;
pub mod ai;
pub mod memory;
pub mod long_work;
pub type 结果<T>=std::result::Result<T, Box<dyn Error>>;