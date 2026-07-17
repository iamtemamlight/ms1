# AllBright Arbitrage Flash Loan App - Port Mapping Configuration

## Service Port Assignments

### Frontend Services
| Service | Main Port | Backup 1 | Backup 2 | Description |
|---------|-----------|----------|----------|-------------|
| Vite Dev Server (Dashboard) | **3002** | 3003 | 5173 | React dev server with HMR |
| Nginx Production (SPA) | **5200** | 8080 | 8443 | Production HTTPS reverse proxy |

### Backend API Services
| Service | Main Port | Backup 1 | Backup 2 | Description |
|---------|-----------|----------|----------|-------------|
| Express API Backend | **3001** | 3004 | 3005 | Main REST API + WebSocket |
| WebSocket Server (wss) | **50051** | 50052 | 50053 | Real-time telemetry (gRPC/C2) |
| Rust Engine HTTP | **3001** | 3004 | 3005 | Production KPI backend |

### Infrastructure Services
| Service | Main Port | Backup 1 | Backup 2 | Description |
|---------|-----------|----------|----------|-------------|
| PostgreSQL (Neon/Cloud) | **5432** | 5433 | 5434 | Primary database |
| Redis Cache | **6379** | 6380 | 6381 | Session/rate limiting cache |
| Prometheus Metrics | **9090** | 9091 | 9092 | Monitoring & alerting |
| Grafana Dashboards | **3006** | 3007 | 3008 | Visualization |
| Loki Log Aggregation | **3100** | 3101 | 3102 | Log storage |
| Alertmanager | **9093** | 9094 | 9095 | Alert routing |

### Kubernetes / Container Ports
| Service | Main Port | Backup 1 | Backup 2 | Description |
|---------|-----------|----------|----------|-------------|
| K8s API Server | **6443** | - | - | Cluster control plane |
| Kubelet | **10250** | - | - | Node agent |
| Container Registry | **5000** | 5001 | 5002 | Image registry |
| LocalPort RPC | **8545** | 8550 | 8551 | Multi-chain EVM RPC |

---

## Environment Variable References

### Dashboard (.env.production)
```env
VITE_API_BASE=http://localhost:3001  # Direct to Express API + WebSocket
VITE_ENGINE_MODE=production
VITE_DEMO_MODE=false
```

### Backend (start_backend.bat / .env)
```bat
set HTTP_BIND_ADDR=0.0.0.0:3001      # Express API + WebSocket
set C2_BIND_ADDR=0.0.0.0:50051       # gRPC/C2 control plane
set RUST_LOG=info
```

### Nginx Production (nginx.conf)
```nginx
server {
    listen 5200 ssl;
    # Proxies /api/ -> http://127.0.0.1:3001
    # Proxies /ws/ -> http://127.0.0.1:50051
}
```

---

## Port Conflict Resolution

| Conflict | Resolution |
|----------|------------|
| Grafana (3000) vs Express (3001) | Grafana moved to **3006**, **3007**, **3008** |
| Vite (3002) vs Grafana backup (3007) | Vite backup: **3003**, **5173** |
| Rust Engine (3001) vs Express (3001) | Same process in production - no conflict |
| Nginx :8080 vs Grafana :3007 | Nginx backup: **8080**, **8443** |

---

## Verification Checklist

### Development Mode
- [ ] Vite dev server running on :3002
- [ ] Express backend running on :3001
- [ ] WebSocket server running on :50051
- [ ] Dashboard accessible at http://localhost:3002
- [ ] API calls proxied: http://localhost:3002/api/* → :3001

### Production Mode
- [ ] Nginx running on :5200 (HTTPS)
- [ ] Rust engine running on :3001
- [ ] WebSocket on :50051
- [ ] PostgreSQL accessible (Neon cloud)
- [ ] Redis accessible
- [ ] Prometheus on :9090
- [ ] Grafana on :3006 (avoid :3000 conflict)

---

## Quick Port Verification Commands

```bash
# Check all listening ports
netstat -tulpn | grep LISTEN

# Verify specific services
curl http://localhost:3001/api/health-check
curl http://localhost:3002/
curl http://localhost:5200/api/health-check

# WebSocket test
wscat -c ws://localhost:50051/ws

# Database
psql $DATABASE_URL -c "SELECT 1"

# Redis
redis-cli ping
```