use crate::parser::ParsedStruct;

/// 生成创建表的SQL语句
pub fn generate_create_table_sql(parsed: &ParsedStruct) -> String {
    let mut sql = format!("CREATE TABLE IF NOT EXISTS {} (\n", parsed.table_name);
    
    let mut columns = Vec::new();
    for field in &parsed.fields {
        let mut column = format!("    {} {}", field.name, field.sql_type);
        
        if field.is_primary_key {
            column.push_str(" PRIMARY KEY");
        }
        
        if let Some(comment) = &field.comment {
            column.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
        }
        
        columns.push(column);
    }
    
    sql.push_str(&columns.join(",\n"));
    sql.push_str("\n)");
    
    if let Some(comment) = &parsed.comment {
        sql.push_str(&format!(" COMMENT '{}'", comment.replace('\'', "''")));
    }
    
    sql.push(';');
    sql
}

/// 生成插入记录的SQL语句
pub fn generate_insert_sql(parsed: &ParsedStruct) -> String {
    let columns = parsed.fields.iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    
    let placeholders = parsed.fields.iter()
        .enumerate()
        .map(|(i, _)| format!("${}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    
    format!("INSERT INTO {} ({}) VALUES ({});", 
        parsed.table_name, columns, placeholders)
}

/// 生成更新记录的SQL语句
pub fn generate_update_sql(parsed: &ParsedStruct) -> String {
    let primary_key = parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined");
    
    let set_clauses = parsed.fields.iter()
        .filter(|f| !f.is_primary_key)
        .enumerate()
        .map(|(i, f)| format!("{} = ${}", f.name, i + 1))
        .collect::<Vec<_>>()
        .join(", ");
    
    let pk_index = parsed.fields.iter()
        .filter(|f| !f.is_primary_key)
        .count() + 1;
    
    format!("UPDATE {} SET {} WHERE {} = ${};", 
        parsed.table_name, set_clauses, primary_key.name, pk_index)
}

/// 生成删除记录的SQL语句
pub fn generate_delete_sql(parsed: &ParsedStruct) -> String {
    let primary_key = parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined");
    
    format!("DELETE FROM {} WHERE {} = $1;", 
        parsed.table_name, primary_key.name)
}

/// 生成查询记录的SQL语句
pub fn generate_select_sql(parsed: &ParsedStruct) -> String {
    let columns = parsed.fields.iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    
    format!("SELECT {} FROM {};", columns, parsed.table_name)
}

/// 生成按主键查询记录的SQL语句
pub fn generate_select_by_id_sql(parsed: &ParsedStruct) -> String {
    let primary_key = parsed.fields.iter()
        .find(|f| f.is_primary_key)
        .expect("No primary key defined");
    
    let columns = parsed.fields.iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(", ");
    
    format!("SELECT {} FROM {} WHERE {} = $1;", 
        columns, parsed.table_name, primary_key.name)
}