1. Axum for handling routing
2. Tokio for Axum dependency and Async handling
3. axum-server and rustls-pemfile for loading PEM files, handling https
4. Downloaded mkcert for certificate generation, and https
5. mkcert -install and mkcert localhost to create certificates valid for those endpoints
6. The DNS server is forwarding every query to 8.8.8.8 which is the standard google's public DNS
7. Now 
   - https: 3000
   - DNS: 3001
8. dhcproto library for parsing and generating DHCP (Dynamic Host Configuration Protocol) packets.
9. The DORA flow is fully implemented — Discover → Offer → Request → Ack. Release and Inform are also handled.
10. 

