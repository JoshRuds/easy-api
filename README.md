# easy-api

A crate that provides a easy api interface without the boilerplate.

## Installation
 
```bash
cargo add easy-api
```

## Example Usage

```rust
use easy_api;

fn hello() {
    println!("Hello!");
}

fn this_is_a_grab(input: String) {
    println!("I got a {}", input);
}

fn main() {
    // Adds a new api which calls hello() upon /hello being tapped.
    easy_api::add_tap_api("/hello", hello); 

    // Adds a new api which calls this_is_a_grab() upon /grab being grabbed.
    // Note: Callback Function must take in one string.
    easy_api::add_grab_api("/grab", this_is_a_grab);
    
    // Starts the server with the configured settings on 127.0.0.1:3000
    // Use start_on(address: string) to use a different host and/or port 
    easy_api::start(); 

    // Keeps theserver alive. Feel free to use other things in this loop.
    loop {
        easy_api::park(); 
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

## Requests

- For a *Tap Api*, all body and headers will be ignored
- For a *Grab Api*, all headers will be ignored, and the body must be a **plain text** value, interpreted as a string.

## Responses

easy-api always replies with code 200 OK when a callback function was found, else 404 NOT FOUND.

## Notes

easy-api is currently **under development**. Things may not work or may work incorrectly.