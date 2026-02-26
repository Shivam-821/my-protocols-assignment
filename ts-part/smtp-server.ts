import net from "net";

const PORT = 2525;
const HOST = "127.0.0.1";

interface ClientState {
  state: "INIT" | "GREETED" | "MAIL_FROM" | "RCPT_TO" | "DATA";
  from: string | null;
  to: string[];
  data: string;
}

const server = net.createServer((socket) => {
  console.log(
    `[+] Client connected: ${socket.remoteAddress}:${socket.remotePort}`,
  );

  const state: ClientState = {
    state: "INIT",
    from: null,
    to: [],
    data: "",
  };

  // Send greeting
  socket.write("220 Welcome to the Simple TS SMTP Server\r\n");

  socket.on("data", (data) => {
    const message = data.toString();

    // In DATA mode, we collect everything until a `.\r\n`
    if (state.state === "DATA") {
      state.data += message;
      if (state.data.endsWith("\r\n.\r\n")) {
        console.log(
          `[MAIL RECEIVED]\nFrom: ${state.from}\nTo: ${state.to.join(
            ", ",
          )}\n\n${state.data.slice(0, -5)}`,
        );
        state.state = "GREETED"; // Reset state for potentially more mail
        state.from = null;
        state.to = [];
        state.data = "";
        socket.write("250 OK: Message accepted for delivery\r\n");
      }
      return;
    }

    const trimmedMessage = message.trim();
    if (!trimmedMessage) return;

    console.log(`[CLIENT] ${trimmedMessage}`);

    const parts = trimmedMessage.split(" ");
    const command = (parts[0] ?? "").toUpperCase();
    const args = parts.slice(1).join(" ");

    switch (command) {
      case "HELO":
      case "EHLO":
        state.state = "GREETED";
        socket.write(`250 Hello ${args || "Client"}, pleased to meet you\r\n`);
        break;

      case "MAIL":
        if (state.state !== "GREETED") {
          socket.write("503 Bad sequence of commands\r\n");
          break;
        }
        if (args.toUpperCase().startsWith("FROM:")) {
          state.from = args.slice(5).trim();
          state.state = "MAIL_FROM";
          socket.write("250 OK\r\n");
        } else {
          socket.write("501 Syntax error in parameters or arguments\r\n");
        }
        break;

      case "RCPT":
        if (state.state !== "MAIL_FROM" && state.state !== "RCPT_TO") {
          socket.write("503 Bad sequence of commands\r\n");
          break;
        }
        if (args.toUpperCase().startsWith("TO:")) {
          state.to.push(args.slice(3).trim());
          state.state = "RCPT_TO";
          socket.write("250 OK\r\n");
        } else {
          socket.write("501 Syntax error in parameters or arguments\r\n");
        }
        break;

      case "DATA":
        if (state.state !== "RCPT_TO") {
          socket.write("503 Bad sequence of commands\r\n");
          break;
        }
        state.state = "DATA";
        socket.write("354 End data with <CR><LF>.<CR><LF>\r\n");
        break;

      case "QUIT":
        socket.write("221 Bye\r\n");
        socket.end();
        break;

      default:
        socket.write("502 Command not implemented\r\n");
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
  console.log(`[SERVER] SMTP Server listening on ${HOST}:${PORT}`);
  console.log(
    `[SERVER] Run 'bun run smtp-client.ts' to test the connection.\n`,
  );
});
