use client::{RESTClient, SchemaManager, article::Article, paginated::Paginated};

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
        }
        Err(e) => {
            eprintln!("Failed to load schemas: {}", e);
            println!("Skipping dynamic schema example...");
        }
    }

    Ok(())
}
