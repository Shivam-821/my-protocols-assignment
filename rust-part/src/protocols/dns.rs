use anyhow::{Context, Result};
use hickory_proto::rr::{
    rdata::{A, NS, SOA},
    LowerName, Name, Record, RData,
};
use hickory_server::authority::{AuthorityObject, Catalog, ZoneType};
use hickory_server::server::ServerFuture;
use hickory_server::store::in_memory::InMemoryAuthority;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, UdpSocket};

pub async fn run_dns_server() -> Result<()> {
    let mut catalog = Catalog::new();

    let origin = Name::from_str("example.local.")?;
    let serial = 2025022701u32; // ← change this when you update records

    let mut authority = InMemoryAuthority::empty(origin.clone(), ZoneType::Primary, false);

    // SOA - mandatory
    authority
        .upsert(
            Record::from_rdata(
                origin.clone(),
                3600,
                RData::SOA(SOA::new(
                    Name::from_str("ns1.example.local.")?,
                    Name::from_str("admin.example.local.")?,
                    serial,
                    3600,   // refresh
                    600,    // retry
                    86400,  // expire
                    3600,   // minimum / negative cache
                )),
            ),
            serial,
        )
        .await;

    // NS record (zone apex)
    authority
        .upsert(
            Record::from_rdata(
                origin.clone(),
                3600,
                RData::NS(NS(Name::from_str("ns1.example.local.")?)),
            ),
            serial,
        )
        .await;

    // Glue: A record for the name server itself (very important!)
    authority
        .upsert(
            Record::from_rdata(
                Name::from_str("ns1.example.local.")?,
                3600,
                RData::A(A(Ipv4Addr::new(127, 0, 0, 1))),
            ),
            serial,
        )
        .await;

    // Your actual records
    authority
        .upsert(
            Record::from_rdata(
                Name::from_str("www.example.local.")?,
                300,
                RData::A(A(Ipv4Addr::new(192, 0, 2, 1))),
            ),
            serial,
        )
        .await;

    authority
        .upsert(
            Record::from_rdata(
                Name::from_str("mail.example.local.")?,
                300,
                RData::A(A(Ipv4Addr::new(192, 0, 2, 2))),
            ),
            serial,
        )
        .await;

    let lower_origin = LowerName::new(&origin);
    let auth: Arc<dyn AuthorityObject> = Arc::new(authority);

    catalog.upsert(lower_origin, vec![auth]);

    // Bind only to loopback — most reliable for local Windows testing
    let addr: SocketAddr = "127.0.0.1:5454".parse().context("invalid addr")?;

    let mut server = ServerFuture::new(catalog);

    // UDP
    let udp = UdpSocket::bind(addr).await.context("bind udp failed")?;
    server.register_socket(udp);

    // TCP (important for Windows nslookup in many cases)
    let tcp = TcpListener::bind(addr).await.context("bind tcp failed")?;
    server.register_listener(tcp, Duration::from_secs(10));

    println!("→ DNS listening on  udp://127.0.0.1:5454");
    println!("→ DNS listening on  tcp://127.0.0.1:5454");

    server
        .block_until_done()
        .await
        .context("DNS server future failed")?;

    Ok(())
}