use reqwest;
use serde_json::Value;
use std::io::{self, Write};
use tokio::time::{sleep, Duration};
use crossterm::{QueueableCommand, terminal, cursor};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Enter the token ID on CoinGecko:");
    let mut token_id = String::new();
    io::stdin().read_line(&mut token_id)?;
    let token_id = token_id.trim();

    let mut stdout = io::stdout();

    loop {
        let url = format!(
            "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_market_cap=true&include_24hr_vol=true",
            token_id
        );
        let response = reqwest::get(&url).await?.json::<Value>().await?;
        if let Some(price) = response[token_id]["usd"].as_f64() {
            let market_cap = response[token_id]["usd_market_cap"].as_f64().unwrap_or(0.0);
            let volume_24h = response[token_id]["usd_24h_vol"].as_f64().unwrap_or(0.0);

            stdout.queue(terminal::Clear(terminal::ClearType::All))?;
            stdout.queue(cursor::MoveTo(0, 0))?;
            print!(
                "Price {}: ${:.2}\nMarket cap: ${:.2}\n24h volume: ${:.2}",
                token_id, price, market_cap, volume_24h
            );
            stdout.flush()?;
        } else {
            stdout.queue(terminal::Clear(terminal::ClearType::All))?;
            stdout.queue(cursor::MoveTo(0, 0))?;
            println!("Failed to retrieve data for the token {}.", token_id);
            stdout.flush()?;
        }

        sleep(Duration::from_secs(60)).await;
    }
}
