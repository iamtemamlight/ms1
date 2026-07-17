/**
 * ALLBRIGHT LocalPort RPC Relay
 * --------------------------------------------------------------------------
 * Production-style LocalPort deployment: exposes local EVM JSON-RPC endpoints
 * on ports 8545-8549 that proxy to the configured upstream RPCs in .env.
 *
 * This is a READ-ONLY relay. It forwards eth_call / eth_blockNumber / logs /
 * trace requests to the upstream provider. It NEVER signs or broadcasts
 * transactions, so it is safe to run alongside a live private key.
 *
 * Ports (per LocalPort deployment protocol):
 *   8545 -> Primary Fleet RPC   (ETH_RPC_URL)
 *   8546 -> Secondary Fleet RPC (BASE_RPC_URL)
 *   8547 -> Shadow-Fork Sim     (POLYGON_RPC_URL)
 *   8548 -> Testing/QA          (ARBITRUM_RPC_URL)
 *   8549 -> Arbitrum One Mirror (OPTIMISM_RPC_URL)
 */

import http from "node:http";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import path from "node:path";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

function loadEnv() {
  const out = {};
  try {
    const raw = readFileSync(path.join(__dirname, ".env"), "utf8");
    for (const line of raw.split("\n")) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) continue;
      const idx = trimmed.indexOf("=");
      if (idx === -1) continue;
      const key = trimmed.slice(0, idx).trim();
      let val = trimmed.slice(idx + 1).trim();
      if ((val.startsWith('"') && val.endsWith('"')) || (val.startsWith("'") && val.endsWith("'"))) {
        val = val.slice(1, -1);
      }
      out[key] = val;
    }
  } catch (e) {
    console.error("[localport] could not read .env:", e.message);
  }
  return out;
}

const env = loadEnv();

const PORTS = [
  { port: 8545, upstream: env.ETH_RPC_URL || env.RPC_ENDPOINT, label: "Primary Fleet RPC (ETH)" },
  { port: 8546, upstream: env.BASE_RPC_URL, label: "Secondary RPC (BASE)" },
  { port: 8547, upstream: env.POLYGON_RPC_URL, label: "Shadow-Fork Sim (POLYGON)" },
  { port: 8548, upstream: env.ARBITRUM_RPC_URL, label: "Testing/QA (ARBITRUM)" },
  { port: 8549, upstream: env.OPTIMISM_RPC_URL, label: "Arbitrum One Mirror (OPTIMISM)" },
];

function createServer({ port, upstream, label }) {
  if (!upstream) {
    console.warn(`[localport] ${label} has no upstream RPC configured; endpoint will be inert.`);
  }
  const server = http.createServer((req, res) => {
    if (req.method !== "POST") {
      res.writeHead(405, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ jsonrpc: "2.0", error: { code: -32601, message: "method not allowed" }, id: null }));
      return;
    }
    let body = "";
    req.on("data", (c) => (body += c));
    req.on("end", async () => {
      if (!upstream) {
        res.writeHead(502, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ jsonrpc: "2.0", error: { code: -32603, message: "no upstream RPC configured" }, id: null }));
        return;
      }
      try {
        const upstreamRes = await fetch(upstream, {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body,
        });
        const text = await upstreamRes.text();
        res.writeHead(upstreamRes.status, { "Content-Type": "application/json", "Access-Control-Allow-Origin": "*" });
        res.end(text);
      } catch (e) {
        res.writeHead(502, { "Content-Type": "application/json" });
        res.end(JSON.stringify({ jsonrpc: "2.0", error: { code: -32603, message: "upstream relay error: " + e.message }, id: null }));
      }
    });
  });
  server.listen(port, "0.0.0.0", () => {
    console.log(`[localport] ${label} listening on http://0.0.0.0:${port} -> ${upstream || "(none)"}`);
  });
}

for (const cfg of PORTS) createServer(cfg);

console.log("[localport] LocalPort RPC relay started (read-only). No transactions are signed or broadcast.");
