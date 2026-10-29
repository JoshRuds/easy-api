# easy-api

A crate that provides an easy API interface without the boilerplate.

## Installation

```bash
cargo add easy-api
```

## Example Usage

```rust
use easy_api;

fn hello() -> easy_api::EasyResult {
    println!("Hello!");
    Ok(())
}

fn receive(input: String) -> easy_api::EasyResult {
    println!("I got: {input}");
    Ok(())
}

fn greet(input: String) -> easy_api::EasyResultOutput {
    Ok(format!("Hello, {input}!"))
}

fn whats_my_name() -> easy_api::EasyResultOutput {
    Ok("My name is Bob!".to_string())
}

fn main() {
    // Calls hello() when /hello is requested.
    easy_api::add_tap_api("/hello", hello);

    // Calls receive() with the request body.
    easy_api::add_grab_api("/grab", receive);

    // Returns the result of whats_my_name() from /name.
    easy_api::add_tap_give_api("/name", whats_my_name);

    // Returns the result of greet() from /greet.
    easy_api::add_grab_give_api("/greet", greet);

    // Starts the server on 127.0.0.1:3000.
    // Use start_on(address) for a custom address.
    // Use start_on_any() to use any available port.
    easy_api::start();

    // Keeps the server alive.
    loop {
        easy_api::park();
    }
}
```

## Registering Closures

You can register closures instead of named functions:

```rust
easy_api::add_tap_api("/hello", || {
    println!("Hello!");
    Ok(())
});

easy_api::add_grab_give_api("/greet", |input| {
    Ok(format!("Hello, {input}!"))
});
```

## Requests

- **Tap API:** Ignores the request body and headers. Requires a `GET` request.
- **Grab API:** Receives the request body as a `String`. Headers are ignored. Requires a `POST` request.
- **Tap Give API:** Works like a Tap API but returns a `String` in the response body.
- **Grab Give API:** Works like a Grab API but returns a `String` in the response body.

## Error Handling

All callback functions must return a `Result`.

- `EasyResult` is used for callbacks that do not return output.
- `EasyResultOutput` is used for callbacks that return a `String`.
- `Ok(())` indicates success for callbacks without output.
- `Ok("message".to_string())` returns a successful response with the specified text.
- `Err(status_code)` indicates an error. The `u16` value specifies the HTTP status code to return.

For example, you can return `Err(400)` if the request contains invalid input:

```rust
easy_api::add_grab_give_api("/greet", |input| {
    if input.is_empty() {
        return Err(400);
    }

    Ok(format!("Hello, {input}!"))
});
```

## Responses

- **200 OK:** The callback succeeds.
- **400 Bad Request:** Can be returned by a callback to indicate invalid input.
- **404 Not Found:** No callback is registered for the requested path.
- **405 Method Not Allowed:** The request uses the wrong HTTP method.
- **Other status codes:** Callbacks can return an error with any valid HTTP status code.

When a callback returns an error, the response body contains the standard message for that HTTP status code, such as `Bad Request` or `Internal Server Error`.

Successful callbacks that return output send that output as plain text. Successful callbacks without output return `OK`.

## Starting the Server

- `start()` starts the server at `127.0.0.1:3000`.
- `start_on(address)` starts the server at the specified address and returns the address it successfully bound to.
- `start_on_any()` starts the server on an available port on `127.0.0.1` and returns the address.

## Notes

easy-api is currently **under development**. Things may not work or may work incorrectly.