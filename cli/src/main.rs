use clap::Parser;
use miette::Report;
use miette::Result;
use wkd_fetch::WkdFetchError;
use wkd_fetch::{fetch_uri, WkdFetch};
use wkd_load::load_key;
use wkd_uri::WkdUri;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
struct Args {
    /// The GPG User ID to look up (example: Joe.Doe@example.org)
    #[arg(short, long, required = true)]
    user_id: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let user_id = args.user_id;

    let wkd_uri = WkdUri::new(user_id)?;

    println!("Advanced method URI: {}", wkd_uri.advanced_uri);
    println!("Direct method URI: {}", wkd_uri.direct_uri);

    let wkd_fetch_advanced = fetch_uri(&wkd_uri.advanced_uri).await;
    let wkd_fetch_direct = fetch_uri(&wkd_uri.direct_uri).await;

    unwrap_wkd_fetch(wkd_fetch_advanced, "Advanced");
    unwrap_wkd_fetch(wkd_fetch_direct, "Direct");

    Ok(())
}

fn unwrap_wkd_fetch(wkd_fetch: Result<WkdFetch, WkdFetchError>, method: &str) {
    match wkd_fetch {
        Ok(wkd_fetch) => {
            if !wkd_fetch.errors.is_empty() {
                println!("{method} method fetch was successful with warnings:");
                for error in wkd_fetch.errors {
                    println!("{:?}", Report::new(error));
                }
            } else {
                println!("{method} method fetch was successful");
            }

            if let Some(data) = wkd_fetch.data {
                match load_key(data) {
                    Ok(fingerprint) => {
                        println!(
                            "{method} method key loading succeed with fingerprint: {}",
                            fingerprint
                        );
                    }
                    Err(error) => {
                        println!("{method} method key loading failed with error:");
                        println!("{:?}", Report::new(error));
                    }
                };
            } else {
                println!("{method} method fetch returned no data");
            }
        }
        Err(error) => {
            println!("{method} method fetch failed with error:");
            println!("{:?}", Report::new(error));
        }
    }
}
