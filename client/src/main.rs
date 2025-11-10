use client::{SpaceDevsClient, SpaceDevsDataClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SpaceDevs Client Example");

    // Create a new news client
    let news_client = SpaceDevsClient::new();

    // Create a new data client
    let _data_client = SpaceDevsDataClient::new();

    // Fetch articles with structured data
    println!("Fetching structured articles...");
    match news_client.get_articles_structured().await {
        Ok(response) => {
            println!("Successfully fetched {} articles!", response.count);
            if let Some(first_article) = response.results.first() {
                println!("First article: {}", first_article.title);
                println!("Published at: {}", first_article.published_at);
                println!("Authors: {}", first_article.authors.len());
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch structured articles: {}", e);
        }
    }

    // Fetch blogs with structured data
    println!("\nFetching structured blogs...");
    match news_client.get_blogs_structured().await {
        Ok(response) => {
            println!("Successfully fetched {} blogs!", response.count);
            if let Some(first_blog) = response.results.first() {
                println!("First blog: {}", first_blog.title);
                println!("Published at: {}", first_blog.published_at);
                println!("Authors: {}", first_blog.authors.len());
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch structured blogs: {}", e);
        }
    }

    // Fetch a single article by ID if we have any
    println!("\nFetching single article...");
    match news_client.get_articles_structured().await {
        Ok(response) => {
            if let Some(first_article) = response.results.first() {
                match news_client.get_article(first_article.id).await {
                    Ok(article) => {
                        println!(
                            "Successfully fetched article #{}: {}",
                            article.id, article.title
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch single article: {}", e);
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Failed to get articles list: {}", e);
        }
    }

    // Demonstrate data client usage
    println!("\nFetching data from SpaceDevs API...");
    // Example: This would fetch data from the SpaceDevs data API
    // For example: launches, events, etc.
    println!("SpaceDevs Data Client ready for use!");

    Ok(())
}
