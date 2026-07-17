@echo off
cd /d D:\ALLBRIGHT\backend
set DATABASE_URL=postgresql://neondb_owner:****@ep-plain-math-a4m60ed2-pooler.us-east-1.aws.neon.tech/neondb?sslmode=require
set GROQ_API_KEY=<set-in-environment>
set OPENROUTER_API_KEY=<set-in-environment>
set RPC_ENDPOINT=https://eth.llamarpc.com
set ETH_RPC_URL=https://lb.drpc.live/ethereum/<set-in-environment>
set HTTP_BIND_ADDR=0.0.0.0:3000
set C2_BIND_ADDR=0.0.0.0:50051
set RUST_LOG=info
set RUST_BACKTRACE=1
set RUSTLS_PROVIDER=ring
start /b allbright-c2-backend.exe
