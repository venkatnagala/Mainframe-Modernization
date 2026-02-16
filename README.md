![CI Status](https://github.com/venkatnagala/Mainframe-Modernization/actions/workflows/rust.yml/badge.svg)
# Mainframe Modernization: AI-Powered COBOL to Rust Pipeline

[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Docker](https://img.shields.io/badge/docker-%230db7ed.svg?style=for-the-badge&logo=docker&logoColor=white)](https://www.docker.com/)
[![AWS](https://img.shields.io/badge/AWS-%23FF9900.svg?style=for-the-badge&logo=amazon-aws&logoColor=white)](https://aws.amazon.com/)

> An automated, AI-powered system that modernizes legacy mainframe COBOL applications to memory-safe Rust with automated validation.

## 🎯 Problem Statement

Enterprise mainframe applications written in COBOL face critical challenges:
- **Aging workforce**: COBOL programmers retiring faster than new ones learning
- **Maintenance costs**: Legacy systems expensive to maintain
- **Technical debt**: Decades-old codebases difficult to modify
- **AWS gap**: AWS Mainframe Modernization uses AI for COBOL→Java but **requires expensive vendors** (like MLogica) for Assembler modernization

## 💡 Our Solution

A complete **AI-powered modernization pipeline** that:
1. ✅ Fetches legacy COBOL from AWS S3
2. ✅ Modernizes to idiomatic Rust using **Gemini 2.5 Pro**
3. ✅ **Validates correctness** by comparing outputs
4. ✅ Saves verified code back to S3 with secure access
5. ✅ **Extends to Assembler** (solving AWS's gap)

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      AWS S3 Storage                         │
│  ┌──────────────┐  ┌──────────────┐  ┌─────────────────┐  │
│  │   programs/  │  │    data/     │  │  modernized/    │  │
│  │ (COBOL src)  │  │ (test data)  │  │ (Rust output)   │  │
│  └──────────────┘  └──────────────┘  └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            ▲ │
                            │ │
                    ┌───────┘ └───────┐
                    │                 │
        ┌───────────▼─────────────────▼──────────┐
        │      Green Agent (Orchestrator)        │
        │  • Fetches COBOL source from S3        │
        │  • Compiles & executes COBOL (GnuCOBOL)│
        │  • Compiles & executes Rust (Cargo)    │
        │  • Validates outputs match             │
        │  • Generates pre-signed S3 URLs        │
        └────────────────┬───────────────────────┘
                         │
                         │ API Call
                         │
        ┌────────────────▼───────────────────────┐
        │    Purple Agent (AI Modernizer)        │
        │  • Powered by Gemini 2.5 Pro          │
        │  • Converts COBOL → Idiomatic Rust    │
        │  • Handles packed decimals (COMP-3)   │
        │  • Generates memory-safe code         │
        └────────────────────────────────────────┘
```

## ✨ Key Features

### 🤖 AI-Powered Modernization
- **Gemini 2.5 Pro** for intelligent code translation
- Handles complex COBOL constructs (COMP-3 packed decimals, file I/O)
- Generates idiomatic, memory-safe Rust code

### ✅ Automated Validation
- Compiles both COBOL (GnuCOBOL) and generated Rust
- Executes with identical test data
- **Compares outputs** to ensure functional equivalence
- Only saves Rust code when outputs match ✓

### 🔒 Security & Best Practices
- **Least-privilege IAM** policies (read-only source, write-only outputs)
- **Pre-signed URLs** for time-limited, secure file access (1-hour expiry)
- **No secrets in code** - environment variables only
- Docker containerization for consistent deployment

### 🚀 Production Ready
- Full Docker Compose setup
- Automated test scripts
- S3 integration for enterprise storage
- Extensible architecture (ready for Assembler support)

## 🎥 Demo

### Success Case: Interest Calculation
```bash
Input:  Loan Amount: $10,000.00, Rate: 5.5%
COBOL:  "CALCULATED INTEREST:     550.00"
Rust:   "CALCULATED INTEREST: 550.00"
Result: ✅ MATCH CONFIRMED - Code saved to S3
```

## 🛠️ Tech Stack

| Component | Technology | Purpose |
|-----------|-----------|---------|
| **AI Model** | Gemini 2.5 Pro | COBOL→Rust translation |
| **Backend** | Rust + Actix-web | Green Agent orchestration |
| **COBOL Compiler** | GnuCOBOL | Validate original code |
| **Storage** | AWS S3 | Source & output storage |
| **Security** | AWS IAM + Pre-signed URLs | Access control |
| **Deployment** | Docker + Docker Compose | Containerization |
| **Languages** | Rust, COBOL, PowerShell | Implementation |

## 📋 Prerequisites

- **Docker Desktop** (with Docker Compose)
- **AWS Account** with S3 access
- **Gemini API Key** (get from [Google AI Studio](https://aistudio.google.com/app/apikey))
- **Git** (for cloning the repository)

## 🚀 Quick Start

### 1. Clone Repository
```bash
git clone https://github.com/venkatnagala/Mainframe-Modernization.git
cd Mainframe-Modernization
```

### 2. Configure Environment
Create `.env` file in project root:
```bash
# AWS Credentials
AWS_ACCESS_KEY_ID=your_access_key
AWS_SECRET_ACCESS_KEY=your_secret_key
AWS_REGION=us-east-1

# Gemini API Key
GEMINI_API_KEY=your_gemini_api_key
```

### 3. Setup AWS S3 Bucket
Create S3 bucket with this structure:
```
mainframe-refactor-lab-venkatnagala/
├── programs/          # Upload COBOL source files here
│   └── interest_calc.cbl
├── data/             # Upload test data here
│   └── loan_data.json
├── modernized/       # Auto-generated Rust outputs
└── raw_logs/         # Auto-generated execution logs
```

**IAM Policy** (attach to your AWS user):
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowListBucket",
      "Effect": "Allow",
      "Action": "s3:ListBucket",
      "Resource": "arn:aws:s3:::mainframe-refactor-lab-venkatnagala"
    },
    {
      "Sid": "AllowReadSource",
      "Effect": "Allow",
      "Action": "s3:GetObject",
      "Resource": [
        "arn:aws:s3:::mainframe-refactor-lab-venkatnagala/programs/*",
        "arn:aws:s3:::mainframe-refactor-lab-venkatnagala/data/*"
      ]
    },
    {
      "Sid": "AllowWriteOutputs",
      "Effect": "Allow",
      "Action": ["s3:PutObject", "s3:GetObject"],
      "Resource": [
        "arn:aws:s3:::mainframe-refactor-lab-venkatnagala/modernized/*",
        "arn:aws:s3:::mainframe-refactor-lab-venkatnagala/raw_logs/*"
      ]
    }
  ]
}
```

### 4. Build and Run
```bash
# Build containers
docker-compose build

# Start services
docker-compose up
```

The Green Agent will be available at `http://localhost:8080`

### 5. Test the Pipeline
```powershell
# PowerShell
$json = '{"task_id":"TEST_01","source_location":{"bucket":"mainframe-refactor-lab-venkatnagala","key":"programs/interest_calc.cbl"}}'
Invoke-RestMethod -Uri http://localhost:8080/evaluate -Method POST -ContentType "application/json" -Body $json
```

**Expected Response:**
```json
{
  "task_id": "TEST_01",
  "status": "SUCCESS - Outputs match!",
  "match_confirmed": true,
  "rust_code_url": "https://s3.amazonaws.com/...?X-Amz-Signature=...",
  "logs_url": "https://s3.amazonaws.com/...?X-Amz-Signature=..."
}
```

## 📁 Project Structure

```
Mainframe-Modernization/
├── green_agent/              # Orchestration service
│   ├── src/
│   │   └── main.rs          # Main orchestrator logic
│   ├── Dockerfile
│   └── Cargo.toml
├── purple_agent/            # AI modernization service (placeholder)
│   └── ...
├── legacy_source/           # Sample COBOL programs
│   └── interest_calc.cbl
├── data/                    # Test data
│   └── loan_data.json
├── docker-compose.yml       # Multi-container orchestration
├── .gitignore
└── README.md
```

## 🧪 Testing

### Run Demo Script
```powershell
.\demo.ps1
```

### Manual Testing
1. **Upload COBOL** to `s3://your-bucket/programs/`
2. **Upload test data** to `s3://your-bucket/data/`
3. **Send API request** to `http://localhost:8080/evaluate`
4. **Download results** via pre-signed URLs

## 📊 Validation Process

```
┌─────────────────────────────────────────┐
│  1. Compile COBOL with GnuCOBOL         │
│     └─> Create executable               │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  2. Execute COBOL with test data        │
│     └─> Capture output                  │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  3. Generate Rust code (Gemini)         │
│     └─> AI translates COBOL→Rust        │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  4. Compile Rust with Cargo             │
│     └─> Build with dependencies         │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  5. Execute Rust with same test data    │
│     └─> Capture output                  │
└───────────────┬─────────────────────────┘
                │
┌───────────────▼─────────────────────────┐
│  6. Compare outputs (normalized)        │
│     └─> If match: Save to S3 ✅         │
│     └─> If mismatch: Needs review ⚠️    │
└─────────────────────────────────────────┘
```

## 🎓 Lessons Learned

### Technical Challenges Overcome:
1. **GnuCOBOL Quirks**: Implied decimal (`V`) handling issues
   - Solution: Used `FUNCTION NUMVAL` and simple `MULTIPLY ... GIVING`
   
2. **Packed Decimal (COMP-3)**: Precision and calculation errors
   - Solution: Decimal literals and explicit arithmetic order

3. **Gemini Model Selection**: Original `gemini-3-pro-preview` didn't exist
   - Solution: Used `gemini-2.5-pro` (correct model name)

4. **Base64 Encoding Issues**: Unnecessary complexity in data transfer
   - Solution: Switched to plain text JSON responses

5. **Secret Management**: Nearly committed AWS credentials to Git
   - Solution: Proper `.gitignore`, environment variables, rotated secrets

### Key Insights:
- 📚 **100+ S3 uploads** to debug COBOL compilation issues
- ⏰ **48+ hours** spent debugging model configurations
- 🎯 **Compiler messages**: Rust's helpful errors vs Assembler's cryptic codes
- 🔒 **Security first**: Pre-signed URLs, least-privilege IAM, no secrets in code

## 🏆 Competitive Advantages

### vs AWS Mainframe Modernization:
| Feature | AWS Solution | Our Solution |
|---------|-------------|--------------|
| **COBOL Modernization** | ✅ AI-powered (to Java) | ✅ AI-powered (to Rust) |
| **Assembler Support** | ❌ Requires vendors (MLogica) | ✅ Architecture supports it |
| **Validation** | ⚠️ Manual testing | ✅ Automated output comparison |
| **Target Language** | Java | **Memory-safe Rust** |
| **Cost** | Vendor fees | Open-source tools |

## 🔮 Future Enhancements

### Phase 1: Assembler Support ⏳
- Extend pipeline to handle IBM mainframe Assembler (HLASM/BAL)
- Prove capability AWS doesn't offer without vendors

### Phase 2: Solo.ai Competition 🎯
- Integrate **KAgent/Agent Gateway** for security
- Add authentication & authorization for MCP servers
- Deploy to Kubernetes with Helm

### Phase 3: Production Features
- Batch processing (multiple files)
- Migration reports & analytics
- Support for CICS, DB2, IMS
- Performance comparison dashboards

## 👥 Contributors

**Venkat Nagala**
- GitHub: [@venkatnagala](https://github.com/venkatnagala)
- LinkedIn: [Venkat Nagala](https://www.linkedin.com/in/venkatnagala/)

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **Google Gemini 2.5 Pro** for AI-powered code translation
- **GnuCOBOL** for open-source COBOL compilation
- **Anthropic Claude** for development assistance
- **AWS** for cloud infrastructure
- **AgentAheads Hackathon** for the inspiration

## 📞 Support

For questions or issues:
1. Open a [GitHub Issue](https://github.com/venkatnagala/Mainframe-Modernization/issues)
2. Check existing documentation
3. Review demo scripts for examples

---

**Built with ❤️ for the AgentAheads Hackathon**

*Modernizing mainframes, one line of COBOL at a time* 🚀
