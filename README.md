# Task Tracker CLI

Implementaciones en **Go** y **Rust** para gestionar las mismas tareas; persisten en `tasks.json` en el directorio desde el que ejecutes el programa.

## Versión Go (raíz)

CLI en **Go**. Requisito: [Go](https://go.dev/dl/) instalado.

```bash
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
```

Compilar binario:

```bash
go build -o task-cli task-cli.go
./task-cli list
```

- Solo biblioteca estándar de Go.

## Versión Rust (`rust/`)

CLI equivalente con **solo `std`**. Ver [rust/README.md](rust/README.md).

```bash
cd rust
cargo run -- add "Comprar leche"
cargo run -- list
```

## Formato de `tasks.json`

El archivo se crea si no existe. Ejemplo:

```json
[
  {
    "id": 1,
    "description": "Buy groceries",
    "status": "todo",
    "createdAt": "2026-03-27T10:00:00Z",
    "updatedAt": "2026-03-27T10:00:00Z"
  }
]
```

- Estados: `todo`, `in-progress`, `done`.
- El filtro `not-done` muestra todo lo que no está `done`.
- En Rust los timestamps se guardan en ISO 8601 UTC (segundos, sufijo `Z`).

## Notas

- Errores de comando, argumentos e IDs inválidos: mensajes claros en stderr y código de salida distinto de cero (en ambas implementaciones).
