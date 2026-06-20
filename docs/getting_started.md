# Getting Started with Bunzo

Welcome to Bunzo! This guide will help you learn the basics of Bunzo, run your first program, and understand its core syntax.

---

## What is Bunzo?

Bunzo is a modern, developer-friendly programming language designed to reduce boilerplate and simplify application development. It focuses on excellent error diagnostics and zero configuration.

---

## Running a Bunzo Program

Bunzo programs are written in text files with the `.bz` extension. The `bzc` tool compiles or interprets these files directly.

To run a program:

1. Create a file named `hello.bz`.
2. Write the following code:
   ```bunzo
   print("Hello, Bunzo!")
   ```
3. Execute the program using the command-line interface:
   ```bash
   cargo run -p bzc -- run hello.bz
   ```
4. Output:
   ```text
   Reading hello.bz...

   Hello, Bunzo!
   ```

---

## Language Syntax

Here is an overview of the features currently supported in Bunzo v0.1.0 (Phase 4):

### 1. Comments

Bunzo supports single-line and multi-line comments:

```bunzo
// This is a single-line comment

/*
This is a
multi-line comment
*/
```

### 2. Variables and Constants

Use `let` to declare mutable variables, and `const` to declare read-only constants:

```bunzo
let name = "Alex"
const PI = 3.14159
```

- Variable names must start with a letter or underscore.
- Variable redeclarations in the same scope are prohibited and will raise an error.
- Constants cannot be reassigned.

### 3. Primitive Data Types

Bunzo is dynamically typed with the following primitive types:

- **Integer**: e.g., `42`, `-5`
- **Float**: e.g., `3.14`, `-0.75`
- **String**: e.g., `"Hello"`, `""`
- **Boolean**: `true` or `false`
- **Null**: `null`

### 4. Operators

#### Arithmetic

Bunzo supports standard arithmetic operations. If you mix an `Integer` and a `Float`, the `Integer` is automatically coerced to a `Float`:

- `+` (Addition / String Concatenation)
- `-` (Subtraction)
- `*` (Multiplication)
- `/` (Division)
- `%` (Modulo)

```bunzo
let a = 10
let b = 3
print(a / b) // Integer division -> prints 3
print(a % b) // Modulo -> prints 1

let c = 1.5
print(a + c) // Coerces a to float -> prints 11.5
```

#### Comparison

Comparison operators evaluate to a boolean (`true` or `false`):

- `==` (Equal)
- `!=` (Not Equal)
- `<` (Less than)
- `>` (Greater than)
- `<=` (Less than or equal)
- `>=` (Greater than or equal)

```bunzo
print(10 > 5)   // true
print(10 == 10.0) // true (coerced comparison)
```

#### Logical

Logical operators support lazy **short-circuit evaluation**:

- `&&` (Logical AND)
- `||` (Logical OR)
- `!` (Logical NOT)

```bunzo
let isAdult = true
let hasId = false
print(isAdult && hasId) // false
print(isAdult || hasId) // true
```

---

## Troubleshooting Errors

Bunzo is designed with **Excellent Error Messages** (Principle 5). If something goes wrong, you will receive a structured error with line, column, and a helpful hint.

For example, referencing an undefined variable:

```text
error[BZ1001]

Undefined variable "username"
  --> line 2, column 7

Hint: declare the variable using 'let' or 'const' before using it.
```
