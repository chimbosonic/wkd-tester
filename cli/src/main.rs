use clap::Parser;
use miette::Report;
use miette::Result;
use wkd::fetch::{WkdFetch, WkdFetchUriResult};
use wkd::uri::WkdUri;
use wkd::load::load_key;

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

    let wkd_uri = WkdUri::new(&user_id)?;

    println!("Advanced method URI: {}", wkd_uri.advanced_uri);
    println!("Direct method URI: {}", wkd_uri.direct_uri);

    let wkd_fetch = WkdFetch::fetch(&wkd_uri).await;

    unwrap_wkd_fetch(wkd_fetch.advanced_method, "Advanced");
    unwrap_wkd_fetch(wkd_fetch.direct_method, "Direct");

    Ok(())
}

fn unwrap_wkd_fetch(wkd_fetch: WkdFetchUriResult, method: &str) {
    if let Some(data) = wkd_fetch.data {
        if !wkd_fetch.errors.is_empty() {
            println!("{method} method fetch was successful with warnings:");
            for error in wkd_fetch.errors {
                println!("{:?}", Report::new(error));
            }
        } else {
            println!("{method} method fetch was successful");
            match load_key(data) {
                Ok(key) => {
                    println!(
                        "{method} method key loading succeed with fingerprint: {}",
                        key.fingerprint
                    );
                    println!(
                        "{method} method key loading succeed with revocation status: {}",
                        key.revocation_status
                    );
                }
                Err(error) => {
                    println!("{method} method key loading failed with error:");
                    println!("{:?}", Report::new(error));
                }
            };
        }
    } else {
        println!("{method} method fetch failed with error:");
        for error in wkd_fetch.errors {
            println!("{:?}", Report::new(error));
        }
    }
}
