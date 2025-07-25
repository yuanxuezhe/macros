//! # SQL CRUD 宏
//! 
//! 这个库提供了一个派生宏 `SqlCRUD`，用于为Rust结构体自动生成SQL CRUD操作。
//! 
//! ## 示例
//! 
//! ```rust
//! use macros::SqlCRUD;
//! 
//! #[derive(SqlCRUD)]
//! struct User {
//!     #[primary_key]
//!     id: i32,
//!     /// 用户名
//!     name: String,
//!     #[comment = "用户邮箱"]
//!     email: String,
//! }
//! ```

extern crate proc_macro;

mod parser;
mod sql_generator;
mod code_generator;
mod utils;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

/// 为结构体自动生成SQL CRUD操作的派生宏
///
/// # 属性
///
/// - `#[primary_key]`: 标记主键字段
/// - `#[comment = "..."]`: 为字段或表添加注释
/// - `#[table_name = "..."]`: 自定义表名
/// - `#[sql_type = "..."]`: 自定义SQL类型
///
/// # 生成的方法
///
/// - `init_table`: 初始化表结构
/// - `table_name`: 获取表名
/// - `insert`: 插入记录
/// - `insert_one`: 插入记录（静态方法）
/// - `update`: 更新记录
/// - `delete`: 删除记录
/// - `delete_by_id`: 按ID删除记录（静态方法）
/// - `find_all`: 查询所有记录（静态方法）
/// - `find_by_id`: 按ID查询记录（静态方法）
#[proc_macro_derive(SqlCRUD, attributes(primary_key, comment, table_name, sql_type))]
pub fn derive_sql_crud(input: TokenStream) -> TokenStream {
    // 解析输入的Rust代码
    let input = parse_macro_input!(input as DeriveInput);
    
    // 解析结构体定义
    let parsed = parser::parse_struct(&input);
    
    // 生成实现代码
    let output = code_generator::generate_impl_block(&parsed);
    
    // 转换为TokenStream并返回
    output.into()
}