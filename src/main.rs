#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    _start_puffin_server();

    let app = zen_sm::ZenSM::default();
    eframe::run_native(
        "Zen SM",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(app)),
    )
    .unwrap();
}

#[cfg(not(target_arch = "wasm32"))]
fn _start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new("0.0.0.0:8585") {
        Ok(puffin_server) => {
            eprintln!("Run:  cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585");

            // We can store the server if we want, but in this case we just want
            // it to keep running. Dropping it closes the server, so let's not drop it!
            #[allow(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            eprintln!("Failed to start puffin server: {}", err);
        }
    };
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    // Unwrap panics at 'console.log'.
    console_error_panic_hook::set_once();

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    let app = zen_sm::ZenSM::default();
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id", // hardcode it
                web_options,
                Box::new(|_cc| Box::new(app)),
            )
            .await
            .expect("failed to start eframe");
    });
}
