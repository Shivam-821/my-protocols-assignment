import net from "net";

const PORT = 2121;
const HOST = "127.0.0.1";

interface ClientState {
  isAuthenticated: boolean;
  username: string | null;
}

const server = net.createServer((socket) => {
  console.log(
    `[+] Client connected: ${socket.remoteAddress}:${socket.remotePort}`,
  );

  const state: ClientState = {
    isAuthenticated: false,
    username: null,
  };

  socket.write("220 Welcome to the Simple TS FTP Server\r\n");

  socket.on("data", (data) => {
    const message = data.toString().trim();
    if (!message) return;

    console.log(`[CLIENT] ${message}`);

    const parts = message.split(" ");
    const command = (parts[0] ?? "").toUpperCase();
    const args = parts.slice(1).join(" ");
    switch (command) {
      case "USER":
        if (!args) {
          socket.write("501 Syntax error in parameters or arguments.\r\n");
          break;
        }
        state.username = args;
        socket.write(`331 User ${args} okay, need password.\r\n`);
        break;

      case "PASS":
        if (!state.username) {
          socket.write("503 Bad sequence of commands (send USER first).\r\n");
          break;
        }
        if (
          state.username === "anonymous" ||
          (state.username === "admin" && args === "password")
        ) {
          state.isAuthenticated = true;
          socket.write("230 User logged in, proceed.\r\n");
        } else {
          socket.write("530 Login incorrect.\r\n");
          state.username = null;
        }
        break;

      case "SYST":
        socket.write("215 UNIX Type: L8\r\n");
        break;
        if (!state.isAuthenticated) {
          socket.write("530 Please login with USER and PASS.\r\n");
          break;
        }
        // 257 means "PATHNAME created" or in this case, returned.
        socket.write('257 "/" is the current directory\r\n');
        break;

      case "QUIT":
        // Client wants to disconnect
        socket.write("221 Service closing control connection.\r\n");
        socket.end();
        break;

      default:
        // 502 means "Command not implemented."
        socket.write("502 Command not implemented.\r\n");
        break;
    }
  });

  socket.on("end", () => {
    console.log(
      `[-] Client disconnected: ${socket.remoteAddress}:${socket.remotePort}`,
    );
  });

  socket.on("error", (err) => {
    console.error(`[!] Socket error:`, err);
  });
});

server.listen(PORT, HOST, () => {
  console.log(`[SERVER] FTP Control Server listening on ${HOST}:${PORT}`);
  console.log(`[SERVER] Run 'bun run ftp-client.ts' to test the connection.\n`);
});
