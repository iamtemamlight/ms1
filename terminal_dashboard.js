// AllBright — HONEST live terminal dashboard.
// Displays ONLY real data from the running API (port 3000).
// It does NOT simulate, auto-increment, or fabricate any trade/profit/latency.
// If nothing is trading, it shows 0 / — (em-dash). That is the truth.
//
// Usage:  node terminal_dashboard.js          (interactive TUI in a real terminal)
//         API_BASE=http://host:port node terminal_dashboard.js

const BASE = process.env.API_BASE || 'http://localhost:3000';
const E = {
  profit: '/api/profit/metrics',
  telemetry: '/api/arbitrage/telemetry',
  fleet: '/api/fleet/status',
};

async function get(path) {
  try {
    const r = await fetch(BASE + path);
    if (!r.ok) return { _error: `HTTP ${r.status}` };
    return await r.json();
  } catch (e) {
    return { _error: e.message };
  }
}

const f = (n, d = 6) =>
  (n === null || n === undefined || Number.isNaN(Number(n))) ? '—' : Number(n).toFixed(d);

function render(p, t, fl) {
  const S = '═'.repeat(72);
  const L = [];
  L.push(S);
  L.push(` ALLBRIGHT LIVE TRADE DASHBOARD        ${new Date().toISOString()}`);
  L.push(S);
  L.push(' PROFIT METRICS');
  if (p._error) L.push(`   unavailable: ${p._error}`);
  else {
    L.push(`   accumulated_profit : ${f(p.accumulated_profit)} ETH`);
    L.push(`   daily_profit       : ${f(p.daily_profit)} ETH`);
    L.push(`   trades_executed    : ${p.trades_executed}`);
    L.push(`   trades_detected    : ${p.trades_detected}`);
    L.push(`   execution_rate_pct : ${f(p.execution_rate_pct, 2)} %`);
    L.push(`   executions_failed  : ${p.executions_failed}`);
    L.push(`   trades_per_hour    : ${f(p.trades_per_hour, 1)}`);
    L.push(`   avg_profit/trade   : ${f(p.profit_per_trade_eth, 8)} ETH  ($${f(p.profit_per_trade_usd, 2)})`);
    L.push(`   simulated          : ${p.simulated}`);
  }
  L.push(S);
  L.push(' ARBITRAGE / LATENCY');
  if (t._error) L.push(`   unavailable: ${t._error}`);
  else {
    L.push(`   mempool_scan_ms    : ${f(t.mempool_scan_ms, 2)}`);
    L.push(`   rpc_latency_ms     : ${f(t.rpc_latency_ms, 2)}`);
    L.push(`   bundle_submit_ms   : ${f(t.bundle_submission_ms, 2)}`);
    L.push(`   block_confirm_ms   : ${f(t.block_confirmation_ms, 1)}`);
    L.push(`   end_to_end_ms      : ${f(t.end_to_end_ms, 2)}`);
    L.push(`   spread_pct         : ${f(t.spread_pct, 4)}`);
    L.push(`   net_profit_eth     : ${f(t.net_profit_eth, 8)}`);
    L.push(`   status             : ${t.status}`);
    if (t.rpc_error) L.push(`   rpc_error          : ${t.rpc_error}`);
    L.push(`   simulated          : ${t.simulated}`);
    if (Array.isArray(t.opportunities) && t.opportunities.length) {
      L.push('   OPPORTUNITIES:');
      for (const o of t.opportunities) {
        L.push(`     ${o.pair}  ${o.buy_dex} -> ${o.sell_dex}  spread ${o.spread_pct}%  (${f(o.price_buy, 2)} -> ${f(o.price_sell, 2)})`);
      }
    }
  }
  L.push(S);
  L.push(' FLEET');
  if (fl._error) L.push(`   unavailable: ${fl._error}`);
  else {
    L.push(`   active_runners     : ${fl.active_runners}`);
    L.push(`   aggregate_yield    : ${f(fl.aggregate_yield_eth)} ETH`);
    L.push(`   alert_level        : ${fl.alert_level}`);
    L.push(`   simulated          : ${fl.simulated}`);
  }
  L.push(S);
  L.push(' Shows ONLY real recorded data. 0 / — means no live trades have occurred yet.');
  L.push('');
  return L.join('\n');
}

async function tick() {
  const [p, t, fl] = await Promise.all([get(E.profit), get(E.telemetry), get(E.fleet)]);
  const out = render(p, t, fl);
  process.stdout.write(process.stdout.isTTY ? `\x1b[2J\x1b[H${out}` : out);
}

tick();
setInterval(tick, 3000);
