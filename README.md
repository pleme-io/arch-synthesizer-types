# arch-synthesizer-types

Self-contained EKS access-entry types shared between arch-synthesizer and downstream typed-ops consumers (cordel). No external deps beyond std so crate2nix can prefetch it hermetically in any sandbox.

Extracted from `arch-synthesizer::operator` so downstream hermetic
crates (cordel) can consume the types without pulling the whole
arch-synthesizer workspace (which uses cargo path deps into 14+
sibling synthesizer crates + `dq-core`, making crate2nix's prefetch
step impossible inside a Nix sandbox).
Every type here is wire-compatible with its namesake in

Part of the pleme-io code-generation pipeline:

```
sekkei -> takumi -> openapi-forge / iac-forge -> backend renderers
```

## License

MIT — see [LICENSE](./LICENSE).
