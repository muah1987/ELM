# Contributing to ELM

> Welcome. This project is free for all. No gatekeeping.

---

## Who We Want

Everyone.

You do not need a PhD. You do not need to be an AI researcher. You do not need to know Rust before you start.

If any of the following describes you, you belong here:

- You can write Rust (or want to learn)
- You work with embedded systems or bare-metal hardware
- You think clearly about memory, compression, or information theory
- You know neuroscience, cognitive science, or biology
- You can design hardware circuits
- You write clear documentation
- You ask good questions that nobody else thought to ask
- You found a bug

---

## What Needs Building

### High Priority

| Area | Skills needed | Difficulty |
|---|---|---|
| Grid World simulator | Rust | Beginner |
| Pocket data structure | Rust | Beginner |
| Cosine similarity engine | Rust, linear algebra | Beginner |
| Compression trigger logic | Rust | Intermediate |
| Tier 1 rule generation | Rust, statistics | Intermediate |
| Prediction engine | Rust | Intermediate |
| Raspberry Pi 4 bare-metal boot | Rust no_std, ARM assembly | Advanced |
| BCM2711 hardware drivers | Rust no_std, hardware | Advanced |
| NVMe block device driver | Rust no_std, hardware | Advanced |

### Medium Priority

| Area | Skills needed |
|---|---|
| Bifurcation / contradiction engine | Rust |
| Multi-step hierarchical planner | Rust |
| ANN (approximate nearest neighbor) search | Rust, algorithms |
| Visualization / debug tooling | Rust or Python |
| Action embedding (Phase 2 transition) | Rust, ML |
| Performance benchmarking | Rust |

### Open Research

| Question | Background helpful |
|---|---|
| What is the right N for compression threshold? | Statistics, empirical testing |
| How should Tier 2 abstraction be triggered? | Cognitive science |
| How should the Self_Model degrade gracefully? | Control theory |
| What is the minimum viable UEE for text? | ML, NLP |
| How do we handle conflicting Tier 1 rules? | Logic, planning |

---

## Getting Started

### 1. Read the docs first

Before writing any code, read these in order:

1. [README.md](../README.md) — what ELM is
2. [docs/ARCHITECTURE.md](ARCHITECTURE.md) — how it works
3. [docs/MEMORY.md](MEMORY.md) — the core data structures
4. [docs/MVP.md](MVP.md) — what we are building right now

### 2. Set up your environment

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup target add aarch64-unknown-none  # for bare-metal Pi target

# Clone the repo
git clone https://github.com/YOUR_ORG/elm
cd elm

# Build and run tests
cargo test
cargo run --example thermal_maze
```

### 3. Pick something to work on

Check the [Issues](https://github.com/YOUR_ORG/elm/issues) tab. Issues are labeled:

- `good-first-issue` — start here if you are new
- `help-wanted` — we need this and nobody owns it
- `research` — open question, discussion welcome
- `hardware` — requires physical hardware
- `documentation` — writing, diagrams, explanations

If you want to work on something that isn't an issue yet, open one first and describe what you are planning. This prevents duplication and gets you early feedback.

---

## How to Contribute

### For code contributions

```bash
# Fork the repo on GitHub, then:
git clone https://github.com/YOUR_USERNAME/elm
cd elm
git checkout -b your-feature-name

# Make your changes
# Add tests for new functionality
cargo test

# Commit with a clear message
git commit -m "Add cosine similarity engine with context weights"

# Push and open a Pull Request
git push origin your-feature-name
```

### Pull Request guidelines

- One concern per PR — keep it focused
- Tests must pass
- New functionality should have tests
- If you are adding a new data structure, add documentation comments
- If your change affects the spec, update the relevant `.md` file

### For documentation contributions

Documentation lives in the `docs/` folder as Markdown files. Same process — fork, branch, edit, PR.

If you find something confusing or missing in the docs, that is a bug. File an issue or fix it directly.

### For research contributions

Open a GitHub Discussion (not an Issue) for:
- Theoretical questions about the architecture
- Comparisons with existing research
- Proposed changes to the specification
- "What if we did X instead of Y" questions

Discussions are not tracked toward completion. They are where ideas live.

---

## Code Style

```rust
// Prefer explicit over clever
// Name things what they are

// Good
let compression_level = decide_quantization_level(delta, cluster_size);

// Not good  
let lvl = qnt(d, c);

// Document the why, not the what
// The what is visible in the code
// The why is not

/// Weights shift based on query context because the same two pockets
/// have different functional similarity depending on what we need.
/// Compression needs focal+action match. Investigation needs ambient match.
fn weighted_similarity(p1: &Pocket, p2: &Pocket, ctx: Context) -> f32 {
```

We use `rustfmt` for formatting. Run `cargo fmt` before committing.

We use `clippy` for lints. Run `cargo clippy` before committing.

---

## Community Standards

One rule: **be useful or be quiet**.

Useful means:
- Asking questions when you are stuck
- Answering questions when you know
- Giving specific feedback on PRs
- Documenting what you build
- Disagreeing with reasoning, not with people

This project is about building something that has never existed. That requires intellectual honesty, including saying "I don't know" and "I was wrong."

---

## License

MIT. Everything you contribute will be MIT licensed. Free for all, forever.

---

## Questions?

Open an Issue labeled `question`. Someone will answer.
