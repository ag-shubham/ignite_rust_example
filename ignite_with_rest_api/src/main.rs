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
    let base_url = "http://127.0.0.1:8080/ignite"; // Default Ignite REST API endpoint
    let cache_name = "my_rest_cache"; // Name of the cache we'll interact with

    // --- 1. Create a Cache (or get it if it already exists) ---
    // Endpoint: /ignite/cache/getOrCreate
    // This is optional, but good practice to ensure the cache exists.
    let create_cache_url = format!("{}/cache/getOrCreate", base_url);
    println!("Creating/getting cache '{}'...", cache_name);
    let create_payload = json!({
        "cacheName": cache_name
    });

    let response = client.post(&create_cache_url)
        .json(&create_payload) // Send JSON payload
        .send() // Send the request
        .await?; // Wait for the response

    if response.status().is_success() { // Check if HTTP status is 2xx (success)
        println!("Cache '{}' ensured (created or already exists).", cache_name);
    } else {
        // If not successful, print the error
        eprintln!("Failed to ensure cache: Status: {}, Body: {:?}", response.status(), response.text().await?);
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
    println!("\nPutting key:{} -> value:{:?} into cache...", key1, value1);
    let response = client.post(&put_url)
        .json(&put_payload1)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Data for key {} put successfully.", key1);
    } else {
        eprintln!("Failed to put data for key {}: Status: {}, Body: {:?}", key1, response.status(), response.text().await?);
        return Err(format!("Failed to put data for key {}", key1).into());
    }

    let key2 = 2;
    let value2 = MyValue { id: 2, name: "Another entry!".to_string() };
    let put_payload2 = json!({
        "cacheName": cache_name,
        "key": key2,
        "value": value2
    });
    println!("Putting key:{} -> value:{:?} into cache...", key2, value2);
    let response = client.post(&put_url)
        .json(&put_payload2)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Data for key {} put successfully.", key2);
    } else {
        eprintln!("Failed to put data for key {}: Status: {}, Body: {:?}", key2, response.status(), response.text().await?);
        return Err(format!("Failed to put data for key {}", key2).into());
    }


    // --- 3. Get data from the cache (READ) ---
    // Endpoint: /ignite/cache/get
    let get_url = format!("{}/cache/get", base_url); // This is the correct URL for GET operations

    // Get key1
    let get_payload1 = json!({
        "cacheName": cache_name,
        "key": key1
    });
    println!("\nGetting value for key:{}...", key1);
    let response_body = client.post(&get_url) // CORRECTED: Use `&get_url` here
        .json(&get_payload1)
        .send()
        .await?
        .json::<serde_json::Value>() // Deserialize response as a generic JSON value
        .await?;

    if let Some(value_obj) = response_body.get("response") {
        if value_obj.is_null() {
            println!("No value found for key {}.", key1);
        } else {
            let retrieved_value1: MyValue = serde_json::from_value(value_obj.clone())?;
            println!("Retrieved value for key {}: {:?}", key1, retrieved_value1);
        }
    } else {
        eprintln!("'response' field missing in GET response for key {}: {:?}", key1, response_body);
    }

    // Get a non-existent key
    let non_existent_key = 99;
    let get_payload_non_existent = json!({
        "cacheName": cache_name,
        "key": non_existent_key
    });
    println!("\nAttempting to get non-existent key:{}...", non_existent_key);
    // This was the problematic line:
    let response_body = client.post(&get_url) // CORRECTED: Use `&get_url` here again
        .json(&get_payload_non_existent)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;

    if let Some(value_obj) = response_body.get("response") {
        if value_obj.is_null() {
            println!("Correctly found no value for non-existent key {}.", non_existent_key);
        } else {
            eprintln!("Unexpectedly found value for non-existent key {}: {:?}", non_existent_key, value_obj);
        }
    } else {
        eprintln!("'response' field missing for non-existent key {}: {:?}", non_existent_key, response_body);
    }

    
    // --- 4. Destroy the cache (DELETE cache itself) ---
    // Endpoint: /ignite/cache/destroy
    let destroy_cache_url = format!("{}/cache/destroy", base_url);
    println!("\nDestroying cache '{}'...", cache_name);
    let destroy_payload = json!({
        "cacheName": cache_name
    });

    let response = client.post(&destroy_cache_url)
        .json(&destroy_payload)
        .send()
        .await?;

    if response.status().is_success() {
        println!("Cache '{}' destroyed.", cache_name);
    } else {
        eprintln!("Failed to destroy cache: Status: {}, Body: {:?}", response.status(), response.text().await?);
    }

    Ok(()) // Indicate success
}