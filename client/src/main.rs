use client::{RESTClient, SchemaManager, article::Article, paginated::Paginated};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Generic REST Client Examples ---");

    // Example of using the generic REST client with SpaceFlight News API
    let space_client = RESTClient::new("https://api.spaceflightnewsapi.net/v4");

    // Fetch articles using static typing
    println!("\nFetching articles with static typing...");
    match space_client.get::<Paginated<Article>>("articles").await {
        Ok(articles) => {
            println!("Successfully fetched {} articles!", articles.count);

            if let Some(first_article) = articles.results.first() {
                println!("First article: {}", first_article.title);
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch articles: {}", e);
        }
    }

    // Example of using dynamic schema loading
    println!("\n--- Dynamic Schema Loading Example ---");

    // Create a schema manager and load schemas from TOML file
    let mut schema_manager = SchemaManager::new();
    match schema_manager.load_from_file("schemas.toml") {
        Ok(()) => {
            println!("Successfully loaded schemas from schemas.toml");

            // Create a REST client with schema support
            let dynamic_client =
                RESTClient::with_schemas("https://api.spaceflightnewsapi.net/v4", schema_manager);

            // Fetch articles using dynamic schema
            println!("\nFetching articles with dynamic schema...");
            match dynamic_client.get_with_schema("articles", "article").await {
                Ok(data) => {
                    println!("Successfully fetched articles with dynamic schema!");
                    println!("Data structure: {:?}", data);
                }
                Err(e) => {
                    eprintln!("Failed to fetch articles with dynamic schema: {}", e);
                }
            }

            // Example of using query parameters with dynamic schema
            println!("\n--- Query Parameters Example ---");

            // Create query parameters
            let mut params = HashMap::new();
            params.insert("limit".to_string(), "5".to_string());
            params.insert("offset".to_string(), "10".to_string());
            params.insert("ordering".to_string(), "-published_at".to_string()); // Order by published date descending

            // Fetch articles with query parameters
            println!(
                "Fetching articles with query parameters (limit=5, offset=10, ordered by date)..."
            );
            match dynamic_client
                .get_with_params::<Paginated<Article>>("articles", "article", &params)
                .await
            {
                Ok(articles) => {
                    println!(
                        "Successfully fetched {} articles with query parameters!",
                        articles.count
                    );
                    if let Some(first_article) = articles.results.first() {
                        println!("First article: {}", first_article.title);
                        println!("Published at: {}", first_article.published_at);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch articles with query parameters: {}", e);
                }
            }

            // Example with date filtering
            let mut date_params = HashMap::new();
            date_params.insert("limit".to_string(), "3".to_string());
            date_params.insert("published_at__gte".to_string(), "2025-11-10".to_string());

            println!("\nFetching recent articles (published after 2025-11-10)...");
            match dynamic_client
                .get_with_params::<Paginated<Article>>("articles", "article", &date_params)
                .await
            {
                Ok(articles) => {
                    println!("Successfully fetched {} recent articles!", articles.count);
                    for article in articles.results.iter() {
                        println!("- {} (Published: {})", article.title, article.published_at);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to fetch recent articles: {}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to load schemas: {}", e);
            println!("Skipping dynamic schema example...");
        }
    }

    Ok(())
}
