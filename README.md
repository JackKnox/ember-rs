# `ember-rs` 🔥
A safe, ergonomic Rust interface for **The Ember Protocol**.

## Features
* 🦀 Idiomatic Rust API.
* 🔒 Safe abstractions over unsafe FFI.
* 🔄 Automatically updates bindings when a new release is published.
* 📦 Workspace containing both low-level and high-level crates.

## Getting Started

Clone the repository:

```bash
git clone https://github.com/JackKnox/ember-rs.git
cd ember-rs
```

Build the workspace and run the test suite:

```bash
cargo build

cargo test
```

Or simply add it to your Cargo project:

```bash
cargo add ember-rs
```

## Regenerating Bindings
To update the FFI bindings to the very latest commit on the main repository, run:

```bash
./scripts/regenerate.sh
```

> [!NOTE]
> This will not update the idiomatic Rust layer, so you will have to use the FFI layer directly.

## Automation
The repository includes a GitHub Actions workflow that can:

* Check for updates to the upstream C repository.
* Regenerate the FFI bindings.
* Run formatting and tests.
* Open or update a pull request containing the generated changes.

This keeps the raw bindings synchronized while allowing the high-level API to evolve through normal code review.

## Contributing
Contributions are welcome.

If the upstream C protocol introduces new functions, the automated workflow will regenerate the bindings and create a pull request. 
Contributors can then review the generated changes and implement any required high-level wrappers.

Please ensure all changes pass:

```bash
cargo fmt
cargo clippy --all-targets --all-features
cargo test
```

before opening a pull request.

## License
`ember-rs` is licensed under the [MIT License](LICENSE).

