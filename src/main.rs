fn main() {
    // La ventana se abre desde el frontend, no desde el core: el core no sabe que
    // existe una interfaz. Si no se puede abrir, se dice por que en vez de dejar el
    // IDE sin explicacion.
    if let Err(error) = miniide::frontend::run() {
        eprintln!("{} no se ha podido abrir: {error}", miniide::APP_NAME);
        std::process::exit(1);
    }
}
