use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use urlencoding::encode;

#[derive(Deserialize, Debug)]
struct SqlResponse {
    successStatus: u32,
    response: Option<SqlResult>,
    error: Option<String>,
}
#[derive(Deserialize, Debug)]
struct SqlResult {
    fieldsMetadata: Option<Vec<FieldMetadata>>,
    items: Option<Vec<Vec<serde_json::Value>>>,
}
#[derive(Deserialize, Debug)]
struct FieldMetadata {
    fieldName: String,
    fieldTypeName: String,
}

async fn execute_sql(cache: &str, query: &str) -> Result<(), Box<dyn Error>> {
    let client = Client::new();
    let encoded_query = encode(query);
    let cmd = "qryfldexe";
    let url = format!(
        "http://localhost:8080/ignite?cmd={}&cacheName={}&qry={}&pageSize=10",
        cmd, cache, encoded_query
    );

    let res = client.get(&url).send().await?;
    let body = res.text().await?;
    
    let parsed: SqlResponse = serde_json::from_str(&body)?;
    
    if let Some(error) = parsed.error {
        eprintln!("Error: {}", error);
    } else if let Some(response) = parsed.response {
        println!("\n\nQuery executed successfully.");
        
        if let Some(fields) = response.fieldsMetadata {
            println!("Fields: {:?}", fields);
        }
        
        if let Some(items) = response.items {
            for row in items {
                println!("Row: {:?}", row);
            }
        }
    }
    
    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Create table
    execute_sql(
        "PersonCache",
        "CREATE TABLE Person (id INT PRIMARY KEY, name VARCHAR(50), age INT)",
    )
    .await?;

    // INSERT
    execute_sql(
        "PersonCache",
        "INSERT INTO Person (id, name, age) VALUES (1, 'John Doe', 30), (2, 'Will Smith', 10)",
    )
    .await?;

    // SELECT all
    execute_sql("PersonCache", "SELECT * FROM Person").await?;

    // SELECT with WHERE
    execute_sql("PersonCache", "SELECT * FROM Person WHERE age > 25").await?;

    // UPDATE
    execute_sql("PersonCache", "UPDATE Person SET age = 31 WHERE id = 2").await?;
    execute_sql("PersonCache", "SELECT * FROM Person").await?;

    // DELETE
    execute_sql("PersonCache", "DELETE FROM Person WHERE age = 30").await?;
    execute_sql("PersonCache", "SELECT * FROM Person").await?;

    Ok(())
}
