bookdb reference implementation 0.4.0

- strict FQCC with required @|% prefix and VAR|DOC anchor
- install seeds HOME with GLOBAL.DOC.MAIN and GLOBAL.VAR.MAIN
- cursor files: ~/.config/bookdb/cursor.base + cursor.chain
- commands: getv/setv/getd/setd/ls/import(export kv/jsonl limited)/migrate noop

Build:
  cargo build

Install:
  cargo run -- install --force

Example:
  book setv USERNAME=bob @home.GLOBAL.VAR.MAIN
  book getv USERNAME @home.GLOBAL.VAR.MAIN
  book setd profile._root="Hello" @home.GLOBAL.DOC.MAIN
  book getd profile._root @home.GLOBAL.DOC.MAIN
