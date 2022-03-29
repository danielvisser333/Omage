use omage::Engine;

fn main(){
    let engine = Engine::new("omage-bin");
    engine.set_window_name(String::from("omage"));
    engine.dispatch();
}