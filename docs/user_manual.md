# Bunzo User Manual

Welcome to the official Bunzo User Manual! Bunzo is a modern, developer-friendly programming language designed for simplicity, performance, and joy. It features a fast interpreter, static type checking, automatic garbage collection, and a powerful standard library.

This guide covers everything you need to know to read, write, and execute Bunzo code.

---

## 1. Installation & Tooling

Bunzo comes with an all-in-one compiler and toolchain called `bzc`.

### Using the CLI (`bzc`)

The `bzc` command-line interface provides several subcommands for managing your code.

- **Run a program**:
  ```bash
  bzc run my_script.bz
  ```
  Executes the specified Bunzo file.

- **Interactive REPL**:
  ```bash
  bzc
  ```
  Running the CLI without any arguments starts an interactive Read-Eval-Print Loop. You can type Bunzo code line-by-line and see immediate results.

- **Code Formatter**:
  ```bash
  bzc fmt .
  ```
  Automatically formats your Bunzo source code to match the standard style guidelines.

- **Linter**:
  ```bash
  bzc lint src/
  ```
  Analyzes your code for potential errors, anti-patterns, and styling issues.

- **Package Manager**:
  Manage third-party dependencies natively via a `bunzo.toml` file.
  - `bzc install`: Installs dependencies listed in `bunzo.toml`.
  - `bzc add <package_name> <git_url>`: Adds a new dependency.
  - `bzc remove <package_name>`: Removes a dependency.
  - `bzc update`: Updates all dependencies to their latest versions.

- **Language Server**:
  ```bash
  bzc lsp
  ```
  Starts the Language Server Protocol (LSP) daemon over standard I/O (used by editor extensions).

---

## 2. Basic Syntax & Types

Bunzo files use the `.bz` extension. The language is dynamically expressive but optionally statically typed. Statements do not require trailing semicolons.

### Variables and Constants

Use `let` for mutable variables and `const` for immutable variables.

```bunzo
// Mutable variable
let count = 10
count = count + 1

// Immutable constant
const PI = 3.14159
// PI = 3.0 // This will throw a compile-time error!
```

### Primitive Types

Bunzo supports standard primitive types out of the box:

```bunzo
let name: string = "Bunzo"     // Strings (UTF-8)
let age: int = 42              // 64-bit integer
let score: float = 99.5        // 64-bit floating point
let is_active: bool = true     // Boolean (true/false)
let empty: null = null         // Null reference
```

### Comments

```bunzo
// This is a single-line comment

/*
This is a
multi-line comment
*/
```

---

## 3. Control Flow

### If / Else Conditionals

```bunzo
let temp = 25

if temp > 30 {
    print("It's hot!")
} else if temp < 15 {
    print("It's cold!")
} else {
    print("Perfect weather.")
}
```

### Loops

**While Loops**:
```bunzo
let n = 5
while n > 0 {
    print(n)
    n = n - 1
}
```

**For-In Range Loops**:
```bunzo
// Iterate from 1 up to (and including) 5
for i in 1..5 {
    if i == 3 {
        continue // Skip 3
    }
    print(i)
}
```

You can also use `break` to exit loops early.

---

## 4. Functions

Functions in Bunzo are declared using the `func` keyword. You can optionally specify argument types and return types for the type checker.

```bunzo
func add(a: int, b: int) -> int {
    return a + b
}

let result = add(10, 20)
print(result) // Outputs: 30
```

---

## 5. Object-Oriented Programming

Bunzo provides both lightweight data structs and fully-featured classes.

### Structs

Structs are plain data containers without methods.

```bunzo
struct Point {
    x: int
    y: int
}

let p = Point { x: 10, y: 20 }
print(p.x)
```

### Classes and Methods

Classes support methods, constructors (`init`), and `this` binding.

```bunzo
class Animal {
    name: string

    func init(name: string) {
        this.name = name
    }

    func speak() {
        print(this.name + " makes a sound.")
    }
}

let dog = Animal("Buddy")
dog.speak()
```

### Inheritance

You can inherit behaviors and properties using the `<` symbol.

```bunzo
class Dog < Animal {
    func speak() {
        print(this.name + " barks!")
    }
}
```

---

## 6. Error Handling

Bunzo uses a structured `try / catch` mechanism instead of return-code-based errors. You can throw exceptions using the `throw` keyword.

```bunzo
func divide(a, b) {
    if b == 0 {
        throw "Division by zero is not allowed!"
    }
    return a / b
}

try {
    let ans = divide(10, 0)
    print(ans)
} catch err {
    print("An error occurred: " + err)
}
```

---

## 7. Standard Library

Bunzo ships with a massive built-in standard library. You import modules using the `import` keyword.

### Examples

**Working with Collections (Vectors)**:
```bunzo
import vector

let v = vector.new()
vector.push(v, 100)
vector.push(v, 200)
print(vector.len(v)) // 2
```

**Filesystem I/O**:
```bunzo
import filesystem

filesystem.write_string("hello.txt", "Hello from Bunzo!")
let content = filesystem.read_string("hello.txt")
print(content)
```

**Concurrency (Threads)**:
```bunzo
import thread
import time

func worker() {
    print("Background work...")
    time.sleep(1000)
}

let t = thread.spawn(worker)
thread.join(t)
```

Other available modules include: `math`, `string`, `path`, `random`, `crypto`, `encoding`, `process`, `environment`, `io`, `networking` (TCP/UDP), `regex`, `algorithm`, and more!

---

## 8. Editor Integration (VS Code)

To get the best experience, we strongly recommend using the official Bunzo VS Code Extension. It leverages the built-in `bzc lsp` server to provide:
- Syntax highlighting
- Real-time error diagnostics and linting
- Hover documentation
- Go-to definition

**Installation**: The extension source code is available in the `editors/vscode` directory. Simply compile the extension using `npm install && npm run compile` and load it into VS Code.

---

## Summary

You are now ready to build fast and reliable software in Bunzo! 
Start a new project, write a `.bz` file, and execute it using `bzc run`. Enjoy coding!
