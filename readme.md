## Comparing http services between Golang and Rust

The goal of this little repo is to compare the development experience between simple servers in Golang and Rust.

Not for production. Inspired by [this blog post](https://www.shuttle.rs/blog/2023/09/27/rust-vs-go-comparison). 

The postgres db on local is accessed via 
```bash
export DATABASE_URL="postgres://forecast:forecast@localhost/forecast?sslmode=disable"
```