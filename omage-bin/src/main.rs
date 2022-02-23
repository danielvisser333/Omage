use omage_engine::Engine;

fn main() {
    let engine = Engine::new("omage-bin");
    engine.await_close_request();
}
