# Bunzo Language Support for VS Code

This extension provides rich language support for the **Bunzo** programming language in Visual Studio Code.

## Features

- **Syntax Highlighting**: Complete syntax coloring for Bunzo keywords, operators, types, and literals.
- **Code Completion (IntelliSense)**: Autocomplete suggestions for keywords, types, built-ins, and code templates (functions, loops, conditionals).
- **Diagnostics (Error Checking)**: Highlights basic syntax issues (like mismatched brackets and constant variable reassignments).
- **Hover Information**: Tooltips showing documentation for language keywords and built-in functions.
- **External Compiler Server Integration**: Option to run the high-performance compiler-based LSP server (`bzc lsp`) as the backend.

## Installation

1. Clone this repository or copy the `vscode-bunzo` directory to your VS Code extensions folder (`~/.vscode/extensions/` on Linux/macOS, `%USERPROFILE%\.vscode\extensions\` on Windows).
2. Open the directory in a terminal and run `npm install` to install dependencies.
3. Run `npm run compile` to build the extension.
4. Reload/restart VS Code.

## Configuration

This extension contributes the following settings:

* `bunzo.compilerPath`: Specify the absolute path to the `bzc` compiler binary. Defaults to `"bzc"`.
* `bunzo.useCompilerLsp`: When set to `true`, the extension will launch the native compiler-based LSP server (`bzc lsp`) instead of the default Node-based server. Defaults to `false`.

## License

MIT
