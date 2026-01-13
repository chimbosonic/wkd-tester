use clap::Parser;
use miette::Report;
use miette::Result;
use wkd::fetch::{WkdFetch, WkdFetchUriResult};
use wkd::load::load_key;
use wkd::uri::WkdUri;

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

    let wkd_fetch = WkdFetch::fetch(&wkd_uri, None).await;

    unwrap_wkd_fetch(wkd_fetch.advanced_method, "Advanced");
    unwrap_wkd_fetch(wkd_fetch.direct_method, "Direct");

    Ok(())
}

fn unwrap_wkd_fetch(wkd_fetch: WkdFetchUriResult, method: &str) {
    if wkd_fetch.data.is_none() {
        println!("{method} method fetch failed with following errors:");
        for error in wkd_fetch.errors {
            println!("{:?}", Report::new(error));
        }
        return;
    }

    if !wkd_fetch.errors.is_empty() {
        println!("{method} method fetch was successful with warnings:");
        for error in wkd_fetch.errors {
            println!("{:?}", Report::new(error));
        }
    }

    println!("{method} tests:");
    for success in wkd_fetch.successes {
        println!(" - {success:?} Passed")
    }

    if let Some(data) = wkd_fetch.data {
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
                println!(
                    "{method} method key loading succeed with expiry status: {}",
                    key.expiry
                );
                println!(
                    "{method} method key loading succeed with algorithm: {}",
                    key.algorithm
                );
                println!(
                    "{method} method key loading succeed with randomart:\n{}",
                    key.randomart
                );
            }
            Err(error) => {
                println!("{method} method key loading failed with following errors:");
                println!("{:?}", Report::new(error));
            }
        };
    }
}
