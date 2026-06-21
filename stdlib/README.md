# Bunzo Standard Library

Built-in modules available via `import <name>`:

| Module | Description |
|--------|-------------|
| `http` | HTTP client — `get`, `post`, `put`, `delete`, `patch` |
| `json` | JSON `encode` / `decode` |
| `math` | `sqrt`, `abs`, `sin`, `cos`, `pow`, `PI`, `E`, … |
| `os` | `args`, `env`, `exit` |
| `db` | Embedded SQL — `open`, `execute`, `query`, `close` |

Implementation lives in `compiler/src/stdlib/` (Rust). User-defined modules can be placed here as `.bz` files and are resolved after `./name.bz`.

## HTTP

```bz
import http

let body = http.get("http://example.com/")
let created = http.post("http://example.com/api", "{\"ok\":true}")
http.put(url, body)
http.patch(url, body)
http.delete(url)
```

## Database

```bz
import db

let conn = db.open("app.db")
db.execute(conn, "CREATE TABLE IF NOT EXISTS users (id INTEGER, name TEXT)")
db.execute(conn, "INSERT INTO users VALUES (1, 'Bart')")
let rows = db.query(conn, "SELECT id, name FROM users")
db.close(conn)
```

Use `:memory:` for an in-memory database.
