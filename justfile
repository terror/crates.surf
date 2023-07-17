export RUST_LOG := 'info'

default:
  just --list

all: build test clippy fmt-check

build:
  cargo build

clippy:
  cargo clippy --all-targets --all-features

dev *args: services
  concurrently \
    --kill-others \
    --names 'SERVER,CLIENT' \
    --prefix-colors 'green.bold,magenta.bold' \
    --prefix '[{name}] ' \
    --prefix-length 2 \
    --success first \
    --handle-input \
    --timestamp-format 'HH:mm:ss' \
    --color \
    -- \
    'just watch run serve --db-name=crates {{args}}' \
    'pnpm run dev'

fmt:
  cargo fmt
  npm run format

fmt-check:
  cargo fmt --all -- --check
  @echo formatting check done

restart-services:
  docker compose down --volumes && just services

run *args:
  cargo run -- {{args}}

services:
  docker compose up -d

test:
  cargo test

watch +COMMAND='test':
  cargo watch --clear --exec "{{COMMAND}}"
