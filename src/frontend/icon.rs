//! El logo de MiniIDE.
//!
//! Aqui solo se carga. El icono de la ventana va incrustado en el ejecutable, asi que
//! MiniIDE abre con su marca en el equipo de al lado y no solo en el suyo.
//!
//! Lo que se incrusta no es el logo entero sino `assets/LogoMiniIDE-icon.png`, que es
//! el mismo logo reducido. Reducirlo aqui costaba mas de un segundo: el PNG del logo
//! mide 1254x1254, y reescalarlo en cada arranque es pagar 1,4 s de Lanczos para pintar
//! un icono de 32 pixeles. Reducido de antemano, son 20 KB y se carga al instante.
//!
//! El icono del ejecutable es otra cosa: es el `.ico` que incrusta la compilacion, y
//! sale tambien del logo. Los dos se generan del PNG grande, y los dos hay que
//! regenerar cuando el logo cambie.

use eframe::egui;

/// El icono de la ventana, ya reducido, dentro del binario.
///
/// Va incrustado y no se lee de disco: un icono que depende de una ruta no aparece en
/// el equipo de al lado, y un IDE que abre con otro icono parece que no es el mismo.
pub const LOGO: &[u8] = include_bytes!("../../assets/LogoMiniIDE-icon.png");

/// Lado del icono de la ventana, en pixeles.
///
/// El doble del lado mayor que se ve de verdad (48 a 100%) para que en una pantalla
/// con escala alta no salga borroso.
pub const LADO: u32 = 128;

/// El logo como icono de la ventana, o `None` si no se puede cargar.
///
/// El fallo no se propaga porque el logo es adorno: MiniIDE tiene que abrir igual, con
/// el icono por defecto de egui, y lo que no puede es no abrir. Que el icono se pueda
/// cargar lo vigila un test, que es donde un logo roto tiene que dar la cara.
pub fn icono() -> Option<egui::IconData> {
    let icono = image::load_from_memory(LOGO).ok()?.into_rgba8();
    let (width, height) = (icono.width(), icono.height());

    Some(egui::IconData {
        rgba: icono.into_raw(),
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El icono va incrustado en el binario y no se busca en disco.
    ///
    /// Un icono que se lee de una ruta es un icono que no aparece en el equipo de al
    /// lado, que es el primer aviso que recibe un usuario: MiniIDE abre con un icono
    /// que no es el suyo. Los bytes van dentro del ejecutable y por eso tienen que
    /// estar aqui y ser un PNG de verdad.
    #[test]
    fn the_logo_is_embedded_in_the_binary() {
        assert_eq!(
            &LOGO[..8],
            [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A],
            "el recurso incrustado tiene que ser el icono en PNG"
        );
    }

    /// El icono incrustado son los pixeles del logo, ya del tamano que se usa.
    ///
    /// El tamano se comprueba porque es el que hace que MiniIDE abra enseguida: con
    /// el logo entero incrustado habia que reescalarlo en cada arranque, y eso costaba
    /// mas de un segundo de retardar la ventana por pintar un icono.
    #[test]
    fn the_logo_can_be_loaded_as_an_icon() {
        let icono = icono().expect("el logo se puede cargar");

        assert_eq!((icono.width, icono.height), (LADO, LADO));
        assert_eq!(
            icono.rgba.len(),
            LADO as usize * LADO as usize * 4,
            "un icono RGBA tiene cuatro bytes por pixel"
        );
        assert!(
            icono.rgba.iter().any(|byte| *byte != 0),
            "un icono en blanco no es el logo"
        );
    }
}
