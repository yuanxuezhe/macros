use syn::{Attribute, Data, DeriveInput, Field, Fields, Lit, Meta, Type};
use crate::utils::{extract_comment, get_table_name};

/// 表示一个字段的解析结果
pub struct ParsedField {
    pub name: String,
    pub ty: Type,
    pub sql_type: String,
    pub is_primary_key: bool,
    pub comment: Option<String>,
}

/// 表示一个结构体的解析结果
pub struct ParsedStruct {
    pub name: String,
    pub table_name: String,
    pub fields: Vec<ParsedField>,
    pub comment: Option<String>,
}

/// 解析结构体字段
pub fn parse_field(field: &Field) -> ParsedField {
    let name = field.ident.as_ref().unwrap().to_string();
    let ty = field.ty.clone();
    
    // 检查是否有自定义SQL类型
    let mut sql_type = None;
    for attr in &field.attrs {
        if attr.path.is_ident("sql_type") {
            if let Ok(Meta::NameValue(meta)) = attr.parse_meta() {
                if let Lit::Str(lit_str) = meta.lit {
                    sql_type = Some(lit_str.value());
                }
            }
        }
    }
    
    // 检查是否是主键
    let is_primary_key = field.attrs.iter().any(|attr| attr.path.is_ident("primary_key"));
    
    // 提取注释
    let comment = extract_comment(&field.attrs);
    
    ParsedField {
        name,
        ty: ty.clone(),
        sql_type: sql_type.unwrap_or_else(|| crate::utils::map_type_to_sql(&ty)),
        is_primary_key,
        comment,
    }
}

/// 解析结构体定义
pub fn parse_struct(input: &DeriveInput) -> ParsedStruct {
    let name = input.ident.to_string();
    let table_name = get_table_name(&input.attrs, &name);
    let comment = extract_comment(&input.attrs);
    
    let fields = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    fields.named.iter().map(parse_field).collect()
                },
                _ => panic!("Only structs with named fields are supported"),
            }
        },
        _ => panic!("Only structs are supported"),
    };
    
    ParsedStruct {
        name,
        table_name,
        fields,
        comment,
    }
}