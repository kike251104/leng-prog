# Task Tracker CLI

Task tracker implementations in:
- Go (`task-cli.go`)
- Rust (`rust/src/main.rs`)

Both versions store tasks in `tasks.json` in the current directory.

## Requirements

- Go (to run `task-cli.go`)
- Rust + Cargo (to run `rust/`)

## Commands

```bash
# Go
go run task-cli.go add "Buy groceries"
go run task-cli.go update 1 "Buy groceries and cook dinner"
go run task-cli.go delete 1
go run task-cli.go mark-in-progress 1
go run task-cli.go mark-done 1
go run task-cli.go list
go run task-cli.go list todo
go run task-cli.go list in-progress
go run task-cli.go list done
go run task-cli.go list not-done

# Rust
cd rust
cargo run -- add "Buy groceries"
cargo run -- update 1 "Buy groceries and cook dinner"
cargo run -- delete 1
cargo run -- mark-in-progress 1
cargo run -- mark-done 1
cargo run -- list
cargo run -- list todo
cargo run -- list in-progress
cargo run -- list done
cargo run -- list not-done
cd ..
```

## Task Format

Tasks are stored in `tasks.json` (auto-created in the current directory) with this structure:

```json
[
  {
    "id": 1,
    "description": "Buy groceries",
    "status": "todo",
    "createdAt": "2026-03-27T10:00:00.000Z",
    "updatedAt": "2026-03-27T10:00:00.000Z"
  }
]
```

## Notes

- Valid statuses: `todo`, `in-progress`, `done`, `not-done`
- Handles invalid commands, missing arguments, and invalid IDs gracefully
- Uses only standard libraries (no external crates/packages)
