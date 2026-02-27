use anyhow::Result;
use hickory_proto::rr::{rdata::A, rdata::NS, LowerName, Name, Record, RData};
use hickory_server::authority::{AuthorityObject, Catalog, ZoneType};
use hickory_server::server::ServerFuture;
use hickory_server::store::in_memory::InMemoryAuthority;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::{TcpListener, UdpSocket};

pub async fn run_dns_server() -> Result<()> {
    let mut catalog = Catalog::new();
    let zone_name = Name::from_str("example.local.")?;
    let serial = 1u32;

    let mut authority = InMemoryAuthority::empty(zone_name.clone(), ZoneType::Primary, false);

    // Add SOA record
    authority.upsert(Record::from_rdata(
        zone_name.clone(), 3600,
        RData::SOA(hickory_proto::rr::rdata::SOA::new(
            Name::from_str("ns1.example.local.")?,
            Name::from_str("admin.example.local.")?,
            serial, 3600, 600, 86400, 3600,
        )),
    ), serial).await;

    // Add NS record
    authority.upsert(Record::from_rdata(
        zone_name.clone(), 3600,
        RData::NS(hickory_proto::rr::rdata::NS(Name::from_str("ns1.example.local.")?)),
    ), serial).await;

    // Add A records
    authority.upsert(Record::from_rdata(
        Name::from_str("www.example.local.")?, 3600,
        RData::A(A(std::net::Ipv4Addr::new(192, 0, 2, 1))),
    ), serial).await;

    let lower_zone_name = LowerName::new(&zone_name);
    let authority_obj: Arc<dyn AuthorityObject> = Arc::new(authority);
    catalog.upsert(lower_zone_name, vec![authority_obj]);

    // Use 0.0.0.0 to ensure it's reachable from all local interface paths
    let addr: SocketAddr = "0.0.0.0:5454".parse()?;
    let mut server = ServerFuture::new(catalog);

    let udp_socket = UdpSocket::bind(addr).await?;
    server.register_socket(udp_socket);

    // Windows nslookup often requires TCP fallback support
    let tcp_listener = TcpListener::bind(addr).await?;
    server.register_listener(tcp_listener, std::time::Duration::from_secs(5));

    println!("DNS server listening on udp/tcp://0.0.0.0:5454");
    server.block_until_done().await?;

    Ok(())
}