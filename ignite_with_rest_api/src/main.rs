use reqwest::Client;
use serde::{Serialize, Deserialize};
use serde_json::json; // Macro for easily creating JSON values

// Define a custom struct for our cache value
// Must derive Serialize and Deserialize for JSON handling with serde
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct MyValue {
    id: i32,
    name: String,
}

#[tokio::main] // Marks the async main function
async fn main() -> Result<(), Box<dyn std::error::Error>> { // Return type for error handling
    let client = Client::new(); // Create a new HTTP client instance
    let base_url = "http://127.00.1:8080/ignite"; // Default Ignite REST API endpoint
    let cache_name = "my_rest_cache"; // Name of the cache we'll interact with (also used for SQL table)

    println!("--- Apache Ignite REST API Client Example ---");

    // --- 1. Create a Cache (or get it if it already exists) ---
    // Endpoint: /ignite/cache/getOrCreate
    let create_cache_url = format!("{}/cache/getOrCreate", base_url);
    println!("\n1. Creating/getting cache '{}'...", cache_name);
    let create_payload = json!({
        "cacheName": cache_name
    });

    let response = client.post(&create_cache_url)
        .json(&create_payload) // Send JSON payload
        .send() // Send the request
        .await?; // Wait for the response

    let status = response.status(); // Store the status code HERE

    if status.is_success() { // Check if HTTP status is 2xx (success)
        println!("   Cache '{}' ensured (created or already exists).", cache_name);
    } else {
        // Now get the body AFTER getting the status
        eprintln!("   Failed to ensure cache: Status: {}, Body: {:?}", status, response.text().await?);
        return Err("Failed to ensure cache".into());
    }

    // --- 2. Put data into the cache (CREATE/UPDATE) ---
    // Endpoint: /ignite/cache/put
    let put_url = format!("{}/cache/put", base_url);

    let key1 = 1;
    let value1 = MyValue { id: 1, name: "Data from Rust REST client!".to_string() };
    let put_payload1 = json!({
        "cacheName": cache_name,
        "key": key1,
        "value": value1 // Serde will automatically convert MyValue to JSON
    });
    println!("\n2. Putting key:{} -> value:{:?} into cache...", key1, value1);
    let response = client.post(&put_url)
        .json(&put_payload1)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        println!("   Data for key {} put successfully.", key1);
    } else {
        eprintln!("   Failed to put data for key {}: Status: {}, Body: {:?}", key1, status, response.text().await?);
        return Err(format!("Failed to put data for key {}", key1).into());
    }

    let key2 = 2;
    let value2 = MyValue { id: 2, name: "Another entry!".to_string() };
    let put_payload2 = json!({
        "cacheName": cache_name,
        "key": key2,
        "value": value2
    });
    println!("   Putting key:{} -> value:{:?} into cache...", key2, value2);
    let response = client.post(&put_url)
        .json(&put_payload2)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        println!("   Data for key {} put successfully.", key2);
    } else {
        eprintln!("   Failed to put data for key {}: Status: {}, Body: {:?}", key2, status, response.text().await?);
        return Err(format!("Failed to put data for key {}", key2).into());
    }


    // --- 3. Get data from the cache (READ) ---
    // Endpoint: /ignite/cache/get
    let get_url = format!("{}/cache/get", base_url);

    // Get key1
    let get_payload1 = json!({
        "cacheName": cache_name,
        "key": key1
    });
    println!("\n3. Getting value for key:{}...", key1);
    let response = client.post(&get_url)
        .json(&get_payload1)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        let response_body = response.json::<serde_json::Value>().await?; // This consumes `response`
        if let Some(value_obj) = response_body.get("response") {
            if value_obj.is_null() {
                println!("   No value found for key {}.", key1);
            } else {
                let retrieved_value1: MyValue = serde_json::from_value(value_obj.clone())?;
                println!("   Retrieved value for key {}: {:?}", key1, retrieved_value1);
            }
        } else {
            eprintln!("   'response' field missing in GET response for key {}: {:?}", key1, response_body);
        }
    } else {
        eprintln!("   Failed to get data for key {}: Status: {}, Body: {:?}", key1, status, response.text().await?);
        return Err(format!("Failed to get data for key {}", key1).into());
    }

    // Get a non-existent key
    let non_existent_key = 99;
    let get_payload_non_existent = json!({
        "cacheName": cache_name,
        "key": non_existent_key
    });
    println!("   Attempting to get non-existent key:{}...", non_existent_key);
    let response = client.post(&get_url)
        .json(&get_payload_non_existent)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        let response_body = response.json::<serde_json::Value>().await?; // This consumes `response`
        if let Some(value_obj) = response_body.get("response") {
            if value_obj.is_null() {
                println!("   Correctly found no value for non-existent key {}.", non_existent_key);
            } else {
                eprintln!("   Unexpectedly found value for non-existent key {}: {:?}", non_existent_key, value_obj);
            }
        } else {
            eprintln!("   'response' field missing for non-existent key {}: {:?}", non_existent_key, response_body);
        }
    } else {
        eprintln!("   Failed to get data for non-existent key {}: Status: {}, Body: {:?}", non_existent_key, status, response.text().await?);
        return Err(format!("Failed to get data for non-existent key {}", non_existent_key).into());
    }

    // --- 4. Execute SQL Query (CREATE TABLE) ---
    // Endpoint: /ignite/sql
    let sql_url = format!("{}/sql", base_url);
    println!("\n4. Executing SQL: CREATE TABLE '{}'...", cache_name);
    let create_table_sql = format!(
        "CREATE TABLE {} (id INT PRIMARY KEY, name VARCHAR) WITH \"template=REPLICATED\"",
        cache_name
    );
    let create_table_payload = json!({
        "schemaName": "PUBLIC", // Default schema in Ignite SQL
        "query": create_table_sql,
        "pageSize": 100 // Required for SQL operations, even DDL
    });

    let response = client.post(&sql_url)
        .json(&create_table_payload)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        let response_body = response.json::<serde_json::Value>().await?; // This consumes `response`
        if response_body["successStatus"].as_i64() == Some(0) {
            println!("   SQL: CREATE TABLE {} executed successfully.", cache_name);
        } else {
            eprintln!("   SQL: CREATE TABLE failed: Status: {}, Body: {:?}", status, response_body);
            return Err("SQL CREATE TABLE failed".into());
        }
    } else {
        eprintln!("   SQL: CREATE TABLE request failed: Status: {}, Body: {:?}", status, response.text().await?);
        return Err("SQL CREATE TABLE request failed".into());
    }

    // --- 5. Execute SQL Query (INSERT Data) ---
    println!("\n5. Executing SQL: INSERT Data into '{}'...", cache_name);
    let insert_sql = format!(
        "INSERT INTO {} (id, name) VALUES (?, ?)",
        cache_name
    );
    let insert_payload_sql = json!({
        "schemaName": "PUBLIC",
        "query": insert_sql,
        "args": [101, "SQL Inserted Item 1"] // Parameterized query arguments
    });
    let response = client.post(&sql_url)
        .json(&insert_payload_sql)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        let response_body = response.json::<serde_json::Value>().await?; // This consumes `response`
        if response_body["successStatus"].as_i64() == Some(0) {
            println!("   SQL: INSERT for Item 1 executed successfully.");
        } else {
            eprintln!("   SQL: INSERT for Item 1 failed: Status: {}, Body: {:?}", status, response_body);
            return Err("SQL INSERT failed".into());
        }
    } else {
        eprintln!("   SQL: INSERT for Item 1 request failed: Status: {}, Body: {:?}", status, response.text().await?);
        return Err("SQL INSERT request failed".into());
    }

    // --- 6. Execute SQL Query (SELECT Data) ---
    println!("\n6. Executing SQL: SELECT Data from '{}'...", cache_name);
    let select_sql = format!("SELECT id, name FROM {}", cache_name);
    let select_payload = json!({
        "schemaName": "PUBLIC",
        "query": select_sql,
        "pageSize": 100 // Mandatory for SELECT queries
    });

    let response = client.post(&sql_url)
        .json(&select_payload)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        let response_body = response.json::<serde_json::Value>().await?; // This consumes `response`
        if response_body["successStatus"].as_i64() == Some(0) {
            println!("   SQL: SELECT query executed successfully.");
            if let Some(items) = response_body["response"]["items"].as_array() {
                println!("   --- SQL Query Results ---");
                for item in items {
                    // Assuming two fields: id (int) and name (string) based on CREATE TABLE
                    let id = item.get(0).and_then(|v| v.as_i64());
                    let name = item.get(1).and_then(|v| v.as_str());
                    println!("     ID: {:?}, Name: {:?}", id, name);
                }
                println!("   -------------------------");
            } else {
                println!("   No items found in SQL SELECT response.");
            }
        } else {
            eprintln!("   SQL: SELECT query failed: Status: {}, Body: {:?}", status, response_body);
            return Err("SQL SELECT failed".into());
        }
    } else {
        eprintln!("   SQL: SELECT request failed: Status: {}, Body: {:?}", status, response.text().await?);
        return Err("SQL SELECT request failed".into());
    }

    // --- 7. Execute SQL Query (Parameterized SELECT Data) ---
    println!("\n7. Executing SQL: Parameterized SELECT Data from '{}'...", cache_name);
    let parameterized_select_sql = format!("SELECT id, name FROM {} WHERE id = ?", cache_name);
    let parameterized_select_payload = json!({
        "schemaName": "PUBLIC",
        "query": parameterized_select_sql,
        "pageSize": 100,
        "args": [101] // Parameter for the WHERE clause
    });

    let response = client.post(&sql_url)
        .json(&parameterized_select_payload)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        let response_body = response.json::<serde_json::Value>().await?; // This consumes `response`
        if response_body["successStatus"].as_i64() == Some(0) {
            println!("   SQL: Parameterized SELECT query executed successfully.");
            if let Some(items) = response_body["response"]["items"].as_array() {
                println!("   --- SQL Parameterized Query Results ---");
                for item in items {
                    let id = item.get(0).and_then(|v| v.as_i64());
                    let name = item.get(1).and_then(|v| v.as_str());
                    println!("     ID: {:?}, Name: {:?}", id, name);
                }
                println!("   -------------------------");
            } else {
                println!("   No items found in SQL Parameterized SELECT response.");
            }
        } else {
            eprintln!("   SQL: Parameterized SELECT query failed: Status: {}, Body: {:?}", status, response_body);
            return Err("SQL Parameterized SELECT failed".into());
        }
    } else {
        eprintln!("   SQL: Parameterized SELECT request failed: Status: {}, Body: {:?}", status, response.text().await?);
        return Err("SQL Parameterized SELECT request failed".into());
    }

    // --- 8. Destroy the cache (DELETE cache itself) ---
    // Endpoint: /ignite/cache/destroy
    let destroy_cache_url = format!("{}/cache/destroy", base_url);
    println!("\n8. Destroying cache/table '{}'...", cache_name);
    let destroy_payload = json!({
        "cacheName": cache_name
    });

    let response = client.post(&destroy_cache_url)
        .json(&destroy_payload)
        .send()
        .await?;

    let status = response.status(); // Store the status code HERE
    if status.is_success() {
        println!("   Cache/table '{}' destroyed.", cache_name);
    } else {
        eprintln!("   Failed to destroy cache/table: Status: {}, Body: {:?}", status, response.text().await?);
    }

    println!("\n--- Program Finished ---");

    Ok(()) // Indicate success
}