## Rock HTTP Library

- this repository contains classes and functions for rockhttp project

link: https://github.com/krishpranav/rockhttp

# Using rockhttp_lib in your project:

## Installation:
```rust
rockhttp_lib = { git = "https://github.com/krishpranav/rockhttp_lib", version = "0.1.0", features = ["reload", "https"] }
```

```rust
extern crate rockhttp_lib;

fn main() {
    rockhttp_lib::run(&"localhost", 8080, "", true ); 
    /* server will run on http://localhost:8080/ */
}

```