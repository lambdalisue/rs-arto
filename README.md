<p align="center">
  <img src="./extras/arto-header.png" alt="Arto" />
</p>

**Arto — the Art of Reading Markdown.**

A local app that faithfully recreates GitHub-style Markdown rendering for a beautiful reading experience.

## Philosophy

Markdown has become more than a lightweight markup language — it's the medium for documentation, communication, and thinking in the developer's world. While most tools focus on _writing_ Markdown, **Arto is designed for _reading_ it beautifully**.

The name "Arto" comes from "Art of Reading" — reflecting the philosophy that reading Markdown is not just a utility task, but a quiet, deliberate act of understanding and appreciation.

Arto faithfully reproduces GitHub's Markdown rendering in a local, offline environment, offering a calm and precise reading experience with thoughtful typography and balanced whitespace.

> [!WARNING]
> **Beta Software Notice**
>
> - This application is still in **beta** and may contain bugs or unstable behavior. Features may change without regard to backward compatibility.
> - **macOS Only Testing**: While the underlying libraries support cross-platform development, this app has only been tested on macOS by the author. It may not work properly on other platforms, especially regarding window management (which differs significantly across operating systems). Cross-platform support is a goal, and **PRs are welcome**.

## Features

- **GitHub-Style Rendering**: Accurate reproduction of GitHub's Markdown styling with full support for extended syntax
- **Native Performance**: Built with Rust for fast, responsive rendering
- **File Explorer**: Built-in sidebar with file tree navigation for browsing local directories
- **Tab Support**: Open and manage multiple documents in tabs within a single window
- **Multi-Window**: Create multiple windows and open child windows for diagrams
- **Auto-Reload**: Automatically updates when the file changes on disk
- **Dark Mode**: Manual and automatic theme switching based on system preferences
- **Advanced Rendering**: Support for Mermaid diagrams, math expressions (KaTeX), code syntax highlighting, and more
- **Mermaid Window**: View Mermaid diagrams in a separate, interactive window with zoom and pan controls
- **Code Block Features**: Copy button for code blocks, copy Mermaid source as image
- **Drag & Drop**: Simply drag markdown files onto the window to open them
- **Live Navigation**: Navigate between linked markdown documents with history support (back/forward)
- **Offline First**: No internet connection required — read your docs anytime, anywhere

## Screenshots
<img width="1610" height="1443" alt="CleanShot 2025-10-26 at 16 24 54" src="https://github.com/user-attachments/assets/675b9a2f-9e1d-4355-bd05-f475b77e62e6" />
<img width="1610" height="1443" alt="CleanShot 2025-10-26 at 16 25 13" src="https://github.com/user-attachments/assets/5012a3ce-9741-4b2e-9fb2-e8c5868638f8" />
<img width="1610" height="1443" alt="CleanShot 2025-10-26 at 16 25 31" src="https://github.com/user-attachments/assets/84fc6c60-4166-4330-a83b-370d5e2fa534" />

## Installation

- [pnpm](https://pnpm.io/)
- [Rust](https://rust-lang.org/)
- [just](https://github.com/casey/just)
- [dioxus-cli](https://crates.io/crates/dioxus-cli)

### From Source

```bash
git clone https://github.com/lambdalisue/rs-arto.git
cd rs-arto
just setup

# For development
cargo run --release

# For production build (macOS)
just build

# To install in /Applications (macOS)
just install
```

The binary will be available at `target/release/arto` or `target/dx/arto/bundle/macos/bundle/`.

## Usage

Launch the application to see the welcome screen with keyboard shortcuts and usage instructions.

## License

See [LICENSE](LICENSE) file for details.
