use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize)]
struct Query {
    symbol: String,
}

#[derive(Deserialize)]
struct NewsArticle {
    title: String,
    source: String,
    published_at: String,
    url: String,
}

async fn fetch_crypto_news(symbol: &str) -> Result<Vec<NewsArticle>, reqwest::Error> {
    let api_url = format!("https://api.coingecko.com/api/v3/coins/{}/news", symbol);
    let client = Client::new();
    
    let response = client.get(&api_url).send().await?;
    let body: Value = response.json().await?;
    
    let articles = body["articles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|article| NewsArticle {
            title: article["title"].as_str().unwrap().to_string(),
            source: article["source"]["name"].as_str().unwrap().to_string(),
            published_at: article["published_at"].as_str().unwrap().to_string(),
            url: article["url"].as_str().unwrap().to_string(),
        })
        .collect();
    
    Ok(articles)
}

async fn get_news(query: web::Query<Query>) -> impl Responder {
    let news = match fetch_crypto_news(&query.symbol).await {
        Ok(news) => news,
        Err(_) => return HttpResponse::InternalServerError().body("Failed to fetch news"),
    };

    let mut response_html = String::from("<h2>Latest News for cryptocurrency: </h2>");
    for article in news {
        response_html.push_str(&format!(
            "<div class='news-article'>
                <h2><a href='{}'>{}</a></h2>
                <p><b>Source:</b> {} | <b>Published:</b> {}</p>
            </div>",
            article.url, article.title, article.source, article.published_at
        ));
    }

    HttpResponse::Ok().body(response_html)
}

async fn index() -> impl Responder {
    // Serve the index.html file directly
    HttpResponse::Ok().body(include_str!("../static/index.html"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(index))  // Use a named function instead of a closure
            .route("/news", web::get().to(get_news))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
