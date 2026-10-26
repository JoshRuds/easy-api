# simple-api

A crate that provides a simple api interface without the boilerplate.

## Installation
 
```bash
cargo add simple-api
```

# Usage

```rust
use simple_api;

fn hello() {
    println!("Hello!");
}

fn main() {
    // Adds a new api which calls hello() upon /hello being tapped.
    simple_api::add_tap_api("/hello", hello); 
    
    // Starts the server with the configured settings.
    simple_api::start(); 

    // Keeps server alive. Feel free to use other things in this loop.
    loop {
        simple_api::park(); 
    }
}
```

## Registering closures

You can also register closures instead of named functions:

```rust
add_tap_api("/hello", || {
    println!("Hello!");
});
```

## Responses

simple-api always replies with code 200 OK when a callback function was found, else 404 NOT FOUND.

## Notes

simple-api is currently under development. Things may not work or may work incorrectly.