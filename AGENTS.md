# Repository Guidelines

## Workflows

Use `cargo x` as the source of truth for repository workflows. Read `cargo x --help` and the relevant subcommand's `--help` before running build, test, lint, or formatting commands.

## Rust Style

Declare restricted visibility at the module boundary and use `pub` for items in that module's API.

## Documentation

Keep each Markdown prose paragraph and list item on one source line. Format Markdown tables so their columns and separators align in the source.

## API Design

Keep encoding profiles and backend selection separate. Public APIs describe RFC 4648 semantics, storage, and streaming state; scalar, SIMD, and runtime dispatch remain private implementation details.

## Tests

Use RFC 4648 vectors or an independent codec for expected wire bytes, including streamed and formatted output. Exercise decoding size estimates with a buffer of the reported capacity. Reserve exact length assertions for APIs that promise an exact length.

## Pull Requests

Format pull request titles according to `.github/semantic.yml` and keep the description concise. Use a `Summary` section for routine changes and add `Design Notes` only when the design needs explanation.
