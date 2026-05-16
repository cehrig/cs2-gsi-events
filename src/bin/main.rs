use clap::{Parser, ValueEnum};
use cs_gsi::error::Error;
use cs_gsi::models::player::Ammo;
use cs_gsi::models::GameState;
use cs_gsi::state::Event;
use futures::future::select_all;
use http_body_util::{BodyExt, Empty};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use std::fs::File;
use std::io::BufReader;
use std::net::{IpAddr, SocketAddr};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tracing::metadata::LevelFilter;
use tracing::{debug, error, info, warn};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "30001")]
    port: u16,

    #[arg(short, long, default_value = "127.0.0.1")]
    ip: IpAddr,

    #[arg(short, long, default_value = "sounds")]
    sound_path: String,

    #[arg(short, long)]
    debug: bool,

    #[arg(long)]
    disable: Vec<Disable>,

    #[arg(long)]
    no_visuals: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Disable {
    RoundTimer,
    BombTimer,
    AmmoLow,
    AmmoIndicator,
}

// Enable tracing
fn tracing_setup(debug: bool) {
    let mut filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if debug {
        filter = filter.add_directive(LevelFilter::DEBUG.into());
    }

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter)
        .init();
}

// Runs the program
#[tokio::main]
async fn main() {
    let args = Args::parse();
    tracing_setup(args.debug);

    let (tx, rx) = mpsc::channel(1024);

    // Audio Events
    let (atx, arx) = mpsc::channel(1024);

    // Senders
    let mut senders = vec![atx];

    // Visual Events
    let mut tasks = vec![];

    #[cfg(windows)]
    tasks.extend(window_events(args.no_visuals, &mut senders).expect("window events"));

    tasks.extend(vec![
        tokio::task::spawn(server(args.ip, args.port, tx)),
        tokio::task::spawn(handler(args.disable, rx, senders)),
        tokio::task::spawn(events(args.sound_path.clone(), arx)),
    ]);

    let _ = select_all(tasks).await;
}

// Starting a webserver that's receiving CS Gamestate requests
async fn server(ip: IpAddr, port: u16, tx: Sender<GameState>) -> Result<(), Error> {
    info!("Starting server...");
    let addr: SocketAddr = (ip, port).into();
    let listener = TcpListener::bind(addr).await?;

    async fn _server(
        req: Request<hyper::body::Incoming>,
        tx: Sender<GameState>,
    ) -> Result<Response<Empty<&'static [u8]>>, Infallible> {
        let Ok(body) = req.into_body().collect().await else {
            warn!("Request body is empty");
            return Ok(Response::new(Empty::new()));
        };

        let bytes = body.to_bytes().to_vec();
        let string = match String::from_utf8(bytes) {
            Ok(s) => s,
            Err(e) => {
                warn!("Unable to build string: {:?}", e);
                return Ok(Response::new(Empty::new()));
            }
        };

        let model = match serde_json::from_str::<GameState>(&string) {
            Ok(model) => {
                debug!("{}", string);
                model
            }
            Err(e) => {
                warn!("{}", string);
                warn!("Unable to parse model: {:?}", e);
                return Ok(Response::new(Empty::new()));
            }
        };

        if let Err(ex) = tx.send(model).await {
            warn!("Unable to send model: {:?}", ex);
        }

        Ok(Response::new(Empty::new()))
    }

    info!("Server Running");
    loop {
        let (tcp, _) = listener.accept().await.expect("accept");
        let io = TokioIo::new(tcp);
        let tx = tx.clone();

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(io, service_fn(|req| _server(req, tx.clone())))
                .await
            {
                error!("Error serving connection: {:?}", err);
            }
        });
    }
}

// Handles Gamestate requests by comparing new state with old states.
// Checks for events and sends them to the event handler
async fn handler(
    disabled: Vec<Disable>,
    mut rx: Receiver<GameState>,
    tx: Vec<Sender<Event>>,
) -> Result<(), Error> {
    let mut state: Option<GameState> = None;

    while let Some(model) = rx.recv().await {
        debug!("{:#?}", model);

        let old_gamestate = state.replace(model);

        let Some(previous) = old_gamestate else {
            continue;
        };

        let old_states = previous.get();
        let new_states = state.as_ref().unwrap().get();

        debug!("{:#?}", new_states);

        for event in new_states.events(&old_states).into_iter().filter(|e| {
            !disabled
                .iter()
                .any(|d| std::mem::discriminant(&Event::from(*d)) == std::mem::discriminant(e))
        }) {
            for t in &tx {
                t.send(event.clone()).await?;
            }
        }
    }

    Ok(())
}

// Handles events. There's not much happening here yet, mainly announcing remaining round & bomb times
async fn events(path: String, mut rx: Receiver<Event>) -> Result<(), Error> {
    let mut task: Option<JoinHandle<()>> = None;

    while let Some(event) = rx.recv().await {
        match event {
            Event::RoundOver => {
                info!("Round over");
                if let Some(task) = task.take() {
                    task.abort();
                }
            }
            Event::BombPlanted => {
                info!("Bomb planted");
                if let Some(task) =
                    task.replace(tokio::task::spawn(countdown_task(path.clone(), 38)))
                {
                    task.abort();
                }
            }
            Event::RoundStarted(limit) => {
                info!("Round started");
                task.replace(tokio::task::spawn(countdown_task(path.clone(), limit)));
            }
            Event::AmmoLow => {
                if let Ok(file) = File::open(format!("{}/ammo_low.wav", path)) {
                    play_sound(file).await;
                }
            }
            _ => {}
        }
    }

    Ok(())
}

#[cfg(windows)]
fn window_events(
    no_visuals: bool,
    senders: &mut Vec<Sender<Event>>,
) -> Result<Vec<JoinHandle<Result<(), Error>>>, Error> {
    use cs_gsi::windows::window::setup;

    if no_visuals {
        return Ok(vec![]);
    }

    // Display Events
    let (dtx, mut drx) = mpsc::channel(1024);

    // Display Texts
    let (tx, rx) = mpsc::channel(1024);
    senders.push(dtx);

    let event_task = tokio::task::spawn(async move {
        let mut is_playing = false;
        while let Some(event) = drx.recv().await {
            match event {
                Event::Ammo(ammo) if is_playing => {
                    let text = match ammo.ammo_clip_max {
                        0 => String::new(),
                        _ => format!("{}", ammo.ammo_clip),
                    };

                    tx.send(text).await?;
                }
                Event::PlayingStopped => {
                    is_playing = false;
                    tx.send(String::new()).await?;
                }
                Event::PlayingStarted => {
                    is_playing = true;
                }

                _ => {}
            }
        }

        Ok(())
    });

    let window_task = tokio::task::spawn_blocking(move || {
        let window = setup()?;
        window.events(rx)?;

        Ok(())
    });

    Ok(vec![event_task, window_task])
}

// Runs a countdown, triggering sound output if a corresponding file exists
// (e.g. "40.wav" for 40 seconds remaining)
async fn countdown_task(path: String, seconds: u8) {
    for i in 0..seconds {
        let remaining = seconds - i;
        info!("{}", remaining);

        if let Ok(file) = File::open(format!("{}/{}.wav", path, remaining)) {
            play_sound(file).await;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

// Plays a sound file. Only tested with wav files
async fn play_sound(file: File) {
    tokio::task::spawn(async {
        let mut sink_handle =
            rodio::DeviceSinkBuilder::open_default_sink().expect("open default audio stream");
        sink_handle.log_on_drop(false);

        let file = BufReader::new(file);
        let player = rodio::play(&sink_handle.mixer(), file).unwrap();

        player.sleep_until_end();
    });
}

impl From<Disable> for Event {
    fn from(value: Disable) -> Self {
        match value {
            Disable::RoundTimer => Event::RoundStarted(0),
            Disable::BombTimer => Event::BombPlanted,
            Disable::AmmoLow => Event::AmmoLow,
            Disable::AmmoIndicator => Event::Ammo(Ammo::default()),
        }
    }
}
