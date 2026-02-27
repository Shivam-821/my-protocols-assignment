use anyhow::Result;
use hickory_proto::rr::{rdata::A, LowerName, Name, Record, RData};
use hickory_server::authority::{AuthorityObject, Catalog, ZoneType};
use hickory_server::server::ServerFuture;
use hickory_server::store::in_memory::InMemoryAuthority;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

pub async fn run_dns_server() -> Result<()> {
    let mut catalog = Catalog::new();

    let zone_name = Name::from_str("example.local.")?;

    let mut authority = InMemoryAuthority::empty(
        zone_name.clone(),
        ZoneType::Primary,
        false, // no DNSSEC
    );

    let serial = 1u32;

    // SOA record — wrap in RData enum to get Record<RData>
    let soa_rdata = RData::SOA(hickory_proto::rr::rdata::SOA::new(
        Name::from_str("ns1.example.local.")?,
        Name::from_str("admin.example.local.")?,
        serial,
        3600,  // refresh
        600,   // retry
        86400, // expire
        3600,  // min TTL
    ));
    let soa_record: Record<RData> = Record::from_rdata(zone_name.clone(), 3600, soa_rdata);
    authority.upsert(soa_record, serial).await;

    // A record for www.example.local.
    let www_record: Record<RData> = Record::from_rdata(
        Name::from_str("www.example.local.")?,
        3600,
        RData::A(A(std::net::Ipv4Addr::new(192, 0, 2, 1))),
    );
    authority.upsert(www_record, serial).await;

    // A record for mail.example.local.
    let mail_record: Record<RData> = Record::from_rdata(
        Name::from_str("mail.example.local.")?,
        3600,
        RData::A(A(std::net::Ipv4Addr::new(192, 0, 2, 2))),
    );
    authority.upsert(mail_record, serial).await;

    // catalog.upsert expects Vec<Arc<dyn AuthorityObject>>
    let lower_zone_name = LowerName::new(&zone_name);
    let authority_obj: Arc<dyn AuthorityObject> = Arc::new(authority);
    catalog.upsert(lower_zone_name, vec![authority_obj]);

    let addr: SocketAddr = "127.0.0.1:3001".parse()?;
    let mut server = ServerFuture::new(catalog);

    println!("DNS server listening on udp://{}", addr);

    let udp_socket = tokio::net::UdpSocket::bind(addr).await?;
    server.register_socket(udp_socket);

    server.block_until_done().await?;

    Ok(())
}