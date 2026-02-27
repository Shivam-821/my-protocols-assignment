use anyhow::Result;
use dhcproto::v4::{
    Decodable, DhcpOption, Encodable, Encoder, Flags, Message, MessageType, Opcode,
};
use dhcproto::Decoder;
use std::collections::HashMap;
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;

// ── Configuration ─────────────────────────────────────────────────────────────

const SERVER_IP: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
const SUBNET_MASK: Ipv4Addr = Ipv4Addr::new(255, 255, 255, 0);
const GATEWAY: Ipv4Addr = Ipv4Addr::new(192, 168, 1, 1);
const LEASE_TIME_SECS: u32 = 86400; // 24 hours

// Dynamic pool: 192.168.1.100 – 192.168.1.200
const POOL_START: u32 = 0xC0A80164; // 192.168.1.100
const POOL_END: u32 = 0xC0A801C8;   // 192.168.1.200

// Static leases: map MAC bytes → IP
fn static_leases() -> HashMap<Vec<u8>, Ipv4Addr> {
    let mut m = HashMap::new();
    // Example: AA:BB:CC:DD:EE:FF → 192.168.1.50
    m.insert(
        vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF],
        Ipv4Addr::new(192, 168, 1, 50),
    );
    m
}

// ── Lease state ───────────────────────────────────────────────────────────────

#[derive(Default)]
struct LeaseDb {
    /// MAC → assigned IP
    leases: HashMap<Vec<u8>, Ipv4Addr>,
    /// IP → MAC (reverse, to detect conflicts)
    allocated: HashMap<Ipv4Addr, Vec<u8>>,
    next_pool_ip: u32,
}

impl LeaseDb {
    fn new() -> Self {
        Self {
            next_pool_ip: POOL_START,
            ..Default::default()
        }
    }

    fn assign(&mut self, mac: &[u8]) -> Option<Ipv4Addr> {
        // Already has a lease?
        if let Some(ip) = self.leases.get(mac) {
            return Some(*ip);
        }
        // Static lease?
        if let Some(ip) = static_leases().get(mac) {
            self.leases.insert(mac.to_vec(), *ip);
            self.allocated.insert(*ip, mac.to_vec());
            return Some(*ip);
        }
        // Allocate from pool
        while self.next_pool_ip <= POOL_END {
            let ip = Ipv4Addr::from(self.next_pool_ip);
            self.next_pool_ip += 1;
            if !self.allocated.contains_key(&ip) {
                self.leases.insert(mac.to_vec(), ip);
                self.allocated.insert(ip, mac.to_vec());
                return Some(ip);
            }
        }
        None // pool exhausted
    }

    fn release(&mut self, mac: &[u8]) {
        if let Some(ip) = self.leases.remove(mac) {
            self.allocated.remove(&ip);
        }
    }
}

// ── Server ────────────────────────────────────────────────────────────────────

pub async fn run_dhcp_server() -> Result<()> {
    // DHCP uses port 67 (requires root/admin). Use 6767 for testing without root.
    let bind_addr: SocketAddr = "0.0.0.0:67".parse()?;
    let socket = UdpSocket::bind(bind_addr).await?;
    socket.set_broadcast(true)?;

    println!("DHCP server listening on {}", bind_addr);

    let db = Arc::new(Mutex::new(LeaseDb::new()));
    let mut buf = vec![0u8; 1500];

    loop {
        let (len, peer) = socket.recv_from(&mut buf).await?;

        let msg = match Message::decode(&mut Decoder::new(&buf[..len])) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to decode DHCP packet from {}: {}", peer, e);
                continue;
            }
        };

        // Only handle boot requests (client → server)
        if msg.opcode() != Opcode::BootRequest {
            continue;
        }

        let msg_type = match msg
            .opts()
            .get(dhcproto::v4::OptionCode::MessageType)
        {
            Some(DhcpOption::MessageType(t)) => *t,
            _ => continue,
        };

        let mac = msg.chaddr().to_vec();
        let db_clone = Arc::clone(&db);
        let socket_ref = &socket;

        match msg_type {
            MessageType::Discover => {
                let mut db = db_clone.lock().await;
                if let Some(offered_ip) = db.assign(&mac) {
                    let reply = build_reply(&msg, offered_ip, MessageType::Offer);
                    send_reply(socket_ref, &reply).await;
                    println!("OFFER {:?} → {}", mac, offered_ip);
                } else {
                    eprintln!("DHCP pool exhausted, cannot offer to {:?}", mac);
                }
            }

            MessageType::Request => {
                let mut db = db_clone.lock().await;
                if let Some(ip) = db.assign(&mac) {
                    let reply = build_reply(&msg, ip, MessageType::Ack);
                    send_reply(socket_ref, &reply).await;
                    println!("ACK  {:?} → {}", mac, ip);
                } else {
                    // Send NAK
                    let reply = build_nak(&msg);
                    send_reply(socket_ref, &reply).await;
                    println!("NAK  {:?}", mac);
                }
            }

            MessageType::Release => {
                let mut db = db_clone.lock().await;
                db.release(&mac);
                println!("RELEASE {:?}", mac);
            }

            MessageType::Inform => {
                // Client already has IP, just wants options — ACK with no yiaddr
                let reply = build_reply(&msg, msg.ciaddr(), MessageType::Ack);
                send_reply(socket_ref, &reply).await;
                println!("INFORM ACK {:?}", mac);
            }

            other => {
                eprintln!("Unhandled DHCP message type: {:?}", other);
            }
        }
    }
}

// ── Message builders ──────────────────────────────────────────────────────────

fn build_reply(req: &Message, offered_ip: Ipv4Addr, msg_type: MessageType) -> Vec<u8> {
    let mut reply = Message::default();

    reply
        .set_opcode(Opcode::BootReply)
        .set_htype(req.htype())
        .set_xid(req.xid())           // must echo client's transaction ID
        .set_flags(Flags::default())
        .set_yiaddr(offered_ip)        // "your" IP
        .set_siaddr(SERVER_IP)         // next server (us)
        .set_giaddr(req.giaddr())      // relay agent (echo back)
        .set_chaddr(req.chaddr());     // echo client MAC

    reply
        .opts_mut()
        .insert(DhcpOption::MessageType(msg_type));

    reply
        .opts_mut()
        .insert(DhcpOption::ServerIdentifier(SERVER_IP));

    reply
        .opts_mut()
        .insert(DhcpOption::AddressLeaseTime(LEASE_TIME_SECS));

    reply
        .opts_mut()
        .insert(DhcpOption::SubnetMask(SUBNET_MASK));

    reply
        .opts_mut()
        .insert(DhcpOption::Router(vec![GATEWAY]));

    // DNS: point at ourselves; swap for real DNS if needed
    reply
        .opts_mut()
        .insert(DhcpOption::DomainNameServer(vec![SERVER_IP]));

    encode_message(reply)
}

fn build_nak(req: &Message) -> Vec<u8> {
    let mut reply = Message::default();

    reply
        .set_opcode(Opcode::BootReply)
        .set_htype(req.htype())
        .set_xid(req.xid())
        .set_flags(Flags::default())
        .set_chaddr(req.chaddr());

    reply
        .opts_mut()
        .insert(DhcpOption::MessageType(MessageType::Nak));

    reply
        .opts_mut()
        .insert(DhcpOption::ServerIdentifier(SERVER_IP));

    encode_message(reply)
}

fn encode_message(msg: Message) -> Vec<u8> {
    let mut buf = Vec::new();
    let mut encoder = Encoder::new(&mut buf);
    msg.encode(&mut encoder).expect("Failed to encode DHCP message");
    buf
}

async fn send_reply(socket: &UdpSocket, payload: &[u8]) {
    // Broadcast the reply — clients may not have an IP yet
    let broadcast: SocketAddr = "255.255.255.255:68".parse().unwrap();
    if let Err(e) = socket.send_to(payload, broadcast).await {
        eprintln!("Failed to send DHCP reply: {}", e);
    }
}