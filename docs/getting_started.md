# Getting Started with Bunzo

Welcome to Bunzo! This guide will help you learn the basics of Bunzo, run your first program, and understand its core syntax, standard library, and tooling.

---

## What is Bunzo?

Bunzo is a modern, developer-friendly programming language designed to reduce boilerplate and simplify application development. It features a fast interpreter, a static type checker, automatic garbage collection, a package manager, an interactive REPL, and a VS Code companion extension.

---

## Installation & Running a Bunzo Program

Bunzo programs are written in text files with the `.bz` extension. The `bzc` tool compiles or interprets these files.

To run a program:

1. Create a file named `hello.bz`.
2. Write the following code:
   ```bunzo
   print("Hello, Bunzo!")
   ```
3. Execute the program using the compiler CLI:
   ```bash
   cargo run -p bzc -- run hello.bz
   ```
4. Output:
   ```text
   Reading hello.bz...

   Hello, Bunzo!
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
