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

// Генерация фиктивных новостей на основе рыночных данных
fn generate_news_articles(info: &CryptoInfo) -> Vec<(String, String, String)> {
    let mut articles = Vec::new();
    
    // Статья о текущей цене и тренде
    let price_trend = if info.percent_change_24h > 0.0 {
        format!("{} вырос на {:.2}% за последние 24 часа, достигнув ${:.2}", 
            info.name, info.percent_change_24h, info.price)
    } else {
        format!("{} упал на {:.2}% за последние 24 часа, опустившись до ${:.2}", 
            info.name, info.percent_change_24h.abs(), info.price)
    };
    
    articles.push((
        format!("Анализ рынка: {}", price_trend),
        "CoinMarketCap Analysis".to_string(),
        format!("За последние 24 часа {} показал значительное изменение цены в условиях {} рынка. Аналитики отмечают, что текущий объем торгов составляет ${:.2} миллионов, что указывает на {} активность трейдеров. Рыночная капитализация составляет ${:.2} миллиардов.", 
            info.name, 
            if info.percent_change_24h > 0.0 { "растущего" } else { "падающего" },
            info.volume_24h / 1_000_000.0,
            if info.volume_24h > info.market_cap * 0.05 { "высокую" } else { "умеренную" },
            info.market_cap / 1_000_000_000.0)
    ));
    
    // Статья о 7-дневном тренде
    let weekly_trend = if info.percent_change_7d > 0.0 {
        format!("{} демонстрирует рост на {:.2}% за неделю", info.name, info.percent_change_7d)
    } else {
        format!("{} показывает снижение на {:.2}% за неделю", info.name, info.percent_change_7d.abs())
    };
    
    articles.push((
        weekly_trend,
        "Weekly Market Report".to_string(),
        format!("Недельный обзор показывает, что {} {} тренд. Трейдеры отмечают {} волатильность и {} объемы торгов. Технические индикаторы указывают на возможное {} в ближайшие дни.",
            info.name,
            if info.percent_change_7d > 0.0 { "продолжает восходящий" } else { "находится в нисходящем" },
            if (info.percent_change_24h - info.percent_change_7d/7.0).abs() > 1.0 { "повышенную" } else { "стабильную" },
            if info.volume_24h > info.market_cap * 0.1 { "значительные" } else { "средние" },
            if info.percent_change_24h > info.percent_change_7d/7.0 { "продолжение роста" } else { "снижение давления продавцов" })
    ));
    
    // Статья о рыночной капитализации
    articles.push((
        format!("{} занимает важное место в рейтинге криптовалют", info.name),
        "Market Position Update".to_string(),
        format!("С текущей рыночной капитализацией в ${:.2} миллиардов, {} остается значимым активом на криптовалютном рынке. Аналитики отмечают, что соотношение объема торгов к рыночной капитализации составляет {:.2}%, что говорит о {} ликвидности актива.",
            info.market_cap / 1_000_000_000.0,
            info.name,
            info.volume_24h / info.market_cap * 100.0,
            if info.volume_24h / info.market_cap > 0.1 { "высокой" } else { "умеренной" })
    ));
    
    articles
}

async fn fetch_crypto_data(symbol: &str) -> Result<Option<CryptoInfo>, reqwest::Error> {
    let api_key = "a4cf64a8-4b9d-4e93-86a7-e1fc8deec244"; // Ваш ключ API
    let api_url = "https://pro-api.coinmarketcap.com/v1/cryptocurrency/quotes/latest";
    
    let client = Client::new();
    let response = client
        .get(api_url)
        .header("X-CMC_PRO_API_KEY", api_key)
        .query(&[("symbol", symbol)])
        .send()
        .await?;
    
    let body: Value = response.json().await?;
    
    // Проверка на наличие ошибок в ответе API
    if let Some(error) = body.get("status").and_then(|s| s.get("error_message")) {
        if !error.is_null() {
            println!("API error: {}", error);
            return Ok(None);
        }
    }
    
    // Извлечение данных о криптовалюте
    if let Some(data) = body.get("data") {
        if let Some(coin_data) = data.get(symbol) {
            // Извлечение всей доступной информации о цене
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
            // Генерация фиктивных новостных статей на основе данных
            let news_articles = generate_news_articles(&crypto_info);
            
            // Построение HTML-страницы с новостями
            let mut response_html = format!(
                r#"<h2>Latest News for cryptocurrency: {}</h2>"#,
                symbol
            );
            
            for (title, source, content) in news_articles {
                // Генерация случайной даты публикации (последние 24 часа)
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
            
            // Добавление текущей цены и информации о рынке
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
            
            // Добавление стилей для новостного формата
            response_html.push_str(
                r#"
                <style>
                    .news-article {
                        background-color: #fff;
                        border-radius: 8px;
                        box-shadow: 0 2px 8px rgba(0,0,0,0.1);
                        padding: 20px;
                        margin-bottom: 20px;
                    }
                    
                    .news-article h2 {
                        margin-top: 0;
                        font-size: 1.4rem;
                    }
                    
                    .news-article a {
                        color: #333;
                        text-decoration: none;
                    }
                    
                    .news-article a:hover {
                        color: #4a89dc;
                    }
                    
                    .news-content {
                        margin-top: 15px;
                        line-height: 1.6;
                    }
                    
                    .read-more {
                        display: inline-block;
                        margin-top: 10px;
                        color: #4a89dc !important;
                        font-weight: 500;
                    }
                    
                    .market-summary {
                        background-color: #f8f9fa;
                        border-radius: 8px;
                        padding: 20px;
                        margin: 30px 0;
                    }
                    
                    .market-summary h3 {
                        margin-top: 0;
                        margin-bottom: 15px;
                    }
                    
                    .market-data {
                        display: flex;
                        justify-content: space-between;
                        flex-wrap: wrap;
                    }
                    
                    .price-item, .data-item {
                        display: flex;
                        flex-direction: column;
                        padding: 10px 15px;
                        background-color: white;
                        border-radius: 6px;
                        min-width: 150px;
                    }
                    
                    .price-up .value {
                        color: #28a745;
                    }
                    
                    .price-down .value {
                        color: #dc3545;
                    }
                    
                    .label {
                        font-size: 0.9rem;
                        color: #666;
                        margin-bottom: 5px;
                    }
                    
                    .value {
                        font-size: 1.1rem;
                        font-weight: 500;
                    }
                </style>
                "#
            );
            
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