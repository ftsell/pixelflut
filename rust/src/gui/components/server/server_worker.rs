//! The *ServerWorker* is where the interaction with the rest of the program takes place for
//! server contexts.
//! The GUI communicates with this worker via [`ServerWorkerMsg`] and the *ServerWorker* sends
//! [`ServerHolderMsg`] back to the application.

use gtk::glib::Sender;
use pixelflut::net;
use pixelflut::net::tcp;
use pixelflut::pixmap::InMemoryPixmap;
use relm4::{send, ComponentUpdate, Model};
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio::{runtime, select, time};

use pixelflut::state_encoding::SharedMultiEncodings;

use crate::gui::components::server::server_config_form::ProtocolChoice;
use crate::gui::components::server::server_holder::{ServerHolderModel, ServerHolderMsg};
use crate::pixelflut::pixmap::Pixmap;

type ParentModel = ServerHolderModel;

/// State of the *ServerWorker* component.
pub(in crate::gui) struct ServerWorkerModel {
    runtime: Runtime,
    running_server: Option<PixelflutServer>,
}

struct PixelflutServer {
    stoppers: Vec<Arc<Notify>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(in crate::gui) enum ServerWorkerMsg {
    /// Start the server with the specified parameters
    StartServer { protocol: ProtocolChoice, port: u32 },
    /// Stop the server if it is running, ignore if not
    StopServer,
}

impl Model for ServerWorkerModel {
    type Msg = ServerWorkerMsg;
    type Widgets = ();
    type Components = ();
}

impl ComponentUpdate<ParentModel> for ServerWorkerModel {
    fn init_model(_parent_model: &ParentModel) -> Self {
        let runtime = runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("Could not construct tokio runtime");

        Self {
            running_server: None,
            runtime,
        }
    }

    fn update(
        &mut self,
        msg: Self::Msg,
        _components: &Self::Components,
        _sender: Sender<Self::Msg>,
        parent_sender: Sender<<ParentModel as Model>::Msg>,
    ) {
        log::debug!("ComponentsUpdate: {:?}", msg);

        match msg {
            ServerWorkerMsg::StartServer { protocol, port } => {
                // create data structures
                let pixmap = Arc::new(InMemoryPixmap::default());
                let encodings = SharedMultiEncodings::default();

                // spawn server task
                let pixmap2 = pixmap.clone();
                let encodings2 = encodings.clone();
                let (_, server_stopper) = self
                    .runtime
                    .block_on(async move { start_server(protocol, port as u16, pixmap2, encodings2) });

                // spawn synchronizer task
                let pixmap2 = pixmap.clone();
                let parent_sender2 = parent_sender.clone();
                let sync_stopper = Arc::new(Notify::new());
                let sync_stopper2 = sync_stopper.clone();
                let _synchronizer_handle = self.runtime.spawn(async move {
                    run_synchronizer(pixmap2, parent_sender2, sync_stopper2).await;
                });

                // spawn encoders
                let encodings2 = encodings.clone();
                let pixmap2 = pixmap.clone();
                let mut stoppers: Vec<_> = self
                    .runtime
                    .block_on(async move { pixelflut::state_encoding::start_encoders(encodings2, pixmap2) })
                    .iter()
                    .map(|h| &h.1)
                    .cloned()
                    .collect();

                // set running server on self
                stoppers.extend(vec![server_stopper, sync_stopper]);
                self.running_server = Some(PixelflutServer { stoppers });
            }
            ServerWorkerMsg::StopServer => match &self.running_server {
                None => {}
                Some(server) => {
                    for stopper in server.stoppers.iter() {
                        stopper.notify_one();
                    }
                    self.running_server = None;
                }
            },
        }
    }
}

async fn run_synchronizer(
    pixmap: Arc<InMemoryPixmap>,
    parent_sender: Sender<<ParentModel as Model>::Msg>,
    notify_stop: Arc<Notify>,
) {
    const FPS: f32 = 5f32;
    let mut interval = time::interval(Duration::from_secs_f32(1f32 / FPS));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

    loop {
        select! {
            _ = interval.tick() => {
                send!(
                    parent_sender,
                    ServerHolderMsg::UpdatePixmapData(Box::new(pixmap.get_raw_data().expect("Could not get raw data")))
                )
            },
            _ = notify_stop.notified() => {
                log::debug!("Stopping Synchronizer");
                break;
            }
        }
    }
}

fn start_server(
    protocol: ProtocolChoice,
    port: u16,
    pixmap: Arc<InMemoryPixmap>,
    encodings: SharedMultiEncodings,
) -> (JoinHandle<tokio::io::Result<()>>, Arc<Notify>) {
    match protocol {
        ProtocolChoice::TCP => tcp::tcp_server::start_listener(
            pixmap,
            encodings,
            tcp::tcp_server::TcpOptions {
                listen_address: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port),
            },
        ),
        ProtocolChoice::UDP => net::udp_server::start_listener(
            pixmap,
            encodings,
            net::udp_server::UdpOptions {
                listen_address: SocketAddr::new(IpAddr::from([0, 0, 0, 0]), port),
            },
        ),
    }
}
