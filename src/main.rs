use headset::Headset;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let client = match Headset::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create Headset client: {e}");
            std::process::exit(1);
        }
    };

    // Fetch a real-time quote for AAPL
    match client.quote("AAPL").await {
        Ok(q) => {
            println!("--- Quote for {} ---", q.symbol);
            println!("  Price:    ${:.2}", q.current_price);
            println!("  Change:   {:.2} ({:.2}%)", q.change, q.percent_change);
            println!("  High:     ${:.2}", q.high);
            println!("  Low:      ${:.2}", q.low);
            println!("  Open:     ${:.2}", q.open);
            println!("  Prev Close: ${:.2}", q.previous_close);
        }
        Err(e) => eprintln!("Quote error: {e}"),
    }

    // Fetch upcoming earnings calendar
    match client.earnings(None, None, None).await {
        Ok(reports) => {
            println!("\n--- Upcoming Earnings ({} entries) ---", reports.len());
            for r in reports.iter().take(10) {
                println!(
                    "  {} | {} | Q{} {} | EPS est: {:?}",
                    r.symbol.as_deref().unwrap_or("???"),
                    r.date.as_deref().unwrap_or("n/a"),
                    r.quarter.unwrap_or(0),
                    r.year.unwrap_or(0),
                    r.eps_estimate,
                );
            }
        }
        Err(e) => eprintln!("Earnings error: {e}"),
    }
}
