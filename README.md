# easy-api

A Rust crate that provides a simple HTTP API interface without the boilerplate.

`easy-api` lets you register callbacks for HTTP endpoints and start a server with minimal setup. You can also enable optional static file hosting to serve HTML, CSS, JavaScript, and other files alongside your API.

## Installation

Add `easy-api` to your project:

```bash
cargo add easy-api
```

### Enabling Static File Hosting

Static file hosting is an optional feature. To install `easy-api` with it enabled, run:

```bash
cargo add easy-api --features static
```

Alternatively, enable it in your `Cargo.toml`:

```toml
[dependencies]
easy-api = { version = "YOUR_VERSION", features = ["static"] }
```

Replace `YOUR_VERSION` with the version you want to use.

Without the `static` feature, you can still use all the standard API functionality.

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
    // Register endpoints.
    easy_api::add_tap_api("/hello", hello);
    easy_api::add_grab_api("/grab", receive);
    easy_api::add_tap_give_api("/name", whats_my_name);
    easy_api::add_grab_give_api("/greet", greet);

    // Start the server on 127.0.0.1:3000.
    easy_api::start();

    // Keep the main thread alive.
    loop {
        easy_api::park();
    }
}
```

## Registering Callbacks

`easy-api` supports four callback types.

### 1. Tap API

Registers a callback that takes no input and returns no output.

**HTTP method:** `GET`

```rust
easy_api::add_tap_api("/hello", || {
    println!("Hello!");
    Ok(())
});
```

A successful request returns `200 OK` with the response body `OK`.

### 2. Grab API

Registers a callback that receives the request body as a `String` and returns no output.

**HTTP method:** `POST`

```rust
easy_api::add_grab_api("/grab", |input| {
    println!("Received: {input}");
    Ok(())
});
```

The request body is passed directly to the callback. Request headers are not provided to the callback.

A successful request returns `200 OK` with the response body `OK`.

### 3. Tap Give API

Registers a callback that takes no input and returns a `String`.

**HTTP method:** `GET`

```rust
easy_api::add_tap_give_api("/name", || {
    Ok("My name is Bob!".to_string())
});
```

The returned string becomes the response body.

### 4. Grab Give API

Registers a callback that receives the request body as a `String` and returns a `String`.

**HTTP method:** `POST`

```rust
easy_api::add_grab_give_api("/greet", |input| {
    Ok(format!("Hello, {input}!"))
});
```

For example, sending a POST request to `/greet` with the body `Bob` returns:

```text
Hello, Bob!
```

## Requests

| Callback | HTTP Method | Receives Request Body | Returns Response Text |
|---|---|---|---|
| `add_tap_api` | GET | No | `OK` |
| `add_grab_api` | POST | Yes | `OK` |
| `add_tap_give_api` | GET | No | Yes |
| `add_grab_give_api` | POST | Yes | Yes |

Endpoints are matched using their request paths, such as `/hello` or `/greet`.

Requests using the wrong HTTP method receive `405 Method Not Allowed`. Requests to unregistered paths receive `404 Not Found`, unless static file hosting handles the path.

## Error Handling

All callbacks must return a `Result`.

- `EasyResult` is used for callbacks that do not return output.
- `EasyResultOutput` is used for callbacks that return a `String`.
- `Ok(())` indicates success for callbacks without output.
- `Ok("message".to_string())` returns a successful response containing the specified text.
- `Err(status_code)` indicates an HTTP error, where `status_code` is a `u16` HTTP status code.

### Example

```rust
easy_api::add_grab_give_api("/greet", |input| {
    if input.is_empty() {
        return Err(400);
    }

    Ok(format!("Hello, {input}!"))
});
```

If the request body is empty, the callback returns `400 Bad Request`.

## Responses

| Status Code | Meaning |
|---|---|
| `200 OK` | The callback completed successfully. |
| `400 Bad Request` | The request body could not be read, or a callback returned this status code. |
| `404 Not Found` | No callback or applicable static file was found. |
| `405 Method Not Allowed` | The request used an unsupported HTTP method for the callback. |
| Other valid status codes | A callback can return another HTTP error status code. |

When a callback returns an error, the response body contains the standard message associated with that HTTP status code, where available.

Callbacks that return output send their returned string as the response body. Callbacks without output return `OK` when successful.

## Starting the Server

### `start()`

Starts the server on the default address:

```rust
easy_api::start();
```

Default address:

```text
127.0.0.1:3000
```

### `start_on(address)`

Starts the server on a specified address and returns the address it successfully bound to.

```rust
let address = easy_api::start_on("127.0.0.1:8080");
println!("Server running at {address}");
```

### `start_on_any()`

Starts the server on an available port on `127.0.0.1`.

```rust
let address = easy_api::start_on_any();
println!("Server running at {address}");
```

This is useful for tests or when you do not need to choose a specific port.

### Keeping the Program Alive

The server runs on a separate thread. If your main function needs to remain active, you can use `park()`:

```rust
easy_api::start();

loop {
    easy_api::park();
}
```

## Static File Hosting

**Static file hosting requires the `static` feature.**

It allows you to serve a website from a directory using the same server and port as your API.

For example, suppose your project has this structure:

```text
my-project/
├── Cargo.toml
├── src/
│   └── main.rs
└── client/
    ├── index.html
    ├── css/
    │   └── style.css
    └── js/
        └── main.js
```

You can register the directory for static hosting:

```rust
easy_api::serve_static("/", "client");
```

Then start the server as usual:

```rust
easy_api::start();

loop {
    easy_api::park();
}
```

Your website will be available at:

| URL | File |
|---|---|
| `/` | `client/index.html` |
| `/css/style.css` | `client/css/style.css` |
| `/js/main.js` | `client/js/main.js` |

You can use your API endpoints and static files on the same server.

For example:

```text
http://127.0.0.1:3000/          → Website
http://127.0.0.1:3000/css/style.css → CSS file
http://127.0.0.1:3000/hello     → API endpoint
```

### Static Hosting Notes

- Static hosting must be enabled when building the crate.
- The directory passed to `serve_static()` specifies where files are served from.
- `GET` and `HEAD` requests can be used to retrieve static files.
- API callbacks take priority when their paths are registered.
- Missing files return `404 Not Found`.
- Keep your static directory free of private files, secrets, and configuration files you do not want visitors to access.

## Development Status

`easy-api` is currently **under development**. Features and behaviour may change, and some functionality may not work as expected.

Contributions, bug reports, and suggestions are welcome.