## Comparing http services between Golang and Rust

The goal of this little repo is to compare the development experience between simple servers in Golang and Rust, in the hopes I figure
out which I want to seriously consider for which projects in the near future. 

Not for production. Inspired by [this blog post](https://www.shuttle.rs/blog/2023/09/27/rust-vs-go-comparison). 

These programs can be run with `go run` or `cargo run` from the respective directories, and requier the existence of a `postgresSQL` server + databaes running on the local host. In this example, the `postgres` user, password, and database name are all `forecast`, and the code replies on the definition of the following environment variable:

```bash
export DATABASE_URL="postgres://forecast:forecast@localhost/forecast?sslmode=disable"
```

Hmm, can a gopher even eat a crustacean? Can a crustacean take down a gopher? Who knows.