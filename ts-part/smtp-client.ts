import net from "net";

const PORT = 2525;
const HOST = "127.0.0.1";

console.log(`Attempting to connect to SMTP Server at ${HOST}:${PORT}...`);

const client = net.createConnection({ port: PORT, host: HOST }, () => {
  console.log("Connected to server.\n");
});

const commands = [
  "EHLO localhost",
  "MAIL FROM:<shivam@example.com>",
  "RCPT TO:<abhinav@example.com>",
  "DATA",
  "Subject: Test Message\r\n\r\nThis is a test email sent from our simple TS SMTP client.\r\n.",
  "QUIT",
];

let currentCommandIndex = 0;

function sendNextCommand() {
  if (currentCommandIndex < commands.length) {
    const cmd = commands[currentCommandIndex];
    console.log(`[SENDING] ${cmd}`);

    // SMTP Requires commands to end in \r\n (Carriage Return + Line Feed)
    client.write(`${cmd}\r\n`);
    currentCommandIndex++;
  }
}

// Listen for the server's response
client.on("data", (data) => {
  const message = data.toString().trim();
  console.log(`[RECEIVED] ${message}\n`);

  // Simple state tracking logic
  // When '354' is received (DATA), we send the body
  if (message.startsWith("354")) {
    sendNextCommand();
  } else if (
    message.startsWith("220") ||
    message.startsWith("250") ||
    message.startsWith("221")
  ) {
    sendNextCommand();
  } else if (message.startsWith("5")) {
    console.error("Error from server. Aborting.");
    client.end();
  }
});

client.on("end", () => {
  console.log("\nDisconnected from server.");
});

client.on("error", (err) => {
  console.error("Client error:", err);
});
