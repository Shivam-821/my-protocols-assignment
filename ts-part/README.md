# ts-part: Application Layer Protocols (FTP & SMTP)

This part of the project is dedicated to implementing the File Transfer Protocol (FTP) and Simple Mail Transfer Protocol (SMTP) from scratch using TypeScript and Node.js core modules.

## What You Need to Do

Your goal is to build servers (or clients) for Application Layer Protocols without relying on third-party libraries that do the heavy lifting (like `ftp-srv` or `nodemailer`).

Specifically, you need to:

1. **Understand exactly how FTP and SMTP work.** You have to read their RFCs (like RFC 959 for FTP) to understand the commands (`USER`, `PASS`, `PORT`, `RETR`, `STOR`) and response codes (`220`, `331`, `230`).
2. **Handle raw TCP Connections.** You will use the `net` module to listen for TCP connections from actual FTP clients (like FileZilla or the command line `ftp` tool).
3. **Parse Commands and Manage State.** Because TCP transmits a continuous stream of data, you'll need to buffer the incoming text, split it by newlines (`\r\n`), and parse the FTP or SMTP commands. You also need to keep track of state for each connection (e.g., is the user authenticated yet?).
4. **Implement Data Connections (FTP Specific).** FTP is unique because it uses _two_ connections:
   - A **Control Connection** (usually port 21) where commands and replies are sent.
   - A **Data Connection** (often a random port) where the actual file contents (or directory listings) are transferred. You will need to manage opening this second connection when the client requests a file.

## 1. What is the `net` module?

The `net` module is a core module built directly into Node.js (and Bun/Deno).
While you are probably used to the `http` module (or frameworks like Express/Hono) which handle HTTP requests (`GET /path`), **HTTP is an Application Layer protocol that runs _on top_ of TCP.**

The `net` module gives you access to raw **Transport Layer (TCP)** streams.
It allows you to:

- Create a server that listens on a port (e.g., `net.createServer()`).
- Establish a continuous, bidirectional stream of data (a `Socket`) with a client.
- Send and receive raw bytes or strings over that socket (`socket.write()`, `socket.on('data')`).

Since FTP and SMTP are also Application Layer protocols (just like HTTP), they also run directly on top of TCP! That's why we use the `net` module to implement them.

## 2. What is FTP (File Transfer Protocol)?

FTP is one of the oldest protocols on the internet (predating TCP/IP itself!). It was designed as a standard way to upload, download, and manage files on a remote server.

### The Core Concept

FTP works on a **Client-Server model** and operates over **TCP**. As mentioned earlier, its defining characteristic is its dual-connection design:

1. **Control Connection (Port 21):** The client connects here to send commands (like "log me in", "change directory to X", "send me file Y") and the server responds with status codes (like "331 Password required", "250 Directory changed", "226 Transfer complete").
2. **Data Connection:** When a file needs to be transferred or a directory needs to be listed (which is just sending a list of files as text), a temporary second connection is opened. The file data is blasted across this connection, and then the connection is closed.

### Where is it used?

- **Legacy Systems and Backups:** Many enterprise systems, mainframes, and automated scripts still use FTP to dump backup files or logs onto centralized servers at night.
- **Web Hosting (Historically):** It used to be the primary way web developers uploaded HTML and PHP files to their shared hosting providers (though SFTP/SSH is much more common now due to security).
- **Large Public Archives:** Organizations that host massive amounts of public data (like Linux ISO images, genomic data, or historical weather data) often operate public, anonymous FTP servers.

### Why does it output multiple "Client connected" logs?

If you access your raw `net` server (`localhost:2121`) via a modern Web Browser (by typing `http://localhost:2121`), the browser expects an HTTP server. It will open a TCP connection and send an HTTP `GET` request. Since your server responds immediately and closes the socket, the browser might automatically open several more connections to ask for things like `favicon.ico` or to retry the original request.

To properly test an FTP server, you should connect using a real FTP client. Open your terminal and run:

```bash
ftp localhost 2121
# or
nc localhost 2121
```
