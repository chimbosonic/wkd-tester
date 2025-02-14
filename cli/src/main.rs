use wkd_uri::WkdUri;
use clap::Parser;
use miette::Result;

#[derive(Parser, Debug)]
#[command(version, author, about, long_about = None)]
struct Args {
    /// The GPG User ID to look up (example: Joe.Doe@example.org)
    #[arg(short, long, required = true)]
    user_id: String,
}


fn main() -> Result<()> {
    let args = Args::parse();
    let user_id = args.user_id;

    let wkd_uri =  WkdUri::new(user_id)?;
    
    println!("Advanced Method Uri: {}", wkd_uri.advanced_uri );
    println!("Direct Method Uri: {}", wkd_uri.direct_uri );

    Ok(())
}
