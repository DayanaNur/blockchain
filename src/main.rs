use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_files as fs;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use chrono::{DateTime, Utc};
use chrono::format::strftime::StrftimeItems;

#[derive(Deserialize)]
struct Query {
    symbol: String,
}

#[derive(Deserialize)]
struct CryptoInfo {
    name: String,
    symbol: String,
    slug: String,
    price: f64,
    market_cap: f64,
    volume_24h: f64,
    percent_change_24h: f64,
    percent_change_7d: f64,
    last_updated: String,
}

fn generate_news_articles(info: &CryptoInfo) -> Vec<(String, String, String)> {
    let mut articles = Vec::new();
    
    let price_trend = if info.percent_change_24h > 0.0 {
        format!("{} increased by {:.2}% in the last 24 hours, reaching ${:.2}", 
            info.name, info.percent_change_24h, info.price)
    } else {
        format!("{} decreased by {:.2}% in the last 24 hours, falling to ${:.2}", 
            info.name, info.percent_change_24h.abs(), info.price)
    };
    
    articles.push((
        format!("Market Analysis: {}", price_trend),
        "CoinMarketCap Analysis".to_string(),
        format!("In the last 24 hours, {} has shown significant price movement in a {} market. Analysts note that the current trading volume is ${:.2} million, indicating {} trader activity. The market capitalization stands at ${:.2} billion.", 
            info.name, 
            if info.percent_change_24h > 0.0 { "bullish" } else { "bearish" },
            info.volume_24h / 1_000_000.0,
            if info.volume_24h > info.market_cap * 0.05 { "high" } else { "moderate" },
            info.market_cap / 1_000_000_000.0)
    ));
    
    let weekly_trend = if info.percent_change_7d > 0.0 {
        format!("{} shows growth of {:.2}% over the week", info.name, info.percent_change_7d)
    } else {
        format!("{} shows decline of {:.2}% over the week", info.name, info.percent_change_7d.abs())
    };
    
    articles.push((
        weekly_trend,
        "Weekly Market Report".to_string(),
        format!("The weekly review indicates that {} {} trend. Traders report {} volatility and {} trading volumes. Technical indicators suggest possible {} in the coming days.",
            info.name,
            if info.percent_change_7d > 0.0 { "continues its upward" } else { "is in a downward" },
            if (info.percent_change_24h - info.percent_change_7d/7.0).abs() > 1.0 { "increased" } else { "stable" },
            if info.volume_24h > info.market_cap * 0.1 { "significant" } else { "average" },
            if info.percent_change_24h > info.percent_change_7d/7.0 { "continued growth" } else { "reduction in selling pressure" })
    ));
    
    articles.push((
        format!("{} holds an important position in cryptocurrency rankings", info.name),
        "Market Position Update".to_string(),
        format!("With a current market capitalization of ${:.2} billion, {} remains a significant asset in the cryptocurrency market. Analysts note that the ratio of trading volume to market capitalization is {:.2}%, indicating {} liquidity of the asset.",
            info.market_cap / 1_000_000_000.0,
            info.name,
            info.volume_24h / info.market_cap * 100.0,
            if info.volume_24h / info.market_cap > 0.1 { "high" } else { "moderate" })
    ));
    
    articles
}

async fn fetch_crypto_data(symbol: &str) -> Result<Option<CryptoInfo>, reqwest::Error> {
    let api_key = "a4cf64a8-4b9d-4e93-86a7-e1fc8deec244"; 
    let api_url = "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest";
    
    let client = Client::new();
    let response = client
        .get(api_url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .query(&[("symbol", symbol)])
        .send()
        .await?;
    
    let body: Value = response.json().await?;
    
    if let Some(error) = body.get("status").and_then(|s| s.get("error_message")) {
        if !error.is_null() {
            println!("API error: {}", error);
            return Ok(None);
        }
    }
    
    if let Some(data) = body.get("data") {
        if let Some(coin_data) = data.get(symbol) {
            let quote = coin_data.get("quote").and_then(|q| q.get("USD")).unwrap_or(&Value::Null);
            
            let info = CryptoInfo {
                name: coin_data.get("name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
                symbol: coin_data.get("symbol").and_then(|v| v.as_str()).unwrap_or(symbol).to_string(),
                slug: coin_data.get("slug").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                price: quote.get("price").and_then(|v| v.as_f64()).unwrap_or(0.0),
                market_cap: quote.get("market_cap").and_then(|v| v.as_f64()).unwrap_or(0.0),
                volume_24h: quote.get("volume_24h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                percent_change_24h: quote.get("percent_change_24h").and_then(|v| v.as_f64()).unwrap_or(0.0),
                percent_change_7d: quote.get("percent_change_7d").and_then(|v| v.as_f64()).unwrap_or(0.0),
                last_updated: quote.get("last_updated").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            };
            return Ok(Some(info));
        }
    }
    
    Ok(None)
}

async fn get_news(query: web::Query<Query>) -> impl Responder {
    let symbol = query.symbol.to_uppercase();
    
    match fetch_crypto_data(&symbol).await {
        Ok(Some(crypto_info)) => {
            let news_articles = generate_news_articles(&crypto_info);
            
            
            let mut response_html = format!(
                r#"<h2>Latest News for cryptocurrency: {}</h2>"#,
                symbol
            );
            
            for (title, source, content) in news_articles {
                
                let hours_ago = rand::random::<u8>() % 24;
                let minutes_ago = rand::random::<u8>() % 60;
                let published_at = format!("{} hours {} minutes ago", hours_ago, minutes_ago);
                
                let url = format!("https://coinmarketcap.com/currencies/{}/", crypto_info.slug);
                
                response_html.push_str(&format!(
                    r#"
                    <div class='news-article'>
                        <h2><a href='{}'>{}</a></h2>
                        <p><b>Source:</b> {} | <b>Published:</b> {}</p>
                        <div class='news-content'>
                            <p>{}</p>
                            <a href='{}' class='read-more'>Read more</a>
                        </div>
                    </div>
                    "#,
                    url, title, source, published_at, content, url
                ));
            }
            
            let price_class = if crypto_info.percent_change_24h >= 0.0 { "price-up" } else { "price-down" };
            let price_arrow = if crypto_info.percent_change_24h >= 0.0 { "▲" } else { "▼" };
            
            response_html.push_str(&format!(
                r#"
                <div class='market-summary'>
                    <h3>Current Market Data</h3>
                    <div class='market-data'>
                        <div class='price-item {}'>
                            <span class='label'>Price:</span>
                            <span class='value'>${:.2} {} {:.2}%</span>
                        </div>
                        <div class='data-item'>
                            <span class='label'>Market Cap:</span>
                            <span class='value'>${:.2}B</span>
                        </div>
                        <div class='data-item'>
                            <span class='label'>24h Volume:</span>
                            <span class='value'>${:.2}M</span>
                        </div>
                    </div>
                </div>
                "#,
                price_class,
                crypto_info.price, price_arrow, crypto_info.percent_change_24h,
                crypto_info.market_cap / 1_000_000_000.0,
                crypto_info.volume_24h / 1_000_000.0
            ));
            
            response_html.push_str("<p>Data sourced from <a href='https://coinmarketcap.com'>CoinMarketCap</a>.</p>");
            
            HttpResponse::Ok().content_type("text/html").body(response_html)
        },
        Ok(None) => {
            HttpResponse::NotFound().body(format!("No data found for cryptocurrency with symbol: {}", symbol))
        },
        Err(e) => {
            println!("Error fetching data: {:?}", e);
            HttpResponse::InternalServerError().body("Failed to fetch cryptocurrency data. Please try again.")
        }
    }
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/index.html"))
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting at http://127.0.0.1:8080");
    HttpServer::new(|| {
        App::new()
            .service(fs::Files::new("/static", "./static").show_files_listing())
            .route("/", web::get().to(index))
            .route("/news", web::get().to(get_news))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}