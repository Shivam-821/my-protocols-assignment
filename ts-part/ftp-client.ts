import net from "net";

const PORT = 2121;
const HOST = "127.0.0.1";

console.log(`Attempting to connect to FTP Server at ${HOST}:${PORT}...`);

const client = net.createConnection({ port: PORT, host: HOST }, () => {
  console.log("Connected to server.\n");
});

const commands = ["USER admin", "PASS password", "SYST", "PWD", "QUIT"];

let currentCommandIndex = 0;

function sendNextCommand() {
  if (currentCommandIndex < commands.length) {
    const cmd = commands[currentCommandIndex];
    console.log(`[SENDING] ${cmd}`);

    // FTP Requires commands to end in \r\n (Carriage Return + Line Feed)
    client.write(`${cmd}\r\n`);
    currentCommandIndex++;
  }
}

// Listen for the server's response
client.on("data", (data) => {
  const message = data.toString().trim();
  console.log(`[RECEIVED] ${message}\n`);

  // FTP server codes indicate readyness.
  // - 220: Greeting (ready for novel user)
  // - 331: Need password
  // - 230: Login successful
  // - 215, 257: Information responses

  // Whenever the server finishes its response, we send our next command!
  // Note: Real clients would parse the 3-digit code strictly.
  sendNextCommand();
});

client.on("end", () => {
  console.log("\nDisconnected from server.");
});

client.on("error", (err) => {
  console.error("Client error:", err);
});
