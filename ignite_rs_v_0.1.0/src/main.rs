use ignite_rs::{ClientConfig, Ignite}; // Import Ignite TRAIT
use ignite_rs::cache::Cache; // Import Cache struct
use ignite_rs_derive::IgniteObj; // Import the derive macro
use serde::{Serialize, Deserialize}; // Still needed for general struct serialization

// Define your custom struct to be stored in the cache.
// It must derive IgniteObj for automatic binary serialization/deserialization.
// Clone and Debug are good for convenience.
#[derive(IgniteObj, Clone, Debug, Serialize, Deserialize)]
struct MyValue {
    id: i32,
    name: String,
}

fn main() {
    // 1. Create a client configuration
    let client_config = ClientConfig::new("127.0.0.1:10800"); // Connect to localhost:10800

    // 2. Create the actual client connection.
    // In 0.1.0, this is done via the synchronous function `ignite_rs::new_client`.
    // The `new_client` function returns a `Client` struct, which implements the `Ignite` trait.
    // By having `use ignite_rs::Ignite;` at the top, the methods from this trait
    // (like get_or_create_cache and destroy_cache) become available on the `client` instance.
    println!("Connecting to Apache Ignite...");
    let mut client = ignite_rs::new_client(client_config)
        .expect("Failed to connect to Apache Ignite. Is the cluster running?");

    println!("Successfully connected to Apache Ignite!");

    let cache_name = "my_rust_cache";

    // 3. Get or create a typed cache.
    // The `Cache` type needs to be explicitly typed with your Key and Value types.
    println!("Getting or creating cache '{}'...", cache_name);
    let mut cache: Cache<i32, MyValue> = client // Use 'client' here (returned by new_client)
        .get_or_create_cache(cache_name) // This method is provided by the Ignite trait
        .expect("Failed to get or create cache");

    println!("Cache '{}' is ready.", cache_name);

    // 4. Put data into the cache
    let key1 = 1;
    let value1 = MyValue { id: 1, name: "Hello, Ignite!".to_string() };
    println!("Putting key:{} -> value:{:?} into cache...", key1, value1);
    cache.put(&key1, &value1) // Note: put and get take references in 0.1.0
        .expect("Failed to put data for key 1");
    println!("Data put successfully.");

    let key2 = 2;
    let value2 = MyValue { id: 2, name: "Rust is great!".to_string() };
    println!("Putting key:{} -> value:{:?} into cache...", key2, value2);
    cache.put(&key2, &value2)
        .expect("Failed to put data for key 2");
    println!("Data put successfully.");

    // 5. Get data from the cache
    println!("Getting value for key:{}...", key1);
    let retrieved_value1 = cache.get(&key1) // Note: get takes a reference
        .expect("Failed to get data for key 1");
    match retrieved_value1 {
        Some(val) => println!("Retrieved value for key {}: {:?}", key1, val),
        None => println!("No value found for key {}.", key1),
    }

    println!("Getting value for key:{}...", key2);
    let retrieved_value2 = cache.get(&key2)
        .expect("Failed to get data for key 2");
    match retrieved_value2 {
        Some(val) => println!("Retrieved value for key {}: {:?}", key2, val),
        None => println!("No value found for key {}.", key2),
    }

    // Example of trying to get a non-existent key
    let non_existent_key = 99;
    println!("Attempting to get non-existent key:{}...", non_existent_key);
    let non_existent_val = cache.get(&non_existent_key)
        .expect("Failed to get data for non-existent key");
    match non_existent_val {
        Some(_) => println!("Unexpectedly found value for key {}.", non_existent_key),
        None => println!("Correctly found no value for key {}.", non_existent_key),
    }

    // 6. Destroy the cache (optional) - This method is also provided by the Ignite trait
    println!("Destroying cache '{}'...", cache_name);
    client.destroy_cache(cache_name) // Call destroy_cache on the 'client' instance
        .expect("Failed to destroy cache");
    println!("Cache '{}' destroyed.", cache_name);
}