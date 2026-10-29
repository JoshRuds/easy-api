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

fn greet(input: String) -> String {
    format!("Hello, {input}!")
}

fn whats_my_name() -> String {
    "My name is Bob!".to_string()
}

fn main() {
    // Adds a new api which calls hello() upon /hello being tapped.
    easy_api::add_tap_api("/hello", hello); 

    // Adds a new api which calls this_is_a_grab() upon /grab being given data.
    // Note: Callback Function must take in one string.
    easy_api::add_grab_api("/grab", this_is_a_grab);
    
    // Adds a new api which returns the result of whats_my_name() upon /name being tapped.
    easy_api::add_tap_give_api("/name", hello); 

    // Adds a new api which returns the result of greet() upon /greet being given data.
    // Note: Callback Function must take in one string.
    easy_api::add_grab_api("/greet", this_is_a_grab);
    
    // Starts the server with the configured settings on 127.0.0.1:3000
    // Use start_on(address: string) to use a different host and/or port.
    // Use start_on_any() to use any available port.
    easy_api::start(); 

    // Keeps the server alive. Feel free to use other things in this loop.
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

- For a *Tap Api*, all body and headers will be ignored. The request must be a **GET** request.
- For a *Grab Api*, all headers will be ignored, and the body must be a **plain text** value, interpreted as a string. The request must be a **POST** request.
- For the *Grab Give Api* and the *Tap Give Api*, the above is true, and the result will be returned as plain text in the responses' bodies.

## Responses

easy-api always replies with code 200 OK when a callback function was found, and 404 NOT FOUND if not. If a incorrect method was used, it will return 405 METHOD NOT ALLOWED.

## Notes

easy-api is currently **under development**. Things may not work or may work incorrectly.