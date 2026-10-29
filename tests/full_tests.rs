use easy_api::{add_tap_api, add_grab_api, add_tap_give_api, add_grab_give_api, start_on_any};
use std::sync::{
    Arc,
    atomic::{AtomicI8, Ordering},
};

#[test]
fn test_apis() {
    let called = Arc::new(AtomicI8::new(0));
    let tap_called = Arc::clone(&called);
    let grab_called = Arc::clone(&called);
    let tap_give_called = Arc::clone(&called);
    let grab_give_called = Arc::clone(&called);

    add_tap_api("/test", move || {
        println!("Callback called!");
        tap_called.fetch_add(1, Ordering::SeqCst);
    });

    add_grab_api("/give-me-pizza", move |hopefully_pizza| {
        println!("Callback called!");
        assert_eq!(hopefully_pizza, "pizza");
        grab_called.fetch_add(1, Ordering::SeqCst);
    });

    add_tap_give_api("/hello", move || {
        tap_give_called.fetch_add(1, Ordering::SeqCst);
        "Hello, world!".to_string()
    });

    add_grab_give_api("/greet", move |name| {
        grab_give_called.fetch_add(1, Ordering::SeqCst);
        format!("Hello, {name}!")
    });


    let address = start_on_any();
    let base_url = format!("http://{address}");

    let response =
        reqwest::blocking::get(format!("{base_url}/does-not-exist"))
            .unwrap();

    assert_eq!(response.status(), 404);
    assert_eq!(response.text().unwrap(), "Not found");

    // Sending a POST to a GET endpoint
    let response = reqwest::blocking::Client::new()
        .post(format!("{base_url}/test"))
        .body("pizza")
        .send()
        .unwrap();

    assert_eq!(response.status(), 405);

    let response =
        reqwest::blocking::get(format!("{base_url}/test")).unwrap();

    assert_eq!(response.status(), 200);

    let response = reqwest::blocking::Client::new()
        .post(format!("{base_url}/give-me-pizza"))
        .body("pizza")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);

    let response =
        reqwest::blocking::get(format!("{base_url}/hello")).unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().unwrap(), "Hello, world!");

    let response = reqwest::blocking::Client::new()
        .post(format!("{base_url}/greet"))
        .body("Alice")
        .send()
        .unwrap();

    assert_eq!(response.status(), 200);
    assert_eq!(response.text().unwrap(), "Hello, Alice!");

    assert_eq!(called.load(Ordering::SeqCst), 4);
}