use std::net::SocketAddr;
use std::path::Path;
use std::process::exit;
use std::str::FromStr;
use std::sync::Arc;

use clap::value_t_or_exit;
use pretty_env_logger;

use pixelflut;
use pixelflut::pixmap::Pixmap;

mod cli;
#[cfg(feature = "gui")]
mod gui;
mod differential_state;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let matches = cli::get_app().get_matches();

    match matches.subcommand() {
        // subcommand to start server
        ("server", Some(sub_matches)) => {
            start_server(
                value_t_or_exit!(sub_matches, "width", usize),
                value_t_or_exit!(sub_matches, "height", usize),
                sub_matches
                    .value_of("path")
                    .expect("path is required but not in matches"),
                value_t_or_exit_opt!(sub_matches, "tcp_port", usize),
                value_t_or_exit_opt!(sub_matches, "udp_port", usize),
                value_t_or_exit_opt!(sub_matches, "ws_port", usize),
            )
            .await;
        }

        // subcommand to start gui
        #[cfg(feature = "gui")]
        ("gui", Some(sub_matches)) => {
            let gtk_args = match sub_matches.values_of("gtk-args") {
                None => Vec::new(),
                Some(values) => values.collect(),
            };
            gui::start_gui(&gtk_args);
        }

        // no subcommand given
        ("", None) => {
            println!("No subcommand given");
            println!("Call with --help for more information");
            exit(1);
        }

        // match exhaustion, this should not happen
        (sub_command, sub_matches) => panic!(
            "Unhandled subcommand '{}' with sub_matches {:?}",
            sub_command, sub_matches
        ),
    }
}

async fn start_server(
    width: usize,
    height: usize,
    path: &str,
    tcp_port: Option<usize>,
    udp_port: Option<usize>,
    ws_port: Option<usize>,
) {
    // create pixmap instances
    let primary_pixmap =
        pixelflut::pixmap::InMemoryPixmap::new(width, height).expect("could not create in memory pixmap");
    let file_pixmap = pixelflut::pixmap::FileBackedPixmap::new(&Path::new(path), width, height, false)
        .expect(&format!("could not create pixmap backed by file {}", path));

    // copy data from file into memory
    primary_pixmap
        .put_raw_data(
            &file_pixmap
                .get_raw_data()
                .expect("could not load pixel data from file"),
        )
        .expect("could not put pixel data into memory");

    // create final pixmap instance which automatically saves data into file
    let pixmap = Arc::new(
        pixelflut::pixmap::ReplicatingPixmap::new(primary_pixmap, vec![Box::new(file_pixmap)], 0.2).unwrap(),
    );
    let encodings = pixelflut::state_encoding::SharedMultiEncodings::default();
    let mut server_handles = Vec::new();

    if let Some(tcp_port) = tcp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, _) = pixelflut::net::tcp_server::start_listener(
            pixmap,
            encodings,
            pixelflut::net::tcp_server::TcpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", tcp_port))
                    .expect("could not build SocketAddr"),
            },
        );
        server_handles.push(handle);
    }

    if let Some(udp_port) = udp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, _) = pixelflut::net::udp_server::start_listener(
            pixmap,
            encodings,
            pixelflut::net::udp_server::UdpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", udp_port))
                    .expect("could not build SocketAddr"),
            },
        );
        server_handles.push(handle);
    }

    if let Some(ws_port) = ws_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, _) = pixelflut::net::ws_server::start_listener(
            pixmap.clone(),
            encodings.clone(),
            pixelflut::net::ws_server::WsOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", ws_port))
                    .expect("could not build SocketAddr"),
            },
        );
        server_handles.push(handle);
    }

    let encoder_handles = pixelflut::state_encoding::start_encoders(encodings, pixmap);

    for handle in server_handles {
        let _ = tokio::join!(handle);
    }
    for handle in encoder_handles {
        let _ = tokio::join!(handle.0);
    }
}
