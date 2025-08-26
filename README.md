# ocpp-rs

*A modern, production-grade OCPP implementation in Rust. Batteries included for CSMS (server), Charge Point simulator (client), conformance tests, and observability.*

---

## Why this project
- **Reliability & scale**: WebSockets for thousands of chargers per node with predictable tail latencies (Tokio).  
- **Safety**: Memory-safe, fearless concurrency for 24/7 operations.  
- **Portability**: Small static binaries, easy on-prem and edge deployments.  

Target protocols: **OCPP 1.6J** first, then **OCPP 2.0.1** modules incrementally.

---

## Scope & Non-Goals

### In scope
- OCPP **WebSocket JSON** framing (CALL/CALLRESULT/CALLERROR)  
- Minimal **CSMS** (server) handling core 1.6J actions  
- **Charge Point simulator** (client) for local/integration tests  
- Persistence (Postgres) with idempotency & audit  
- Observability (metrics, tracing) & operational tooling  
- Transport hardening: backpressure, queue bounds, graceful drain  

### Out of scope (for this repo)
- End-user apps, billing, pricing, vouchers, e-invoicing  
- OCPI roaming (separate project)  
- Vendor-specific charger drivers/diagnostics beyond standard OCPP  
