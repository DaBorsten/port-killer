#!/usr/bin/env bun
import { serve } from "bun";

const port = process.argv[2] ? parseInt(process.argv[2]) : 3000;
const name = process.argv[3] || "Server";

const server = serve({
  port,
  fetch(req) {
    return new Response(`${name} running on port ${port}`);
  },
});

console.log(`✅ ${name} läuft auf Port ${server.port} (PID: ${process.pid})`);
console.log(`   Besuche: ${server.url}`);
console.log(`   Drücke Ctrl+C zum Beenden\n`);

// Keep the process running
process.on("SIGINT", () => {
  console.log(`\n${name} wird beendet...`);
  process.exit(0);
});
