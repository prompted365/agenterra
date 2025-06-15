# 🚀 Agenterra: Model Context Protocol Generator

**Generate production-ready MCP (Model Context Protocol) servers from OpenAPI specs with minimal configuration.**

[![CI](https://github.com/clafollett/agenterra/workflows/CI/badge.svg)](https://github.com/clafollett/agenterra/actions/workflows/ci.yml)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/clafollett/agenterra?style=for-the-badge)](https://github.com/clafollett/agenterra/releases)
[![Rust](https://img.shields.io/badge/Rust-1.86.0%2B-orange?logo=rust&style=for-the-badge)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](LICENSE)
[![OpenAPI](https://img.shields.io/badge/OpenAPI-3.0-85ea2d?logo=openapi-initiative&style=for-the-badge)](https://www.openapis.org/)

---

**Agenterra** transforms your OpenAPI specifications into fully-functional MCP servers with type-safe Rust code, ready for integration with AI tools and workflows. Perfect for:

- **AI/ML Engineers** 🤖 - Quickly expose APIs for LLM tool use
- **API Developers** 🛠️ - Generate production-ready MCP servers from existing OpenAPI specs
- **FinTech & Data Teams** 📊 - Build compliant financial data APIs with built-in validation
- **Startups & Enterprises** 🚀 - Accelerate development of AI-powered applications

## ✨ Features

- **⚡ Blazing Fast** - Built with Rust for maximum performance and safety
- **🔌 OpenAPI 3.0+ Support** - Seamless integration with existing API specifications
- **🦀 Type-Safe Rust** - Generate idiomatic, production-ready Rust code
- **🎨 Template-Based** - Customize every aspect with Tera templates
- **🔍 Built-in Validation** - Automatic OpenAPI schema validation
- **🚀 Production Ready** - Includes logging, error handling, and configuration out of the box
- **🔌 MCP Protocol Support** - Full compatibility with Model Context Protocol
- **📦 Docker & Binary** - Multiple deployment options for any environment

## 🚀 Quick Start

### Prerequisites

- [Rust 1.86.0+](https://rustup.rs/)
- [Docker](https://www.docker.com/) (optional, for containerized deployment)

### Method 1: Build & Run from Source

Run an interactive wizard with:

```bash
agenterra init
```

```bash
# Clone the repository
git clone https://github.com/clafollett/agenterra.git
cd agenterra

# Generate from a local file without install:
cargo run -p agenterra -- scaffold --schema-path ./tests/fixtures/openapi/petstore.openapi.v3.json --output .agenterra/cargo_run_petstore_mcp_server_local_file

# Generate from a remote URL without install:
cargo run -p agenterra -- scaffold --schema-path https://petstore3.swagger.io/api/v3/openapi.json --output .agenterra/cargo_run_petstore_mcp_server_remote_url

# Or install the CLI
cargo install --path crates/agenterra-cli

# Generate your MCP server from a local file
agenterra scaffold --schema-path ./tests/fixtures/openapi/petstore.openapi.v3.json --output .agenterra/installed_petstore_mcp_server_local_file

# Generate from a remote URL
agenterra scaffold --schema-path https://petstore3.swagger.io/api/v3/openapi.json --output .agenterra/installed_petstore_mcp_server_remote_url

```

> **Note:** Agenterra uses a Cargo workspace. You must use the CLI crate path (`crates/agenterra-cli`) for `cargo install`. Top-level install will not work.

### Method 2: Install from Git

```bash
# Install latest version
cargo install --git https://github.com/clafollett/agenterra.git

# Install specific version. Example: v0.1.0
cargo install --git https://github.com/clafollett/agenterra.git --tag v<VERSION>
```

### Method 3: From Pre-built Binary (Coming soon)

1. Download the latest release for your platform from [Releases](https://github.com/clafollett/agenterra/releases)
2. Make it executable and run:
   ```bash
   chmod +x agenterra
   
   # Generate your MCP server from a local file
   ./agenterra scaffold --schema-path ./tests/fixtures/openapi/petstore.openapi.v3.json --output .agenterra/installed_petstore_mcp_server_local_file

   # Generate from a remote URL
   ./agenterra scaffold --schema-path https://petstore3.swagger.io/api/v3/openapi.json --output .agenterra/installed_petstore_mcp_server_remote_url
   ```

## 🔌 Integrating with MCP Clients

### VS Code Integration

Add this to your VS Code settings (File > Preferences > Settings > Open Settings JSON):

```json
{
  "mcp": {
    "servers": {
      "petstore": {
        "command": "cargo",
        "args": ["run", "--manifest-path", "/path/to/petstore-server/Cargo.toml"]
      }
    }
  }
}
```

### Cursor Integration

Add this to your Cursor settings (File > Preferences > Settings > Extensions > MCP):

```json
{
  "mcpServers": {
    "petstore": {
      "command": "cargo",
      "args": ["run", "--manifest-path", "/path/to/petstore-server/Cargo.toml"]
    }
  }
}
```

### 🕵️‍♂️ Testing with MCP Inspector

Test your MCP server with the MCP Inspector:

```bash
# Run directly with npx
npx @modelcontextprotocol/inspector cargo run --manifest-path=/path/to/petstore-server/Cargo.toml

# Or install globally
npm install -g @modelcontextprotocol/inspector
modelcontextprotocol-inspector cargo run --manifest-path=/path/to/petstore-server/Cargo.toml
```

### 🐳 Docker Integration

For production use, build and run with Docker:

```bash
# Build the image
cd petstore-server
docker build -t petstore-mcp .

# Run the container
docker run -i --rm petstore-mcp
```

Then update your MCP client configuration to use the Docker container:

```json
{
  "mcp": {
    "servers": {
      "petstore": {
        "command": "docker",
        "args": ["run", "-i", "--rm", "petstore-mcp"]
      }
    }
  }
}
```

## ⚡ Best Practices

Agenterra is designed to scaffold a well-structured MCP servers from OpenAPI specs. This is a great starting point, not necessarily a `Best Practice`. Wrapping an OpenAPI spec under an MCP facade is convenient, but not always the “proper” way to build MCPs. For robust, agent-friendly tools, consider how your server can best expose business logic, aggregate data, and provide clear, useful tool contracts.

**Considerations:**
- Treat the generated code as a foundation to extend and customize.
- Don't assume a 1:1 mapping of OpenAPI endpoints to MCP tools is ideal; you may want to aggregate multiple API calls into a single tool, or refactor handlers for advanced logic.
- Use the scaffold to rapidly stub out endpoints, then iterate and enhance as needed.

---

## 🤔 Why Agenterra?

Postman now offers robust support for the Model Context Protocol (MCP), including:
- MCP client and server features
- Code generation
- A catalog of hosted, discoverable MCP endpoints
- Visual agent-building and cloud collaboration

**When should you use Agenterra?**
- **Offline, air-gapped, or regulated environments** where cloud-based tools aren’t an option
- **Rust-first, codegen-centric workflows:** Generate type-safe, production-grade Rust MCP servers from OpenAPI specs, ready for CI/CD and self-hosting
- **Full template control:** Tweak every line of generated code, use custom templates, and integrate with your own infra
- **CLI-first automation:** Perfect for embedding in build scripts, Docker, and serverless workflows

**When should you use Postman?**
- Visual design, rapid prototyping, and cloud collaboration
- Building, testing, and deploying MCP agents with a GUI
- Discovering and consuming public MCP endpoints

**Summary:**
- Use Postman for visual, collaborative, and cloud-first agent development
- Use Agenterra for local, reproducible, code-first MCP server generation with maximum control and zero cloud dependencies

---

## 🏛️ Architecture

Agenterra is built for extensibility, automation, and code quality. Here’s how the core pieces fit together:

**Core Modules:**
- `openapi`: Loads and validates OpenAPI specs (YAML/JSON, local or URL)
- `generator`: Orchestrates code generation from the parsed OpenAPI model
- `template`: Handles Tera-based templates for idiomatic Rust code
- `cli`: Command-line interface for scaffolding, configuration, and workflow

**Code Generation Flow:**

```
OpenAPI Spec (local file or URL)
         │
         ▼
   [openapi module]
         │
         ▼
   [generator module]
         │
         ▼
   [template module]
         │
         ▼
Generated Rust MCP Server (Axum, etc.)
```

- The generated server uses [Stdio](https://modelcontextprotocol.io/introduction) as the primary MCP protocol for agent integration, but can be extended for HTTP/SSE and other transports.
- All code is idiomatic Rust, ready for further customization and production deployment.

---

## 🤝 Contributing

We welcome contributions from the community! To keep Agenterra high-quality and maintainable, please follow these guidelines:

- **Fork & Clone**: Fork the repo and clone your fork locally.
- **Branch Naming**: Use the convention `GH-<issue-number>_<ProperCaseSummary>` (e.g., `GH-9_EndToEndIntegrationTest`).
- **Pull Requests**:
  - All PRs require review.
  - All tests must pass (`cargo test` and integration tests).
  - Code coverage must not decrease.
  - Update documentation for any user-facing or API changes.
- **Testing**:
  - Add or update unit and integration tests for all new features or bugfixes.
  - Run: `cargo test -p agenterra --test integration_test`
- **Docs**:
  - Update relevant docs and add examples for new features.
  - Document any new patterns or conventions.
- **CI/CD**:
  - Ensure your branch passes all checks before requesting review.

For more details, see [CONTRIBUTING.md](CONTRIBUTING.md) if available.

---

## 🛠️ Developer Workflow

Here’s how to work productively with Agenterra as a contributor or advanced user:

### 🧪 Running Tests
- **Unit & Integration Tests:**
  - Run all tests: `cargo test`
  - Run integration tests (all templates with OpenAPI specs):
    ```bash
    cargo test -p agenterra --test integration_test
    ```
- **Test Location:** See [`crates/agenterra-cli/tests/integration_test.rs`](crates/agenterra-cli/tests/integration_test.rs) for integration coverage.
- **Test-First Principle:** Add failing tests before implementing new features or bugfixes.

### 🏗️ Building
- **Standard build:**
  ```bash
  cargo build --release
  ```
- **Docker build:**
  ```bash
  docker build -t agenterra .
  ```

### 🧩 Adding Templates or Plugins
- See [`docs/TEMPLATES.md`](docs/TEMPLATES.md) for template structure, manifest, variables, and hooks.
- Add new templates under the `templates/` directory (do not modify existing templates without explicit approval).

### ⚡ Local Development Tips
- Use branch naming convention: `GH-<issue-number>_<ProperCaseSummary>`
- Run `cargo fmt` and `cargo clippy` before pushing
- Update documentation and examples with every user-facing change
- All code must be idiomatic Rust and pass CI checks

---

## 🎬 Demo

Want to see Agenterra in action? Check back soon for:
- **Loom/asciinema walkthroughs** showing:
  - Scaffolding an MCP server from an OpenAPI spec (local & URL)
  - Running the generated server (locally or via Docker)
  - Integrating with AI agents via the MCP protocol
- **Community demos**: Submit your own demo links via PR!

👉 _Have a killer workflow or agent integration? [Open a PR](https://github.com/clafollett/agenterra/pulls) to add your demo here!_

---

## 🏗️ Project Structure

```
petstore-server/
├── Cargo.toml          # Rust project manifest
├── src/
│   ├── mcp/            # MCP protocol implementation
│   │   ├── mod.rs       # MCP server implementation
│   │   └── handlers/    # MCP request handlers
│   ├── api/             # Generated API code
│   │   ├── mod.rs       # API module exports
│   │   ├── models/      # Generated data models
│   │   └── operations/  # API operation handlers
│   ├── config.rs        # Server configuration
│   ├── error.rs         # Error handling
│   └── main.rs          # MCP server entry point
├── .env                # Environment variables
└── README.md           # Project documentation
```

## Configuration ⚙️

Agenterra can be configured through multiple methods (in order of precedence):

1. **Command-line arguments**
   ```bash
   agenterra generate --input spec.yaml --output my_server --template rust_axum
   ```

2. **Configuration file** (`agenterra.toml` in project root)
   ```toml
   [generate]
   input = "openapi.yaml"
   output = "my_server"
   template = "rust_axum"
   ```

3. **Environment variables**
   ```bash
   export AGENTERRA_INPUT=openapi.yaml
   export AGENTERRA_OUTPUT=my_server
   agenterra generate
   ```

## Templates 🎨

Agenterra uses [Tera](https://tera.netlify.app/) templates for code generation. You can use built-in templates or create your own.

### Built-in Templates
- `rust_axum`: Generate a server using the [Axum](https://github.com/tokio-rs/axum) web framework

### Custom Templates
Create a `templates` directory in your project root and add your template files. Agenterra will use these instead of the built-in templates.

## Examples 📚

### Generate a server from Petstore API
```bash
# Download the Petstore OpenAPI spec
curl -o petstore.yaml https://raw.githubusercontent.com/OAI/OpenAPI-Specification/main/examples/v3.0/petstore.yaml

# Generate the server
agenterra generate --input petstore.yaml --output petstore-server

# Build and run
cd petstore-server
cargo run
```

## Contributing 🤝

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

## 📄 License

This project is licensed under the [MIT License](LICENSE).


## Related Projects 🔗

- [RMCP](https://github.com/windsurf-eng/rmcp) - Rust MCP implementation
- [MCP Protocol](https://github.com/windsurf-eng/mcp) - Model Context Protocol specification
- [Axum](https://github.com/tokio-rs/axum) - Web framework for Rust
- [Tera](https://tera.netlify.app/) - Template engine for Rust
