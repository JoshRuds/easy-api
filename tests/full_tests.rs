use easy_api::{add_tap_api, add_grab_api, start_on_any};
use std::sync::{
    Arc,
    atomic::{AtomicI8, Ordering},
};

#[test]
fn test_apis() {
    let called = Arc::new(AtomicI8::new(0));
    let tap_called = Arc::clone(&called);
    let grab_called = Arc::clone(&called);

    add_tap_api("/test", move || {
        println!("Callback called!");
        tap_called.fetch_add(1, Ordering::SeqCst);
    });

    add_grab_api("/give-me-pizza", move |hopefully_pizza| {
        println!("Callback called!");
        assert_eq!(hopefully_pizza, "pizza");
        grab_called.fetch_add(1, Ordering::SeqCst);
    });

    let address = start_on_any();
    let base_url = format!("http://{address}");

    let response = reqwest::blocking::get(format!("{base_url}/test")).unwrap();
    assert_eq!(response.status(), 200);

    let response = reqwest::blocking::Client::new()
        .post(format!("{base_url}/give-me-pizza"))
        .body("pizza")
        .send()
        .unwrap();
    assert_eq!(response.status(), 200);


    assert_eq!(called.load(Ordering::SeqCst), 2);
}