# Task Tracker CLI

CLI en **Go** para gestionar tareas. Persisten en `tasks.json` en el directorio actual.

## Requisitos

- [Go](https://go.dev/dl/) instalado

## Uso

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

Opcional: compilar un binario:

```bash
go build -o task-cli task-cli.go
./task-cli list
```

## Formato de las tareas

`tasks.json` (se crea solo si no existe):

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

## Notas

- Estados: `todo`, `in-progress`, `done`; el filtro `not-done` muestra todo lo que no está `done`.
- Solo biblioteca estándar de Go.
- Errores de comando, argumentos e IDs inválidos se manejan con mensajes claros.
