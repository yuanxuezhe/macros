extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, Lit, Meta, Type,
};

// 提取字段注释
fn extract_comment(attrs: &[Attribute]) -> Option<String> {
    let mut comments = Vec::new();
    let mut explicit_comment = None;

    for attr in attrs {
        // 处理 #[comment = "..."] 属性
        if attr.path.is_ident("comment") {
            if let Ok(Meta::NameValue(meta)) = attr.parse_meta() {
                if let Lit::Str(lit_str) = meta.lit {
                    explicit_comment = Some(lit_str.value());
                }
            }
            continue;
        }

        // 处理文档注释 ///
        if attr.path.is_ident("doc") {
            if let Ok(Meta::NameValue(meta)) = attr.parse_meta() {
                if let Lit::Str(lit_str) = meta.lit {
                    let comment = lit_str.value().trim().to_string();
                    comments.push(comment);
                }
            }
        }
    }

    explicit_comment.or_else(|| {
        if comments.is_empty() {
            None
        } else {
            Some(comments.join(" "))
        }
    })
}

// 类型映射到SQL类型
fn map_type_to_sql(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            let ident = type_path.path.segments.last().unwrap().ident.to_string();
            match ident.as_str() {
                "i32" => "INT".into(),
                "i64" => "BIGINT".into(),
                "String" => "VARCHAR(255)".into(),
                "bool" => "BOOLEAN".into(),
                "f32" => "FLOAT".into(),
                "f64" => "DOUBLE".into(),
                "NaiveDateTime" => "DATETIME".into(),
                "Uuid" => "UUID".into(),
                _ => ident,
            }
        }
        _ => "TEXT".into(),
    }
}

// 获取表名（支持 #[table_name] 属性）
fn get_table_name(attrs: &[Attribute], default: &str) -> String {
    for attr in attrs {
        if attr.path.is_ident("table_name") {
            if let Ok(Meta::NameValue(meta)) = attr.parse_meta() {
                if let Lit::Str(lit_str) = meta.lit {
                    return lit_str.value();
                }
            }
        }
    }
    default.to_lowercase()
}

#[proc_macro_derive(
    SqlCRUD,
    attributes(primary_key, comment, table_name, sql_type)
)]
pub fn sql_crud_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // 处理表级属性
    let table_comment = extract_comment(&input.attrs);
    let table_name = get_table_name(&input.attrs, &struct_name.to_string());

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Only structs with named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    // 处理字段级属性
    let mut columns = Vec::new();
    let mut primary_keys = Vec::new();

    for field in &fields {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let sql_type = field
            .attrs
            .iter()
            .find(|a| a.path.is_ident("sql_type"))
            .and_then(|a| {
                if let Ok(Meta::NameValue(meta)) = a.parse_meta() {
                    if let Lit::Str(lit_str) = meta.lit {
                        return Some(lit_str.value());
                    }
                }
                None
            })
            .unwrap_or_else(|| map_type_to_sql(&field.ty));

        let comment = extract_comment(&field.attrs)
            .map(|c| format!(" COMMENT '{}'", c.replace('\'', "''")))
            .unwrap_or_default();

        // 构建列定义
        let column_def = format!("{field_name} {sql_type}{comment}");
        columns.push(column_def);

        // 记录主键
        if field.attrs.iter().any(|a| a.path.is_ident("primary_key")) {
            primary_keys.push(field_name);
        }
    }

    // 构建建表语句
    let primary_key_def = if !primary_keys.is_empty() {
        format!(
            ",\n    PRIMARY KEY ({})",
            primary_keys.join(", ")
        )
    } else {
        String::new()
    };

    let table_comment_def = table_comment
        .map(|c| format!(" COMMENT '{}'", c.replace('\'', "''")))
        .unwrap_or_default();

    let create_table_sql = format!(
        "CREATE TABLE {table_name} (\n    {columns}{primary_key_def}\n){table_comment_def};",
        columns = columns.join(",\n    ")
    );

    // 生成表存在性检查SQL
    let check_table_sql = format!(
        "SELECT name FROM sqlite_master 
         WHERE type='table' AND name='{}'",
        table_name
    );

    // 生成CRUD SQL
    let insert_fields = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    let insert_values = fields.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let insert_sql = format!("INSERT INTO {table_name} ({insert_fields}) VALUES ({insert_values})");

    let select_sql = format!("SELECT {insert_fields} FROM {table_name}");

    let update_fields = fields
        .iter()
        .filter(|f| !primary_keys.contains(&&f.ident.as_ref().unwrap().to_string()))
        .map(|f| format!("{} = ?", f.ident.as_ref().unwrap()))
        .collect::<Vec<_>>()
        .join(", ");
    let where_clause = primary_keys
        .iter()
        .map(|pk| format!("{pk} = ?"))
        .collect::<Vec<_>>()
        .join(" AND ");
    let update_sql = format!("UPDATE {table_name} SET {update_fields} WHERE {where_clause}");

    let delete_sql = format!("DELETE FROM {table_name} WHERE {where_clause}");

    let select_key_sql = format!("SELECT {insert_fields} FROM {table_name} WHERE {where_clause}");

    // 分离主键和非主键字段
    let (primary_keys, non_primary): (Vec<_>, Vec<_>) = fields
        .iter()
        .partition(|f| f.attrs.iter().any(|a| a.path.is_ident("primary_key")));

    let key_params: Vec<_> = primary_keys.iter().map(|f: &&syn::Field| {
        let ident = &f.ident;
        quote! { .set_param(self.#ident.clone()) }
    }).collect();

    let nokey_params: Vec<_> = non_primary.iter().map(|f| {
        let ident = &f.ident;
        quote! { .set_param(self.#ident.clone()) }
    }).collect();

    // 生成最终代码
    let expanded = quote! {
        impl #struct_name {
            pub fn check_table_sql() -> String {
                #check_table_sql.into()
            }

            pub fn create_table_sql() -> String {
                #create_table_sql.into()
            }

            pub fn insert_sql() -> String {
                #insert_sql.into()
            }

            pub fn select_all_sql() -> String {
                #select_sql.into()
            }
            
            pub fn select_key_sql() -> String {
                #select_key_sql.into()
            }

            pub fn update_sql() -> String {
                #update_sql.into()
            }

            pub fn delete_sql() -> String {
                #delete_sql.into()
            }

            pub async fn init(db: &Database) -> Result<(), sqlx::Error> {
                // 1. 检查表是否存在
                let result = db.open(&Self::check_table_sql())
                    .query(db)
                    .await?;
    
                // 2. 不存在则创建
                if result.rows() == 0 {
                    db.open(&Self::create_table_sql())
                        .exec(db)
                        .await?;
                }
    
                Ok(())
            }

            pub async fn insert(&self, db: &Database) -> Result<u64, sqlx::Error> {
                db.open(&Self::insert_sql())
                #(#key_params)*
                #(#nokey_params)*
                .exec(db)
                .await
            }

            pub async fn update(&self, db: &Database) -> Result<u64, sqlx::Error> {
                db.open(&Self::update_sql())
                #(#nokey_params)*
                #(#key_params)*
                .exec(db)
                .await
            }

            pub async fn delete(&self, db: &Database) -> Result<u64, sqlx::Error> {
                db.open(&Self::delete_sql())
                #(#key_params)*
                .exec(db)
                .await
            }

            pub async fn query(&self, db: &Database) -> Result<ResultSet, sqlx::Error> {
                db.open(&Self::select_key_sql())
                #(#key_params)*
                .query(db)
                .await
            }
        }
    };

    TokenStream::from(expanded)
}