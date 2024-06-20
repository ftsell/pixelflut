#![feature(sync_unsafe_cell)]
use std::sync::Arc;
use std::{cell::SyncUnsafeCell, error::Error, sync::atomic::AtomicUsize};

use minifb::{Key, Window, WindowOptions};
use pixelparser::fast::ParserState;
use pixelparser::Pixmap;
use tokio::io::AsyncBufReadExt;
use tokio::time;

const WIDTH: usize = 1920;
const HEIGHT: usize = 1080;

static FRAMES: AtomicUsize = AtomicUsize::new(0);
static THROUGHPUT: AtomicUsize = AtomicUsize::new(0);

async fn stats() {
    let mut interval = time::interval(time::Duration::from_secs(1));
    loop {
        let frames = FRAMES.load(std::sync::atomic::Ordering::SeqCst);
        let bytes = THROUGHPUT.load(std::sync::atomic::Ordering::SeqCst);
        interval.tick().await;
        let frames2 = FRAMES.load(std::sync::atomic::Ordering::SeqCst);
        let bytes2 = THROUGHPUT.load(std::sync::atomic::Ordering::SeqCst);
        println!("{}fps {}GB/s", frames2 - frames, (bytes2 - bytes) as f64 / 1e9);
    }
}

async fn handle_socket(
    socket: tokio::net::TcpStream,
    pixmap: Arc<SyncUnsafeCell<Pixmap>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut parser = pixelparser::fast::Parser {
        state: ParserState::default(),
        pixmap: unsafe { pixmap.get().as_mut().unwrap() },
    };
    let mut reader = tokio::io::BufReader::with_capacity(4096 * 4, socket);
    loop {
        let buf = reader.fill_buf().await?;
        if buf.len() == 0 {
            break;
        }
        parser.consume(buf);
        let bytes = buf.len();
        reader.consume(bytes);
        THROUGHPUT.fetch_add(bytes, std::sync::atomic::Ordering::SeqCst);
    }
    Ok(())
}

async fn tcp_server(pixmap: Arc<SyncUnsafeCell<Pixmap>>) -> Result<(), Box<dyn Error + Send + Sync>> {
    let listener = tokio::net::TcpListener::bind("0.0.0.0:1234").await?;
    loop {
        let (socket, _) = listener.accept().await?;
        tokio::spawn(handle_socket(socket, pixmap.clone()));
    }
}

#[tokio::main]
async fn main() {
    let pixmap = Arc::new(SyncUnsafeCell::new(Pixmap::new(WIDTH as u32, HEIGHT as u32)));

    tokio::spawn(stats());
    tokio::spawn(tcp_server(pixmap.clone()));

    let mut window = Window::new("Test - ESC to exit", WIDTH, HEIGHT, WindowOptions::default())
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let mut interval = time::interval(time::Duration::from_millis(1000 / 60));
    while window.is_open() && !window.is_key_down(Key::Escape) {
        interval.tick().await;
        FRAMES.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let pixmap = unsafe { pixmap.get().as_ref().unwrap() };
        let buffer = pixmap.pixels.as_slice();

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
