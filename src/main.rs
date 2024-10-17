use reqwest::Client;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::error::Error;
use tokio::time::{sleep, Duration};
use eframe::egui;

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Price Checker",
        options,
        Box::new(|_cc| Ok(Box::new(MyApp::default()))),
    )
}

#[derive(Default, Clone)]
struct MyApp {
    tokenid: Arc<Mutex<String>>,
    price: Arc<Mutex<f64>>,
    marketcap: Arc<Mutex<f64>>,
    volume: Arc<Mutex<f64>>,
    autoupdate: Arc<Mutex<bool>>,
    isfetching: Arc<Mutex<bool>>,
    isupdating: Arc<Mutex<bool>>,
}

async fn gettoken(client: &Client, token_id: &str) -> Result<(f64, f64, f64), Box<dyn Error>> {
    let url = format!(
        "https://api.coingecko.com/api/v3/simple/price?ids={}&vs_currencies=usd&include_market_cap=true&include_24hr_vol=true",
        token_id
    );

    let response = client.get(&url).send().await?.json::<Value>().await?;
    //println!("ok");
    if let Some(price) = response[token_id]["usd"].as_f64() {
        let marketcap = response[token_id]["usd_market_cap"].as_f64().unwrap_or(0.0);
        let volume = response[token_id]["usd_24h_vol"].as_f64().unwrap_or(0.0);
        Ok((price, marketcap, volume))
    } else {
        Err("err".into())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Price Checker");

            ui.label("Token ID (from coingecko):");
            let mut tokenid_clone = self.tokenid.lock().unwrap().clone();
            ui.text_edit_singleline(&mut tokenid_clone);
            *self.tokenid.lock().unwrap() = tokenid_clone.clone();

            if ui.button("Fetch").clicked() {
                let tokenid = Arc::clone(&self.tokenid);
                let price = Arc::clone(&self.price);
                let marketcap = Arc::clone(&self.marketcap);
                let volume = Arc::clone(&self.volume);
                let isfetching = Arc::clone(&self.isfetching);
                let ctx_clone = ctx.clone();
                tokio::spawn(async move {
                    if *isfetching.lock().unwrap() {
                        return;
                    }
                    *isfetching.lock().unwrap() = true;
                    let client = Client::new();
                    let token_id = tokenid.lock().unwrap().clone();
                    match gettoken(&client, &token_id).await {
                        Ok((p, m, v)) => {
                            *price.lock().unwrap() = p;
                            *marketcap.lock().unwrap() = m;
                            *volume.lock().unwrap() = v;
                        }
                        Err(_) => {
                            *price.lock().unwrap() = 0.0;
                            *marketcap.lock().unwrap() = 0.0;
                            *volume.lock().unwrap() = 0.0;
                        }
                    }
                    *isfetching.lock().unwrap() = false;
                    ctx_clone.request_repaint();
                });
            }

            let autoupdate = Arc::clone(&self.autoupdate);
            if ui.button("Auto-Update").clicked() {
                let mut autoupdate_state = autoupdate.lock().unwrap();
                *autoupdate_state = !*autoupdate_state;
            }

            let isupdating = Arc::clone(&self.isupdating);
            if *self.autoupdate.lock().unwrap() && !*isupdating.lock().unwrap() {
                let tokenid = Arc::clone(&self.tokenid);
                let price = Arc::clone(&self.price);
                let marketcap = Arc::clone(&self.marketcap);
                let volume = Arc::clone(&self.volume);
                let autoupdate = Arc::clone(&self.autoupdate);
                let isfetching = Arc::clone(&self.isfetching);
                let isupdating = Arc::clone(&self.isupdating);
                let ctx_clone = ctx.clone();
                *isupdating.lock().unwrap() = true;

                tokio::spawn(async move {
                    *isfetching.lock().unwrap() = true;
                    let client = Client::new();
                    let token_id = tokenid.lock().unwrap().clone();
                    match gettoken(&client, &token_id).await {
                        Ok((p, m, v)) => {
                            *price.lock().unwrap() = p;
                            *marketcap.lock().unwrap() = m;
                            *volume.lock().unwrap() = v;
                        }
                        Err(_) => {
                            *price.lock().unwrap() = 0.0;
                            *marketcap.lock().unwrap() = 0.0;
                            *volume.lock().unwrap() = 0.0;
                        }
                    }
                    *isfetching.lock().unwrap() = false;
                    ctx_clone.request_repaint();

                    while *autoupdate.lock().unwrap() {
                        sleep(Duration::from_secs(15)).await;
                        if *isfetching.lock().unwrap() {
                            continue;
                        }
                        *isfetching.lock().unwrap() = true;
                        let token_id = tokenid.lock().unwrap().clone();
                        match gettoken(&client, &token_id).await {
                            Ok((p, m, v)) => {
                                *price.lock().unwrap() = p;
                                *marketcap.lock().unwrap() = m;
                                *volume.lock().unwrap() = v;
                            }
                            Err(_) => {
                                *price.lock().unwrap() = 0.0;
                                *marketcap.lock().unwrap() = 0.0;
                                *volume.lock().unwrap() = 0.0;
                            }
                        }
                        *isfetching.lock().unwrap() = false;
                        ctx_clone.request_repaint();
                    }
                    *isupdating.lock().unwrap() = false;
                });
            }

            let price_display = *self.price.lock().unwrap();
            let marketcap_display = *self.marketcap.lock().unwrap();
            let volume_display = *self.volume.lock().unwrap();

            ui.label(format!("Price: ${:.4}", price_display));
            ui.label(format!("Market Cap: ${:.4}", marketcap_display));
            ui.label(format!("24h Volume: ${:.4}", volume_display));
        });
    }
}
