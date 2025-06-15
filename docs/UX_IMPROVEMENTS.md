# UX Improvement Next Steps

This document outlines a few practical next steps for improving the developer and user experience when using Agenterra. These suggestions are informed by the existing [ROADMAP](../ROADMAP.md) and current repo docs.

## 1. Interactive Scaffolding
- [x] Provide a `agenterra init` flow that prompts for schema location, output directory, and template choice.
- [x] Offer sensible defaults and validation to reduce setup friction.

## 2. Enhanced Error Messages
- [x] Display actionable suggestions when OpenAPI parsing fails.
- [x] Link to documentation sections for common issues.

## 3. Development Server
- [x] Allow rapid iteration with a `--watch` flag that rebuilds generated code on schema changes.
- [x] Surface build errors immediately within the CLI output.

## 4. Template Hot-Reload
- Detect template changes and regenerate files without restarting the CLI.
- Ideal for customizing built-in templates during development.

These improvements aim to streamline onboarding and day-to-day usage for new users, making the CLI feel more approachable and responsive.
