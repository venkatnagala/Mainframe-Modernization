# Mainframe Modernization: AI-Powered COBOL to Rust Pipeline

[![CI Status](https://github.com/venkatnagala/Mainframe-Modernization/actions/workflows/rust.yml/badge.svg)](https://github.com/venkatnagala/Mainframe-Modernization/actions/workflows/rust.yml)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![AWS](https://img.shields.io/badge/AWS-%23FF9900.svg?style=for-the-badge&logo=amazon-aws&logoColor=white)](https://aws.amazon.com/)
[![Kubernetes](https://img.shields.io/badge/kubernetes-%23326ce5.svg?style=for-the-badge&logo=kubernetes&logoColor=white)](https://kubernetes.io/)

> An automated, AI-powered system that modernizes legacy mainframe COBOL applications to memory-safe Rust with automated validation, secured by a zero-trust Agent Gateway.

---

## 🏆 Competition History

| Competition | Phase | Status |
|---|---|---|
| **AgentAheads Hackathon 2026** | Phase 1: AI-powered COBOL→Rust pipeline with Docker Compose | ✅ Completed |
| **SOLO AI Competition 2026** | Phase 2: Agent Gateway (JWT AuthN/AuthZ) + Kubernetes | 🔄 In Progress |

---

## 🎯 Problem Statement

Enterprise mainframe applications written in COBOL face critical challenges:

- **Aging workforce**: COBOL programmers retiring faster than new ones learning
- **Maintenance costs**: Legacy systems expensive to maintain
- **Technical debt**: Decades-old codebases difficult to modify
- **AWS gap**: AWS Mainframe Modernization uses AI for COBOL→Java but **requires expensive vendors** (like MLogica) for Assembler modernization

---

## 💡 Our Solution

A complete **AI-powered modernization pipeline** that:

1. ✅ Fetches legacy COBOL from AWS S3
2. ✅ Modernizes to idiomatic Rust using **Claude claude-opus-4-6**
3. ✅ **Validates correctness** by comparing outputs
4. ✅ Saves verified code back to S3 with secure access
5. ✅ **Secures all agent-to-MCP communication** via Agent Gateway (JWT + RBAC)
6. ✅ **Orchestrates containers** via Kubernetes with zero-trust NetworkPolicy
7. ✅ **Extends to Assembler** (solving AWS's gap)

---

## 🏗️ Architecture

### Phase 2: Zero-Trust Multi-Agent on Kubernetes (Current)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        Kubernetes Cluster                                   │
│                    (namespace: mainframe-modernization)                     │
│                                                                             │
│  ┌──────────────┐     JWT/HTTPS      ┌─────────────────────────────────┐   │
│  │              │ ────────────────► │       Agent Gateway              │   │
│  │ Green Agent  │                   │   (AuthN + AuthZ + Audit)        │   │
│  │(Orchestrator)│ ◄──────────────── │   Port: 8090 | Replicas: 2      │   │
│  │  Port: 8080  │    Proxy Result   └──────────┬──────────────────────┘   │
│  │  Replicas: 1 │                              │ Authorized calls only      │
│  └──────────────┘                              ▼                            │
│                                 ┌─────────────────────────────────────┐   │
│  ┌──────────────┐               │            MCP Servers              │   │
│  │ Purple Agent │ ──JWT/HTTPS──►│  ┌──────┐ ┌────────┐ ┌─────┐ ┌──┐  │   │
│  │(AI Modernizer│               │  │  S3  │ │ Gemini │ │COBOL│ │RS│  │   │
│  │  Port: 8085  │               │  │:8081 │ │ :8082  │ │:8083│ │T │  │   │
│  │  HPA: 1-5    │               └─────────────────────────────────────┘   │
│  └──────────────┘                                                           │
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

### Phase 1: Docker Compose Pipeline (AgentAheads Hackathon)

```
┌─────────────────────────────────────────────────────────────┐
│                      AWS S3 Storage                         │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   programs/  │  │    data/     │  │  modernized/    │  │
│  │ (COBOL src)  │  │ (test data)  │  │ (Rust output)   │  │
│  └──────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ▲ │
        ┌───────────────────┘ └───────────────────┐
        │                                         │
        ▼                                         │
┌───────────────────────────────────┐             │
│   Green Agent (Orchestrator)      │             │
│  • Fetches COBOL source from S3   │             │
│  • Compiles & executes COBOL      │             │
│  • Compiles & executes Rust       │             │
│  • Validates outputs match        │             │
│  • Generates pre-signed S3 URLs   │             │
└────────────────┬──────────────────┘             │
                 │ API Call                       │
                 ▼                                │
┌───────────────────────────────────┐             │
│   Purple Agent (AI Modernizer)    │             │
│  • Powered by claude-opus-4-6     │             │
│  • Converts COBOL → Rust          │             │
│  • Handles packed decimals        │             │
└───────────────────────────────────┘             │
```

---

## 🔐 Agent Gateway: Zero-Trust Security (Phase 2)

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

| Agent Role | S3 MCP | Gemini MCP | COBOL MCP | Rust MCP |
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
- **claude-opus-4-6** for intelligent code translation
- Handles complex COBOL constructs (COMP-3 packed decimals, file I/O)
- Generates idiomatic, memory-safe Rust code

### ✅ Automated Validation
- Compiles both COBOL (GnuCOBOL) and generated Rust
- Executes with identical test data
- **Compares outputs** to ensure functional equivalence
- Only saves Rust code when outputs match ✓

### 🔒 Security & Best Practices
- **Agent Gateway**: JWT authentication + RBAC for all MCP server access
- **Zero-trust NetworkPolicy**: Default DENY ALL in Kubernetes
- **Least-privilege IAM** policies (read-only source, write-only outputs)
- **Pre-signed URLs** for time-limited, secure file access (1-hour expiry)
- **No secrets in code** — environment variables + Kubernetes Secrets

### 🚀 Production Ready
- Kubernetes deployment with Helm chart
- HPA auto-scales Purple Agent (1→5 replicas) based on CPU
- Agent Gateway: 2 replicas, zero-downtime rolling updates
- Full Docker Compose setup for local development
- Automated test scripts

---

## 🎥 Demo

### Success Case: Interest Calculation

```
Input:  Loan Amount: $10,000.00, Rate: 5.5%
COBOL:  "CALCULATED INTEREST:     550.00"
Rust:   "CALCULATED INTEREST: 550.00"
Result: ✅ MATCH CONFIRMED - Code saved to S3
```

---

## 🛠️ Tech Stack

| Component | Technology | Purpose |
|---|---|---|
| **AI Model** | Claude claude-opus-4-6 (Anthropic) | COBOL→Rust translation |
| **Agent Gateway** | Rust + Actix-web | JWT AuthN + RBAC AuthZ |
| **Backend** | Rust + Actix-web | Green Agent orchestration |
| **COBOL Compiler** | GnuCOBOL | Validate original code |
| **Storage** | AWS S3 | Source & output storage |
| **Security** | JWT + AWS IAM + Pre-signed URLs | Access control |
| **Orchestration** | Kubernetes + Helm | Container orchestration |
| **Deployment** | Docker + Docker Compose | Local development |
| **Languages** | Rust, COBOL, PowerShell | Implementation |

---

## 📋 Prerequisites

- **Docker Desktop** (with Docker Compose) for local dev
- **Kubernetes cluster** (Docker Desktop K8s / EKS) for Phase 2
- **Helm 3.x** for Kubernetes deployment
- **AWS Account** with S3 access
- **Claude API Key** (get from [Claude Developer Platform]( https://console.anthropic.com))
- **Git** (for cloning the repository)

---

## 🚀 Quick Start

### Option A: Local Development (Docker Compose)

#### 1. Clone Repository
```bash
git clone https://github.com/venkatnagala/Mainframe-Modernization.git
cd Mainframe-Modernization
```

#### 2. Configure Environment
Create `.env` file in project root:
```bash
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_REGION=us-east-1
Claude_API_KEY=your_claude_api_key
```

#### 3. Build and Run
```powershell
docker-compose build
docker-compose up
```

#### 4. Test the Pipeline
```powershell
$json = '{"task_id":"TEST_01","source_location":{"bucket":"mainframe-refactor-lab-venkatnagala","key":"programs/interest_calc.cbl"}}'
Invoke-RestMethod -Uri http://localhost:8080/evaluate -Method POST -ContentType "application/json" -Body $json
```

---

### Option B: Kubernetes Deployment (Phase 2)

#### 1. Set environment variables
```powershell
$env:CLAUDE_API_KEY = "your-claude-key"
$env:AWS_ACCESS_KEY_ID = "your-aws-key"
$env:AWS_SECRET_ACCESS_KEY = "your-aws-secret"
```

#### 2. Deploy with one command
```powershell
.\deploy.ps1 -Environment local
```

#### 3. Test Agent Gateway
```powershell
# Get JWT token
$token = (Invoke-RestMethod -Uri http://localhost:8090/auth/token -Method POST `
  -ContentType "application/json" `
  -Body '{"agent_id":"green_agent","api_key":"your-key","requested_role":"orchestrator"}'
).access_token

# Health check
Invoke-RestMethod -Uri http://localhost:8090/health -Method GET

# Invoke MCP via gateway
Invoke-RestMethod -Uri http://localhost:8090/mcp/invoke -Method POST `
  -Headers @{Authorization="Bearer $token"} `
  -ContentType "application/json" `
  -Body '{"target_mcp":"s3_mcp","operation":"fetch_source","payload":{"bucket":"mainframe-refactor-lab-venkatnagala","key":"programs/interest_calc.cbl"}}'
```

---

## 📁 Project Structure

```
Mainframe-Modernization/
├── agent_gateway/            # 🆕 Phase 2: Zero-trust security gateway
│   ├── src/main.rs          # JWT auth, RBAC, audit trail
│   ├── Cargo.toml
│   └── Dockerfile
├── green_agent/              # Orchestration service
│   ├── src/main.rs          # Updated: routes calls via Agent Gateway
│   ├── Dockerfile
│   └── Cargo.toml
├── purple_agent/             # AI modernization service
│   └── ...
├── k8s/base/                 # 🆕 Phase 2: Kubernetes manifests
│   ├── 00-namespace-rbac.yaml
│   ├── 01-secrets-config.yaml
│   ├── 02-agent-gateway.yaml
│   ├── 03-agents.yaml
│   └── 04-network-policy.yaml
├── helm/                     # 🆕 Phase 2: Helm chart
│   └── mainframe-modernization/
│       └── values.yaml
├── legacy_source/            # Sample COBOL programs
│   └── interest_calc.cbl
├── data/                     # Test data
│   └── loan_data.json
├── docker-compose.yml        # Local development
├── deploy.ps1                # 🆕 Phase 2: One-command K8s deploy
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
| **Assembler Support** | ❌ Requires vendors (MLogica) | ✅ Architecture supports it |
| **Validation** | ⚠️ Manual testing | ✅ Automated output comparison |
| **Target Language** | Java | **Memory-safe Rust** |
| **Agent Security** | ❌ No agent AuthZ | ✅ JWT + RBAC Agent Gateway |
| **Cost** | Vendor fees | Open-source tools |

---

## 🎓 Lessons Learned

### Technical Challenges Overcome

1. **GnuCOBOL Quirks**: Implied decimal (`V`) handling → Used `FUNCTION NUMVAL`
2. **Packed Decimal (COMP-3)**: Precision errors → Decimal literals and explicit arithmetic
3. **AI Model Selection**: Gemini model inconsistencies → Switched to Claude claude-opus-4-6 for reliable, consistent Rust code generation
4. **Base64 Encoding**: Unnecessary complexity → Switched to plain text JSON
5. **Secret Management**: Nearly committed AWS credentials → Proper `.gitignore`, rotated secrets
6. **Rust Workspace**: `agent_gateway` not in workspace members → Added to root `Cargo.toml`
7. **RwLock Clone**: `GatewayClient` derived `Clone` on non-cloneable field → Removed derive

### Key Insights
- 📚 **100+ S3 uploads** to debug COBOL compilation issues
- ⏰ **48+ hours** debugging model configurations
- 🎯 **Rust compiler messages** are invaluable vs Assembler's cryptic codes
- 🔒 **Security first**: Agent Gateway enforces zero-trust between all agents

---

## 🔮 Future Enhancements

### Phase 3: Production Features
- Batch processing (multiple COBOL files)
- Migration reports & analytics
- Support for CICS, DB2, IMS
- Performance comparison dashboards
- MCP Tasks for long-running async translation jobs

---

## 👥 Contributors

**Venkat Nagala**
- GitHub: [@venkatnagala](https://github.com/venkatnagala)
- LinkedIn: [Venkat Nagala](https://www.linkedin.com/in/tenalirama)

---

## 📄 License

MIT License — see [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- **Claude claude-opus-4-6** for AI-powered code translation
- **GnuCOBOL** for open-source COBOL compilation
- **Anthropic Claude** for development assistance
- **AWS** for cloud infrastructure
- **AgentAheads Hackathon** for the Phase 1 inspiration
- **Solo.io** for the SOLO AI Competition platform

---

*Built with ❤️ — Modernizin
 mainframes, one line of COBOL at a time* 🚀
