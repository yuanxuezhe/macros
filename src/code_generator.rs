use proc_macro2::{TokenStream, Span};
use quote::{quote, format_ident};
use syn::Ident;
use crate::parser::ParsedStruct;
use crate::sql_generator::{
    generate_create_table_sql,
    generate_insert_sql,
    generate_update_sql,
    generate_delete_sql,
    generate_select_sql,
    generate_select_by_id_sql
};

/// 生成表初始化方法
pub fn generate_init_table_method(parsed: &ParsedStruct) -> TokenStream {
    let create_table_sql = generate_create_table_sql(parsed);
    let table_name = &parsed.table_name;
    
    quote! {
        /// 初始化表结构
        pub async fn init_table(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
            let sql = #create_table_sql;
            sqlx::query(sql).execute(pool).await?;
            Ok(())
        }

        /// 获取表名
        pub fn table_name() -> &'static str {
            #table_name
        }
    }
}

/// 生成插入记录方法
pub fn generate_insert_method(parsed: &ParsedStruct) -> TokenStream {
    let insert_sql = generate_insert_sql(parsed);
    let struct_name = format_ident!("{}", parsed.name);
    
    let field_names: Vec<Ident> = parsed.fields.iter()
        .map(|f| format_ident!("{}", f.name))
        .collect();
    
    quote! {
        /// 插入记录
        pub async fn insert(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
            let sql = #insert_sql;
            sqlx::query(sql)
                #(.bind(&self.#field_names))*
                .execute(pool)
                .await?;
            Ok(())
        }

        /// 插入记录并返回插入的对象
        pub async fn insert_one(pool: &sqlx::Pool<sqlx::Sqlite>, item: &#struct_name) -> Result<(), sqlx::Error> {
            item.insert(pool).await
        }
    }
}

/// 生成更新记录方法
pub fn generate_update_method(parsed: &ParsedStruct) -> TokenStream {
    let update_sql = generate_update_sql(parsed);
    
    let non_pk_fields: Vec<Ident> = parsed.fields.iter()
        .filter(|f| !f.is_primary_key)
        .map(|f| format_ident!("{}", f.name))
        .collect();
    
    let pk_field = format_ident!("{}", parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined")
        .name);
    
    quote! {
        /// 更新记录
        pub async fn update(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
            let sql = #update_sql;
            sqlx::query(sql)
                #(.bind(&self.#non_pk_fields))*
                .bind(&self.#pk_field)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}

/// 生成删除记录方法
pub fn generate_delete_method(parsed: &ParsedStruct) -> TokenStream {
    let delete_sql = generate_delete_sql(parsed);
    let struct_name = format_ident!("{}", parsed.name);
    
    let pk_field = format_ident!("{}", parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined")
        .name);
    
    let pk_type = &parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined")
        .ty;
    
    quote! {
        /// 删除记录
        pub async fn delete(&self, pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<(), sqlx::Error> {
            let sql = #delete_sql;
            sqlx::query(sql)
                .bind(&self.#pk_field)
                .execute(pool)
                .await?;
            Ok(())
        }

        /// 按ID删除记录
        pub async fn delete_by_id(pool: &sqlx::Pool<sqlx::Sqlite>, id: &#pk_type) -> Result<(), sqlx::Error> {
            let sql = #delete_sql;
            sqlx::query(sql)
                .bind(id)
                .execute(pool)
                .await?;
            Ok(())
        }
    }
}

/// 生成查询记录方法
pub fn generate_select_methods(parsed: &ParsedStruct) -> TokenStream {
    let select_sql = generate_select_sql(parsed);
    let select_by_id_sql = generate_select_by_id_sql(parsed);
    let struct_name = format_ident!("{}", parsed.name);
    
    let pk_field = format_ident!("{}", parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined")
        .name);
    
    let pk_type = &parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined")
        .ty;
    
    quote! {
        /// 查询所有记录
        pub async fn find_all(pool: &sqlx::Pool<sqlx::Sqlite>) -> Result<Vec<#struct_name>, sqlx::Error> {
            let sql = #select_sql;
            let records = sqlx::query_as::<_, #struct_name>(sql)
                .fetch_all(pool)
                .await?;
            Ok(records)
        }

        /// 按ID查询记录
        pub async fn find_by_id(pool: &sqlx::Pool<sqlx::Sqlite>, id: &#pk_type) -> Result<Option<#struct_name>, sqlx::Error> {
            let sql = #select_by_id_sql;
            let record = sqlx::query_as::<_, #struct_name>(sql)
                .bind(id)
                .fetch_optional(pool)
                .await?;
            Ok(record)
        }
    }
}

/// 生成所有CRUD方法
pub fn generate_impl_block(parsed: &ParsedStruct) -> TokenStream {
    let struct_name = format_ident!("{}", parsed.name);
    
    let init_table_method = generate_init_table_method(parsed);
    let insert_method = generate_insert_method(parsed);
    let update_method = generate_update_method(parsed);
    let delete_method = generate_delete_method(parsed);
    let select_methods = generate_select_methods(parsed);
    
    quote! {
        impl #struct_name {
            #init_table_method
            #insert_method
            #update_method
            #delete_method
            #select_methods
        }
    }
}