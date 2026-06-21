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

Here is an overview of the features currently supported in Bunzo v0.4.0:

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
- Variable redeclarations in the same scope are prohibited and will raise error `BZ1005`.
- Constants cannot be reassigned (error `BZ1002`).

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
print(10 > 5)    // true
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

### 5. `if` / `else` — Conditional Execution

Use `if` to branch on a boolean expression. The body must be enclosed in `{ }`:

```bunzo
let age = 20

if age >= 18 {
    print("You are an adult.")
} else {
    print("You are a minor.")
}
```

#### `else if` chaining

You can chain conditions using `else if`:

```bunzo
let score = 75

if score >= 90 {
    print("Grade: A")
} else if score >= 75 {
    print("Grade: B")
} else if score >= 60 {
    print("Grade: C")
} else {
    print("Grade: F")
}
```

#### Block scoping

Variables declared inside a block are **not** accessible outside it:

```bunzo
if true {
    let secret = 42
    print(secret) // ok: prints 42
}
// print(secret) // error BZ1001: undefined variable "secret"
```

Variables from outer scopes **are** readable inside a block:

```bunzo
let greeting = "Hello"

if true {
    print(greeting) // ok: prints Hello
}
```

> **Note:** The condition must evaluate to a boolean. Passing an integer, string,
> or null as a condition causes a runtime type error (`BZ1003`).

### 6. Loops — `while`, `for`, `break`, `continue`

Bunzo supports `while` and `for` loop control structures, along with `break` and `continue` statement primitives.

#### `while` Loop

A `while` loop repeatedly executes its body as long as the condition evaluates to `true`:

```bunzo
let count = 5
while count > 0 {
    print(count)
    count = count - 1
}
```

#### `for` Loop

A `for` loop iterates over a range of integer values using the `start..end` syntax (exclusive of `end`):

```bunzo
for i in 1..4 {
    print(i) // prints 1, 2, 3
}
```

- The loop variable (e.g., `i`) is automatically declared and scoped to the loop body. It cannot be re-declared inside the body using `let` or `const`.
- If `start >= end`, the loop executes zero times (e.g. `for i in 5..2 { }`).

#### `break` and `continue`

- `break` exits the nearest enclosing loop immediately.
- `continue` skips the rest of the current iteration and starts the next iteration of the nearest enclosing loop.

```bunzo
for i in 1..6 {
    if i == 3 {
        continue
    }
    if i == 5 {
        break
    }
    print(i) // prints 1, 2, 4
}
```

---

## Complete Example

Here is a program that combines all current features:

```bunzo
// Sum of odd numbers between 1 and 10 using a loop

let sum = 0
for i in 1..11 {
    let is_odd = i % 2 == 1
    if is_odd {
        sum = sum + i
    }
}
print(sum) // prints 25
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
