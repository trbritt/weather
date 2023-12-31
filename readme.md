## Comparing http services between Golang and Rust

The goal of this little repo is to compare the development experience between simple servers in Golang and Rust, in the hopes I figure
out which I want to seriously consider for which projects in the near future. 

Not for production. 

These programs can be run with `go run` or `cargo run` from the respective directories, and require the existence of a `postgresSQL` server + database running on the local host. In this example, the `postgres` user, password, and database name are all `forecast`, and the code replies on the definition of the following environment variable:

```bash
export DATABASE_URL="postgres://forecast:forecast@localhost/forecast?sslmode=disable"
```

Current efforts focus on putting `tailwindcss` and `htmx` onto the frontend. Simple startup looks like
```bash
pnpm init
pnpm add -D tailwindcss prettier prettier-plugin-tailwindcss
```
and to be really sleek, we serve our static assets (namely the style sheet) in a compressed optimized `css` form that is autogenerated on changes to any `html` using `pnpm dlx tailwindcss -i styles/tailwind.css -o assets/main.css --watch`. Very cool. 

For right now, the `htmx` elements only serve a dummy api backend on the Rust server. 

Hmm, can a gopher even eat a crustacean? Can a crustacean take down a gopher? Who knows.