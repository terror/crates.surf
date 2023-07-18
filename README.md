## crates.surf 🏄

**crates.surf** is a full-text/semantic search engine for all 100k+ crates in
the rust ecosystem.

![](https://github.com/terror/crates.surf/assets/31192478/06bfb588-74a2-4cab-8b6e-3056bb669fb9)

### Development

You'll need [docker](https://www.docker.com/),
[cargo](https://doc.rust-lang.org/cargo/) and [pnpm](https://pnpm.io/) installed
on your machine in order to get the project running locally.

First, mount local [postgres](https://www.postgresql.org/),
[elasticsearch](https://www.elastic.co/?ultron=B-Stack-Trials-AMER-CA-Exact&gambit=Stack-Core&blade=adwords-s&hulk=paid&Device=c&thor=elasticsearch)
and
[rabbitmq](https://www.cloudamqp.com/blog/part1-rabbitmq-for-beginners-what-is-rabbitmq.html)
instances with docker:

```bash
docker compose up -d
```

Spawn the server with a database name:

```bash
RUST_LOG=info cargo run serve --db-name=crates
```

Finally, spawn the svelte frontend:

```bash
pnpm run dev
```

By default, the server is listening on `http://localhost:8000` and the frontend
over at `http://localhost:5173`.
