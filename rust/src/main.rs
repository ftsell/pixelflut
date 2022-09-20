use actix::prelude::*;
use std::net::SocketAddr;
use std::process::exit;
use std::str::FromStr;
use std::time::Duration;

use clap::value_t_or_exit;
use pretty_env_logger;

use pixelflut;
use pixelflut::differential_state::TrackerActor;
use pixelflut::net::tcp::{TcpOptions, TcpServer};
use pixelflut::net::udp::{UdpOptions, UdpServer};
use pixelflut::net::ws::{WsOptions, WsServer};
use pixelflut::pixmap::pixmap_actor::PixmapActor;
use pixelflut::state_encoding::{AutoEncoder, MultiEncodersClient, Rgb64Encoder, Rgba64Encoder};

mod cli;
#[cfg(feature = "gui")]
mod gui;

#[actix::main]
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
    // let file_pixmap = pixelflut::pixmap::FileBackedPixmap::new(&Path::new(path), width, height, false)
    //     .expect(&format!("could not create pixmap backed by file {}", path));

    // copy data from file into memory
    // primary_pixmap
    //     .put_raw_data(
    //         &file_pixmap
    //             .get_raw_data()
    //             .expect("could not load pixel data from file"),
    //     )
    //     .expect("could not put pixel data into memory");

    // create final pixmap instance which automatically saves data into file
    // let pixmap =
    //     pixelflut::pixmap::ReplicatingPixmap::new(primary_pixmap, vec![Box::new(file_pixmap)], 0.2).unwrap();

    let tracker = TrackerActor::new(width, height).start();
    let pixmap_addr = PixmapActor::new(primary_pixmap, Some(tracker.clone().recipient())).start();

    // start AutoEncoders for the pixmap
    let rgb64_encoder: Addr<AutoEncoder<_, Rgb64Encoder>> =
        AutoEncoder::new(Duration::from_secs(1), pixmap_addr.clone()).start();
    let rgba64_encoder: Addr<AutoEncoder<_, Rgba64Encoder>> =
        AutoEncoder::new(Duration::from_secs(1), pixmap_addr.clone()).start();
    let enc_client = MultiEncodersClient::new(rgb64_encoder.recipient(), rgba64_encoder.recipient());

    let _tcp_server = tcp_port.map(|tcp_port| {
        TcpServer::new(
            TcpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", tcp_port))
                    .expect("could not build SocketAddr"),
            },
            pixmap_addr.clone(),
            enc_client.clone(),
            tracker.clone(),
        )
        .start()
    });

    let _udp_server = udp_port.map(|udp_port| {
        UdpServer::new(
            UdpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", udp_port))
                    .expect("could not build SocketAddr"),
            },
            pixmap_addr.clone(),
            enc_client.clone(),
        )
        .start()
    });

    let _ws_server = ws_port.map(|ws_port| {
        WsServer::new(
            WsOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", ws_port))
                    .expect("could not build SocketAddr"),
            },
            pixmap_addr,
            enc_client,
        )
        .start()
    });

    // block the runtime so the program doesn't shutdown
    let mut interval = actix::clock::interval(Duration::from_secs(1));
    loop {
        interval.tick().await;
    }
}
