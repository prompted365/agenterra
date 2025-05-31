# MCPGen 🚀

Generate and scaffold MCP (Model Context Protocol) servers from OpenAPI specifications.

## Features 🌟

- Generate MCP servers from OpenAPI specifications
  - OpenAPI 3.0 support
  - OpenAPI/Swagger 2.0 support (coming soon)
- Scaffold new endpoints and handlers
- Type-safe code generation
- Customizable templates
- OpenAPI validation
- Development server

## Installation 📦

```bash
cargo install mcpgen
```

## Quick Start 🏃

1. Create a new MCP server:
```bash
mcpgen scaffold --spec api.yaml --output my-server
```

2. Add a new endpoint:
```bash
mcpgen scaffold --spec api.yaml --component endpoint --method GET --path /users/{id}
```

3. Generate handlers:
```bash
mcpgen generate --spec api.yaml --component handlers
```

4. Update existing components:
```bash
mcpgen update --spec api.yaml
```

## Project Structure 📁

See [tests/fixtures/README.md](tests/fixtures/README.md) for details on local OpenAPI test fixtures and the Petstore update script.

```
my-server/
├── Cargo.toml
├── src/
│   ├── handlers/           # Generated handlers
│   ├── models/            # Generated types
│   ├── config.rs          # Configuration
│   └── main.rs            # Server entry point
└── templates/             # Custom templates (optional)
```

## Configuration 🔧

MCPGen can be configured through:
- Command line arguments
- Configuration file (config.toml)
- Environment variables

See [Configuration Guide](docs/configuration.md) for details.

## Contributing 🤝

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## License 📄

This project is licensed under both:
- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

## Related Projects 🔗

- [RMCP](https://github.com/windsurf-eng/rmcp) - Rust MCP implementation
- [MCP Protocol](https://github.com/windsurf-eng/mcp) - Model Context Protocol specification
