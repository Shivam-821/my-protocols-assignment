```
application-layer-protocols/
├── README.md                  # explain split, how to run each part, why Rust vs JS
├── rust-part/                 # HTTPS server + DNS + DHCP
│   ├── Cargo.toml
│   ├── Cargo.lock
│   └── src/
│       ├── main.rs            # can launch all or use subcommands
│       ├── https_server.rs
│       ├── dns.rs
│       └── dhcp.rs
└── js-part/                   # FTP server + SMTP (client or simple server)
      ├── package.json
      ├── server.js              # main entry, or separate files
      ├── ftp-server.js
      └── smtp.js
```
