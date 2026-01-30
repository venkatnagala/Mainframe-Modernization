Mainframe-Modernization: Surgical AI Refactoring
🚀 Overview
Mainframe-Modernization is an AI-driven orchestration framework designed to refactor legacy IBM Mainframe logic (COBOL/JCL) into cloud-native Rust.

Unlike enterprise-scale migration services that are cost-prohibitive for smaller workloads, this project targets the "Long Tail" of legacy systems—modular applications under 10,000 Lines of Code (LOC) where manual rewriting is risky but full-scale enterprise migration is unjustified.

The "Cost-Gap" Mission
Enterprise migration tools (like AWS Mainframe Modernization) are designed for millions of LOC. For a standalone 3,000 LOC module, the overhead of these services often exceeds the value. Our agents provide a surgical Refactoring-as-Code path:

Cost-Efficient: Pay only for LLM tokens and minimal compute.

Logic Fidelity: Uses a dual-runtime execution engine to verify logic parity.

Modern Stack: Converts procedural COBOL into memory-safe, asynchronous Rust with AWS S3 integration.

🛠️ Architecture
The system utilizes a Green Agent / Purple Agent architecture via the Agent-to-Agent (A2A) protocol:

Green Agent (The Referee):

Fetches legacy COBOL source from AWS S3.

Establishes "Ground Truth" by executing code via GnuCOBOL (cobc) using the -std=ibm dialect.

Challenges the Purple Agent to refactor the logic.

Validates the refactored Rust code by compiling and comparing stdout results.

Purple Agent (The Modernizer):

Receives the COBOL challenge.

Generates idiomatic Rust code, incorporating aws-sdk-s3 for cloud-native data handling.

🚦 Getting Started
Prerequisites
Docker & Docker Compose

AWS Credentials (with S3 Read access)

Rust 1.75+ (for local development)

Quick Start (Demo)
Clone the repository:

Bash
git clone https://github.com/YOUR_USERNAME/Mainframe-Modernization.git
cd Mainframe-Modernization
Set up your environment: Create a .env file in the root directory:

Code snippet
AWS_ACCESS_KEY_ID=your_key_here
AWS_SECRET_ACCESS_KEY=your_secret_here
Run the evaluation:

Bash
chmod +x demo.sh
./demo.sh
Verify Results: Check the logs to see the Fidelity Match between the legacy COBOL and the new Rust binary:

Bash
docker-compose logs -f green-agent
📊 Security & Safety
Isolated Execution: All code compilation and execution occur within Docker containers.

Credential Safety: AWS keys are managed via environment variables and are ignored by Git via .gitignore.

Alignment: The modernization agent is prompted to prioritize memory safety and prevent "hallucinations" in financial arithmetic.

⚖️ License
This project is licensed under the MIT License - see the LICENSE file for details.
