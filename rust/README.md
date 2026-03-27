# Task Tracker CLI (Rust)

Misma idea que el CLI en Go en la raíz de este repo: tareas en `tasks.json` en el **directorio de trabajo actual** (donde ejecutes el binario). Solo biblioteca estándar de Rust.

## Requisitos

- [Rust](https://www.rust-lang.org/tools/install) (edición 2021+)

## Compilar y usar

Desde esta carpeta `rust/`:

```bash
cd rust
cargo build --release
cargo run -- add "Comprar leche"
cargo run -- list
```

El binario compilado: `target/release/task-tracker`.

## Comandos

```bash
cargo run -- add "descripción"
cargo run -- update <id> "nueva descripción"
cargo run -- delete <id>
cargo run -- mark-in-progress <id>
cargo run -- mark-done <id>
cargo run -- list
cargo run -- list todo
cargo run -- list in-progress
cargo run -- list done
cargo run -- list not-done
```

Detalles de campos, validación y errores: ver el README en la raíz del repositorio.
