use syn::{Attribute, Lit, Meta, Type};

/// 提取字段注释，支持从文档注释（///）和 #[comment = "..."] 属性中提取
pub fn extract_comment(attrs: &[Attribute]) -> Option<String> {
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

/// 将Rust类型映射到SQL类型
pub fn map_type_to_sql(ty: &Type) -> String {
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

/// 获取表名，支持 #[table_name = "..."] 属性
pub fn get_table_name(attrs: &[Attribute], default: &str) -> String {
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