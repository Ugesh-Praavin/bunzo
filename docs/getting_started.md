# Getting Started with Bunzo

Welcome to Bunzo! This guide will help you learn the basics of Bunzo, run your first program, and understand its core syntax, standard library, and tooling.

---

## What is Bunzo?

Bunzo is a modern, developer-friendly programming language designed to reduce boilerplate and simplify application development. It features a fast interpreter, a static type checker, automatic garbage collection, a package manager, an interactive REPL, and a VS Code companion extension.

---

## Installation

### Windows Installer (Recommended)
Bunzo provides a professional Windows installer (`bunzo-<version>-windows-x64-setup.exe`) that automatically installs the compiler, standard library, runtime context, and a **self-contained Clang/LLVM toolchain**. 

**Features:**
* **Zero Dependencies**: You do not need to install Microsoft Visual Studio, MinGW, or separate LLVM/Clang packages manually. Bunzo detects and uses its bundled toolchain out-of-the-box.
* **Automatic PATH Setup**: Installs to `C:\Program Files\Bunzo` and adds the binary directory (`C:\Program Files\Bunzo\bin`) to the system PATH.
* **Windows Integration**: Fully registers in Windows "Apps & Features" for tracking, display size computation, and uninstallation.

#### Silent Installation (for Package Managers like WinGet/Chocolatey/Scoop)
To perform a completely silent, non-interactive installation:
```powershell
bunzo-0.8.0-alpha-windows-x64-setup.exe /VERYSILENT /NORESTART /SUPPRESSMSGBOXES
```

#### Uninstallation
Uninstall Bunzo easily through the Windows Settings app ("Apps & Features") or run the uninstaller directly:
```powershell
"C:\Program Files\Bunzo\Uninstall Bunzo.exe" /VERYSILENT
```
Uninstallation will completely remove all Bunzo binaries, standard libraries, runtime, bundled Clang toolchain files, shortcuts, and PATH entries without touching your custom project files.

### Verifying the Installation
Once installed, open a new terminal window and run the following commands to verify:
```powershell
# 1. Verify compiler version and environment metadata
bzc --version

# 2. Create a new project hello
bzc new hello
cd hello

# 3. Compile the hello project using the bundled Clang compiler
bzc build

# 4. Run the compiled executable
.\hello.exe
```

---

## Interactive REPL

If you run `bzc` with no arguments in a terminal, it starts the interactive shell:
```text
Bunzo Interactive REPL v0.8.0-alpha
>>> let x = 42
>>> x + 8
50
```

---

## Language Syntax (v0.8.0-alpha)

### 1. Variables and Constants
* Mutable variables: `let x = 10`
* Read-only constants: `const PI = 3.14159`

### 2. Primitive Types
* **Integer**: e.g., `42`, `-5`
* **Float**: e.g., `3.14`, `-0.75`
* **String**: e.g., `"Hello"`
* **Boolean**: `true` or `false`
* **Null**: `null`

### 3. Control Flow
* `if` / `else if` / `else`:
  ```bunzo
  if x > 10 {
      print("high")
  } else {
      print("low")
  }
  ```
* `while` and `for` loops (with `break` and `continue`):
  ```bunzo
  for i in 1..5 {
      print(i) // prints 1, 2, 3, 4
  }
  ```

### 4. Functions
Declared using the `func` keyword:
```bunzo
func add(a: int, b: int) -> int {
    return a + b
}
print(add(2, 3)) // 5
```

### 5. Structs and Classes
* **Structs**: Lightweight plain data structures:
  ```bunzo
  struct Point {
      x: int
      y: int
  }
  let p = Point { x: 10, y: 20 }
  ```
* **Classes**: Supports OOP inheritance, interfaces, and `this` receiver bindings:
  ```bunzo
  class Animal {
      name: string
      func init(name: string) {
          this.name = name
      }
      func speak() {
          print("animal noise")
      }
  }
  ```

### 6. Error Handling
Bunzo supports structured exception handling via `try` / `catch` / `throw` blocks:
```bunzo
try {
    throw "something went wrong"
} catch err {
    print(err)
}
```

---

## Garbage Collection
Bunzo features automatic tracing Garbage Collection. The runtime tracks allocations and periodically reclaims unreachable memory, preventing memory leaks during long-running tasks.

---

## Standard Library Modules
Import built-in standard library utilities:
```bunzo
import vector
import math
import path

let v = vector.new()
vector.push(v, math.sqrt(100))
print(vector.get(v, 0)) // 10
```

---

## Developer Tooling

### Package Manager
Manage project dependencies inside `bunzo.toml` fetched from Git repositories:
* `bzc install`
* `bzc add <package_name> <git_url>`
* `bzc remove <package_name>`
* `bzc update`

### Formatter & Linter
* Autoformat code: `bzc fmt <file_or_dir>`
* Analyze style and quality issues: `bzc lint <file_or_dir>`

### VS Code Extension
Bunzo bundles a language server client inside `editors/vscode`. It boots the Bunzo Language Server (`bzc lsp`) to provide syntax highlighting, diagnostics, hover definitions, and go-to definition.
