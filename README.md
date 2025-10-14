<p align="center">
  <img src="./extras/arto-header.png" alt="Arto" />
</p>

**Arto — the Art of Reading Markdown.**

A local app that faithfully recreates GitHub-style Markdown rendering for a beautiful reading experience.

## Philosophy

Markdown has become more than a lightweight markup language — it's the medium for documentation, communication, and thinking in the developer's world. While most tools focus on _writing_ Markdown, **Arto is designed for _reading_ it beautifully**.

The name "Arto" comes from "Art of Reading" — reflecting the philosophy that reading Markdown is not just a utility task, but a quiet, deliberate act of understanding and appreciation.

Arto faithfully reproduces GitHub's Markdown rendering in a local, offline environment, offering a calm and precise reading experience with thoughtful typography and balanced whitespace.

> [!WARNING] > **Beta Software Notice**
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

## Installation

### From Source

```bash
git clone https://github.com/lambdalisue/rs-arto.git
cd rs-arto
cargo build --release
```

The binary will be available at `target/release/arto`.

## Usage

Launch the application to see the welcome screen with keyboard shortcuts and usage instructions.

## Development

### Prerequisites

- Rust 1.70 or higher
- Cargo

### Running in Development

```bash
cargo run
```

### Building for Production

```bash
cargo build --release
```

## License

See [LICENSE](LICENSE) file for details.
