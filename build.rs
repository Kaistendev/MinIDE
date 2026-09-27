//! Incrusta el icono del ejecutable.
//!
//! Windows saca el icono del ejecutable de un recurso que va dentro del binario, y ese
//! recurso no aparece solo: hay que generarlo al compilar. Sin esto, el logo solo se ve
//! dentro de la ventana.
//!
//! El recurso sale de `assets/LogoMiniIDE.ico`, que a su vez se genera del PNG del
//! logo. Si el logo cambia hay que regenerar el `.ico` y volver a compilar, y por eso
//! el `rerun-if-changed`: cambiar el icono tiene que recompilar, y cambiar el PNG
//! tambien, aunque el PNG solo lo use la biblioteca.

fn main() {
    println!("cargo:rerun-if-changed=assets/LogoMiniIDE.ico");
    println!("cargo:rerun-if-changed=assets/LogoMiniIDE.png");

    // `winresource` genera el recurso en Rust puro. La alternativa de siempre
    // (`embed-resource`) necesita `rc.exe` del SDK de Windows, que no esta en todas
    // las maquinas donde se compila MiniIDE.
    let mut recurso = winresource::WindowsResource::new();

    recurso.set_icon("assets/LogoMiniIDE.ico");

    recurso
        .compile()
        .expect("el icono del ejecutable tiene que quedar incrustado");
}
