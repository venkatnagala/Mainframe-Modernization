# Mainframe Modernization: AI-Powered COBOL to Rust Pipeline

[![CI Status](https://github.com/venkatnagala/Mainframe-Modernization/actions/workflows/rust.yml/badge.svg)](https://github.com/venkatnagala/Mainframe-Modernization/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![Kubernetes](https://img.shields.io/badge/kubernetes-%23326ce5.svg?style=for-the-badge&logo=kubernetes&logoColor=white)](https://kubernetes.io/)
[![AWS](https://img.shields.io/badge/AWS-%23FF9900.svg?style=for-the-badge&logo=amazon-aws&logoColor=white)](https://aws.amazon.com/)

> An automated, AI-powered system that modernizes legacy mainframe COBOL applications to memory-safe Rust with automated validation, secured by a zero-trust Agent Gateway — deployed on Kubernetes.

---

## 🏆 Competition History

| Competition | Phase | Status |
|---|---|---|
| **AgentAheads Hackathon 2026** | Phase 1: AI-powered COBOL→Rust pipeline with Docker Compose | ✅ Completed |
| **SOLO AI Competition 2026** | Phase 2: Agent Gateway (JWT AuthN/AuthZ) + Kubernetes deployment | ✅ Submitted |

---

## 🎯 SOLO AI Competition 2026 Submission

**Category:** Secure & Govern MCP

### 📺 Demo Videos
- **Quick Demo (2 min):** https://www.youtube.com/watch?v=a7Yfz614d5Y
- **Detailed Walkthrough (9 min):** https://www.youtube.com/watch?v=5s6MMIfxNf0
- **Kubernetes Deployment Demo (3 min):** https://www.youtube.com/watch?v=05I-q2Ugw5Q

### 📝 Blog Post
https://dev.to/venkateshwar_raonagala_4/how-i-added-zero-trust-guardrails-to-4-mcp-servers-using-agentgateway-and-modernized-legacy-cobol-1fl8

---

## 🎯 Problem Statement

Enterprise mainframe applications written in COBOL face critical challenges:

- **Aging workforce**: COBOL programmers retiring faster than new ones learning
- **Maintenance costs**: Legacy systems expensive to maintain
- **Technical debt**: Decades-old codebases difficult to modify
- **AWS gap**: AWS Mainframe Modernization targets Java — no open-source, memory-safe Rust option exists

---

## 💡 Our Solution

A complete **AI-powered modernization pipeline** that:

1. ✅ Fetches legacy COBOL from AWS S3
2. ✅ Modernizes to idiomatic Rust using **Claude claude-opus-4-6**
3. ✅ **Validates correctness** by comparing outputs
4. ✅ Saves verified code back to S3 with secure access
5. ✅ **Secures all agent-to-MCP communication** via Agent Gateway (JWT + RBAC)
6. ✅ **Deployed on Kubernetes** with zero-trust network policies and HPA

---

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                     Kubernetes (mainframe-modernization)                    │
│                                                                             │
│  ┌──────────────┐     JWT/HTTPS      ┌─────────────────────────────────┐   │
│  │              │ ────────────────► │       Agent Gateway              │   │
│  │ Green Agent  │                   │   (AuthN + AuthZ + Audit)        │   │
│  │(Orchestrator)│ ◄──────────────── │   Port: 8090                    │   │
│  │  Port: 8080  │    Proxy Result   └──────────┬──────────────────────┘   │
│  └──────────────┘                              │ Authorized calls only      │
│                                                ▼                            │
│  ┌──────────────┐      ┌─────────────────────────────────────────────┐     │
│  │ Purple Agent │─────►│            MCP Servers                      │     │
│  │(AI Modernizer│      │  ┌──────┐  ┌──────────┐  ┌───────┐ ┌────┐  │     │
│  │  Port: 8081  │      │  │  S3  │  │AI Trans. │  │ COBOL │ │Rust│  │     │
│  └──────────────┘      │  │:8081 │  │  :8082   │  │ :8083 │ │:84 │  │     │
│                        └─────────────────────────────────────────────┘     │
│   NetworkPolicy: Default DENY ALL — whitelist only                          │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                               ┌──────┴──────┐
                               │   AWS S3    │
                               │  programs/  │
                               │  data/      │
                               │  modernized/│
                               └─────────────┘
```

---

## 🔐 Agent Gateway: Zero-Trust Security

The Agent Gateway is the **security spine** of the pipeline. No agent communicates directly with MCP servers — every call is authenticated (JWT) and authorized (RBAC) at the gateway.

### Authentication Flow

```
Agent                Agent Gateway              MCP Server
  │                       │                         │
  │── POST /auth/token ──►│                         │
  │   {agent_id, api_key} │                         │
  │                       │ Validates API key        │
  │◄── JWT token ─────────│                         │
  │                       │                         │
  │── POST /mcp/invoke ──►│                         │
  │   Bearer: JWT         │ Validates JWT            │
  │   {target, operation} │ Checks RBAC              │
  │                       │── Forward if allowed ──►│
  │                       │◄── MCP result ──────────│
  │◄── Proxied result ────│                         │
  │                       │ Audit log entry          │
```

### Role-Based Access Control (RBAC)

| Agent Role | S3 MCP | AI Translation MCP | COBOL MCP | Rust MCP |
|---|---|---|---|---|
| **Orchestrator** (Green Agent) | ✅ All ops | ✅ All ops | ✅ All ops | ✅ All ops |
| **Modernizer** (Purple Agent) | ❌ Blocked | ✅ Translate only | ❌ Blocked | ❌ Blocked |
| **ReadOnly** (Audit) | List only | ❌ | ❌ | ❌ |

> **AI Safety by Design**: Purple Agent is explicitly blocked from S3 write access even if compromised — blast radius is limited to translation operations only.

### Tested & Verified

```
✅ Health check:       GET  /health          → {status: healthy, mcps: 4}
✅ JWT issuance:       POST /auth/token      → Bearer token, role: orchestrator
✅ Authorized call:    POST /mcp/invoke      → authorized: true (Green → S3)
✅ Unauthorized call:  POST /mcp/invoke      → authorized: false (Purple → S3)
   "Role Modernizer is not authorized to call fetch_source on s3_mcp"
✅ Audit trail:        Every call logged with request_id and timestamp
```

---

## ✨ Key Features

### 🤖 AI-Powered Modernization
- **Claude claude-opus-4-6** for intelligent code translation
- Handles complex COBOL constructs (COMP-3 packed decimals, file I/O)
- Generates idiomatic, memory-safe Rust code

### ✅ Automated Validation
- Compiles both COBOL (GnuCOBOL) and generated Rust
- Executes with identical test data
- **Compares outputs** to ensure functional equivalence
- Only saves Rust code when outputs match ✓

### 🔒 Security & Best Practices
- **Agent Gateway**: JWT authentication + RBAC for all MCP server access
- **Zero-trust NetworkPolicy**: Default DENY ALL
- **Least-privilege IAM** policies (read-only source, write-only outputs)
- **Pre-signed URLs** for time-limited, secure file access (1-hour expiry)
- **No secrets in code** — environment variables only
- **JWT auto-refresh**: Green Agent automatically re-authenticates on token expiry

### ☸️ Kubernetes Deployment
- **7 services** deployed in `mainframe-modernization` namespace
- **HorizontalPodAutoscaler** on Purple Agent (1-5 replicas, CPU/memory based)
- **Zero-trust NetworkPolicy** — default DENY ALL between pods
- **Health endpoints** on all services for liveness/readiness probes
- **GitHub Actions CI/CD** — automatic rebuild and push to Docker Hub on every commit
- Single command deployment via `.\deploy.ps1`

### 🚀 Why Rust (not Java)?
- **Memory safe** — no garbage collector, no null pointer exceptions
- **Serverless ready** — sub-millisecond cold starts vs 2-5 seconds for Java
- **10× cheaper** on AWS Lambda vs Java (128MB vs 512MB+ memory)
- **True portability** — runs on any cloud provider

---

## 🎥 Demo

### Success Case: Interest Calculation

```
Input:  Loan Amount: $10,000.00, Rate: 5.5%
COBOL:  "CALCULATED INTEREST:     550.00"
Rust:   "CALCULATED INTEREST: 550.00"
Result: ✅ SUCCESS - Outputs match! Code saved to S3
```

---

## 🛠️ Tech Stack

| Component | Technology | Purpose |
|---|---|---|
| **AI Model** | Claude claude-opus-4-6 (Anthropic) | COBOL→Rust translation |
| **AI Translation MCP** | ai_mcp (Rust + Actix-web) | Calls Claude claude-opus-4-6 internally |
| **Agent Gateway** | Rust + Actix-web | JWT AuthN + RBAC AuthZ |
| **Backend** | Rust + Actix-web | Green Agent orchestration |
| **COBOL Compiler** | GnuCOBOL (free, open source) | Validate original code |
| **Storage** | AWS S3 | Source & output storage |
| **Security** | JWT + AWS IAM + Pre-signed URLs | Access control |
| **Deployment** | Docker + Kubernetes (Docker Desktop) | Container orchestration |
| **CI/CD** | GitHub Actions | Auto build + push to Docker Hub |
| **Languages** | Rust, COBOL, PowerShell | Implementation |

---

## 📋 Prerequisites

To run this pipeline you will need:

| Requirement | Cost | How to Get |
|---|---|---|
| **Docker Desktop** (with Kubernetes enabled) | Free | https://www.docker.com/products/docker-desktop |
| **AWS Account + S3** | Free tier available | https://console.aws.amazon.com |
| **Claude API Key** | $5 minimum credit | https://console.anthropic.com |
| **Git** | Free | https://git-scm.com |

> **Note:** Enable Kubernetes in Docker Desktop: Settings → Kubernetes → Enable Kubernetes → Apply & Restart.
> The Anthropic API key requires a minimum $5 credit to use Claude claude-opus-4-6.
> AWS Free Tier is sufficient for S3 storage used by this pipeline.

---

## 🚀 Quick Start

### Step 1 — Clone Repository
```bash
git clone https://github.com/venkatnagala/Mainframe-Modernization.git
cd Mainframe-Modernization
```

### Step 2 — Configure Environment
```bash
# Copy the example environment file
cp .env.example .env

# Edit .env and add your credentials:
# CLAUDE_API_KEY=your_key_here           (from console.anthropic.com - $5 minimum)
# AWS_ACCESS_KEY_ID=your_key_here        (from console.aws.amazon.com)
# AWS_SECRET_ACCESS_KEY=your_secret_here
# AWS_REGION=us-east-1
# S3_BUCKET_NAME=your_bucket_name_here
# JWT_SECRET=your-minimum-32-char-secret-here
# AGENT_API_KEY=your-agent-api-key-here
```

### Step 3 — Upload Sample COBOL to S3
```bash
# Upload the sample COBOL program to your S3 bucket
aws s3 cp legacy_source/interest_calc.cbl s3://YOUR_BUCKET/programs/interest_calc.cbl
aws s3 cp data/loan_data.json s3://YOUR_BUCKET/data/loan_data.json
```

### Step 4 — Run Everything (One Command!)
```powershell
.\deploy.ps1
```

This automatically:
- ✅ Verifies kubectl and cluster connection
- ✅ Loads environment variables from `.env`
- ✅ Creates Kubernetes namespace and RBAC
- ✅ Creates all Kubernetes secrets from `.env`
- ✅ Deploys Agent Gateway and waits for readiness
- ✅ Deploys all 4 MCP servers
- ✅ Deploys Green Agent and Purple Agent
- ✅ Applies zero-trust network policies
- ✅ Waits for all 7 pods to be ready
- ✅ Port-forwards Green Agent and runs demo automatically

Expected output:
```
🚀 Mainframe Modernization Pipeline - Kubernetes Deployment
============================================================
...
🎉 DEPLOYMENT COMPLETE!
============================================================

📊 Pod Status:
NAME                          READY   STATUS    RESTARTS
agent-gateway-xxx             1/1     Running   0
ai-mcp-xxx                    1/1     Running   0
cobol-mcp-xxx                 1/1     Running   0
green-agent-xxx               1/1     Running   0
purple-agent-xxx              1/1     Running   0
rust-mcp-xxx                  1/1     Running   0
s3-mcp-xxx                    1/1     Running   0

Sending Modernization Task to Green Agent...
Task accepted!
@{task_id=MODERN-DEMO-2026; status=SUCCESS - Outputs match! ✅; match_confirmed=True}
```

### Step 5 — Verify Zero-Trust Security
```powershell
# Test RBAC — Purple Agent blocked from S3
$purpleToken = (Invoke-RestMethod -Uri http://localhost:8090/auth/token `
  -Method POST -ContentType "application/json" `
  -Body '{"agent_id":"purple_agent","api_key":"purple-agent-dev-key-change-in-prod","requested_role":"modernizer"}'
).access_token

Invoke-RestMethod -Uri http://localhost:8090/mcp/invoke -Method POST `
  -Headers @{Authorization="Bearer $purpleToken"} `
  -ContentType "application/json" `
  -Body '{"target_mcp":"s3_mcp","operation":"fetch_source","payload":{}}'
```

Expected security response:
```json
{
  "authorized": false,
  "error": "Role Modernizer is not authorized to call fetch_source on s3_mcp",
  "audit_trail": {
    "agent_id": "purple_agent",
    "authorized": false,
    "request_id": "cf1d3191-4053-4b8e-b8a8-d4035023f92a"
  }
}
```

### Step 6 — Run Demo Again Manually (Optional)
```powershell
# Port-forward Green Agent
kubectl port-forward svc/green-agent 8080:8080 -n mainframe-modernization

# In a new terminal:
.\demo.ps1
```

---

## 📁 Project Structure

```
Mainframe-Modernization/
├── agent_gateway/            # Zero-trust security gateway
│   ├── src/main.rs          # JWT auth, RBAC, audit trail
│   ├── Cargo.toml
│   └── Dockerfile
├── green_agent/              # Orchestration service
│   ├── src/main.rs          # Routes calls via Agent Gateway + JWT auto-refresh
│   ├── Dockerfile
│   └── Cargo.toml
├── purple_agent/             # AI modernization service
│   ├── src/main.rs          # Claude claude-opus-4-6 integration
│   ├── Cargo.toml
│   └── Dockerfile
├── s3_mcp/                  # S3 storage MCP server
├── ai_mcp/                  # AI translation MCP server (calls Claude claude-opus-4-6)
├── cobol_mcp/               # COBOL compilation MCP server
├── rust_mcp/                # Rust compilation MCP server
├── k8s/base/                # Kubernetes manifests
│   ├── 00-namespace-rbac.yaml
│   ├── 01-secrets-config.yaml
│   ├── 02-agent-gateway.yaml
│   ├── 03-agents.yaml
│   ├── 04-network-policy.yaml
│   └── 05-mcp-servers.yaml
├── legacy_source/           # Sample COBOL programs
│   └── interest_calc.cbl
├── data/                    # Test data
│   └── loan_data.json
├── .env.example             # Environment template — start here!
├── deploy.ps1               # One-command Kubernetes deployment + demo
├── demo.ps1                 # Run modernization pipeline
└── README.md
```

---

## 📊 Validation Process

```
┌─────────────────────────────────────────┐
│  1. Compile COBOL with GnuCOBOL         │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  2. Execute COBOL with test data        │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  3. Generate Rust code (claude-opus-4-6)│
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  4. Compile Rust with Cargo             │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  5. Execute Rust with same test data    │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  6. Compare outputs (normalized)        │
│     └─> If match: Save to S3 ✅         │
│     └─> If mismatch: Needs review ⚠️    │
└─────────────────────────────────────────┘
```

---

## 🏆 Competitive Advantages

### vs AWS Mainframe Modernization

| Feature | AWS Solution | Our Solution |
|---|---|---|
| **COBOL Modernization** | ✅ AI-powered (to Java) | ✅ AI-powered (to Rust) |
| **Assembler Support** | ❌ Requires vendors (MLogica) | 🔄 Planned Phase 3 |
| **Validation** | ⚠️ Manual testing | ✅ Automated output comparison |
| **Target Language** | Java | **Memory-safe Rust** |
| **Agent Security** | ❌ No agent AuthZ | ✅ JWT + RBAC Agent Gateway |
| **Orchestration** | Managed services | ✅ Kubernetes + HPA |
| **Cost** | Vendor fees ($1M+) | Open-source tools |
| **Vendor Lock-in** | High | None |

---

## 🎓 Lessons Learned

### Technical Challenges Overcome

1. **GnuCOBOL Quirks**: Implied decimal (`V`) handling → Used `FUNCTION NUMVAL`
2. **Packed Decimal (COMP-3)**: Precision errors → Decimal literals and explicit arithmetic
3. **AI Model Selection**: Evaluated Gemini 2.5 Pro → Switched to **Claude claude-opus-4-6** for reliable, consistent Rust code generation
4. **Base64 Encoding**: Unnecessary complexity → Switched to plain text JSON
5. **Secret Management**: Nearly committed AWS credentials → Proper `.gitignore`, rotated secrets
6. **Rust Workspace**: `agent_gateway` not in workspace members → Added to root `Cargo.toml`
7. **MCP Naming**: Renamed from `gemini_mcp` to `ai_mcp` — reflects actual function (AI translation) not the model name
8. **RwLock Clone**: `GatewayClient` derived `Clone` on non-cloneable field → Removed derive
9. **GLIBC Mismatch on Kubernetes**: `cobol-mcp` and `s3-mcp` crashed on `debian:bookworm-slim` — runtime stage changed to `FROM rust:latest`
10. **JWT Token Expiry**: Green Agent cached token at startup, expired after ~3 hours → Added auto-refresh on 401 in `invoke_mcp()`
11. **Missing Health Endpoint**: Purple Agent lacked `/health` route — Kubernetes liveness probe killed it every 30 seconds (34 restarts!) → Added `/health` GET endpoint

### Key Insights
- 📚 **100+ S3 uploads** to debug COBOL compilation issues
- ⏰ **48+ hours** debugging model configurations
- 🎯 **Rust compiler messages** are invaluable — far clearer than Assembler's cryptic codes
- 🔒 **Security first**: Agent Gateway enforces zero-trust between all agents
- 🤖 **Claude consistency**: Claude claude-opus-4-6 produced working Rust where Gemini 2.5 Pro failed repeatedly
- ☸️ **Kubernetes hardening**: Real-world issues (GLIBC, JWT expiry, missing health endpoints) only surface under production-like conditions

---

## 🔮 Future Enhancements

### Phase 3: Enterprise Features
- **IBM HLASM Assembler → Rust translation** (architecture designed, implementation planned)
- IBM z/OS COBOL compiler integration (IBM Developer for z/OS Enterprise Edition — active trial)
- Batch processing (multiple COBOL files simultaneously)
- COBOL-CICS transaction support
- COBOL-DB2 embedded SQL support
- VSAM→DynamoDB/RDS migration
- Migration reports & analytics
- Kubernetes production hardening (multi-node cluster, Ingress controller, TLS termination, persistent volumes)

---

## 👥 Contributors

**Venkat Nagala** — 30 years insurance and banking mainframe technology
- GitHub: [@venkatnagala](https://github.com/venkatnagala)
- LinkedIn: [Venkat Nagala](https://www.linkedin.com/in/tenalirama)
- Blog: [Dev.to](https://dev.to/venkateshwar_raonagala_4)

---

## 📄 License

MIT License — see [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- **Claude claude-opus-4-6** (Anthropic) for AI-powered code translation
- **GnuCOBOL** for open-source COBOL compilation
- **AWS** for cloud infrastructure
- **AgentAheads Hackathon** for Phase 1 inspiration
- **Solo.io** for the SOLO AI Competition platform

---

*Built with ❤️ — Modernizing mainframes, one COBOL line at a time. Assembler support coming in Phase 3!* 🚀
