use std::error::Error as StdError;
use std::fmt;
use std::net::Ipv4Addr;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::atomic::{AtomicU64, Ordering};

use iron::modifiers::Redirect;
use iron::prelude::*;
use iron::{
    headers, status, typemap, AfterMiddleware, Iron, IronError, IronResult, Request, Response, Url,
};
use iron_cors::CorsMiddleware;
use mount::Mount;
use params::{FromValue, Params};
use path::PathBuf;
use persistent::Write;
use router::Router;
use serde_json;
use staticfile::Static;
use std::thread;
use std::time::Duration;

use errors::*;
use exit::{exit, ExitResult};
use network::{NetworkCommand, NetworkCommandResponse};
use std::sync::Mutex;

static TIMER: AtomicU64 = AtomicU64::new(0);

struct RequestSharedState {
    gateway: Ipv4Addr,
    server_rx: Receiver<NetworkCommandResponse>,
    network_tx: Sender<NetworkCommand>,
    exit_tx: Sender<ExitResult>,
}

impl typemap::Key for RequestSharedState {
    type Value = RequestSharedState;
}

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl StdError for StringError {
    fn description(&self) -> &str {
        &*self.0
    }
}

macro_rules! get_request_ref {
    ($req:ident, $ty:ty, $err:expr) => {
        match $req.get_ref::<$ty>() {
            Ok(val) => val,
            Err(err) => {
                error!($err);
                return Err(IronError::new(err, status::InternalServerError));
            },
        }
    };
}

macro_rules! get_param {
    ($params:ident, $param:expr, $ty:ty) => {
        match $params.get($param) {
            Some(value) => match <$ty as FromValue>::from_value(value) {
                Some(converted) => converted,
                None => {
                    let err = format!("Unexpected type for '{}'", $param);
                    error!("{}", err);
                    return Err(IronError::new(
                        StringError(err),
                        status::InternalServerError,
                    ));
                },
            },
            None => {
                let err = format!("'{}' not found in request params: {:?}", $param, $params);
                error!("{}", err);
                return Err(IronError::new(
                    StringError(err),
                    status::InternalServerError,
                ));
            },
        }
    };
}

macro_rules! get_request_state {
    ($req:ident) => {
        get_request_ref!(
            $req,
            Write<RequestSharedState>,
            "Getting reference to request shared state failed"
        )
        .as_ref()
        .lock()
        .unwrap()
    };
}

fn exit_with_error<E>(state: &RequestSharedState, e: E, e_kind: ErrorKind) -> IronResult<Response>
where
    E: ::std::error::Error + Send + 'static,
{
    let description = e_kind.description().into();
    let err = Err::<Response, E>(e).chain_err(|| e_kind);
    exit(&state.exit_tx, err.unwrap_err());
    Err(IronError::new(
        StringError(description),
        status::InternalServerError,
    ))
}

struct RedirectMiddleware;

impl AfterMiddleware for RedirectMiddleware {
    fn catch(&self, req: &mut Request, err: IronError) -> IronResult<Response> {
        let gateway = {
            let request_state = get_request_state!(req);
            format!("{}", request_state.gateway)
        };

        if let Some(host) = req.headers.get::<headers::Host>() {
            if host.hostname != gateway {
                let url = Url::parse(&format!("http://{}/", gateway)).unwrap();
                return Ok(Response::with((status::Found, Redirect(url))));
            }
        }

        Err(err)
    }
}

pub fn start_server(
    gateway: Ipv4Addr,
    listening_port: u16,
    server_rx: Receiver<NetworkCommandResponse>,
    network_tx: Sender<NetworkCommand>,
    exit_tx: Sender<ExitResult>,
    ui_directory: &PathBuf,
) {
    let exit_tx_clone = exit_tx.clone();
    let gateway_clone = gateway;
    let request_state = RequestSharedState {
        gateway: gateway,
        server_rx: server_rx,
        network_tx: network_tx,
        exit_tx: exit_tx,
    };

    let mut router = Router::new();
    router.get("/", Static::new(ui_directory), "index");
    router.post("/reset_dhcp", reset_dhcp, "reset_dhcp");
    router.get("/get_timer", get_timer, "get_timer");

    let mut assets = Mount::new();
    assets.mount("/", router);
    assets.mount("/static", Static::new(&ui_directory.join("static")));
    assets.mount("/css", Static::new(&ui_directory.join("css")));
    assets.mount("/img", Static::new(&ui_directory.join("img")));
    assets.mount("/js", Static::new(&ui_directory.join("js")));

    let cors_middleware = CorsMiddleware::with_allow_any();

    let mut chain = Chain::new(assets);
    chain.link(Write::<RequestSharedState>::both(request_state));
    chain.link_after(RedirectMiddleware);
    chain.link_around(cors_middleware);

    let address = format!("{}:{}", gateway_clone, listening_port);

    info!("Starting HTTP server on {}", &address);

    if let Err(e) = Iron::new(chain).http(&address) {
        exit(
            &exit_tx_clone,
            ErrorKind::StartHTTPServer(address, e.description().into()).into(),
        );
    }
}

pub fn start_timer(secs: u64, network_tx: Sender<NetworkCommand>) {
    TIMER.store(secs, Ordering::Relaxed);
    while TIMER.load(Ordering::Relaxed) > 0 {
        thread::sleep(Duration::from_secs(1));
        TIMER.store(TIMER.load(Ordering::Relaxed)-1, Ordering::Relaxed);
    }
    if let Err(err) = network_tx.send(NetworkCommand::OverallTimeout) {
        error!(
            "Sending NetworkCommand::Timeout failed: {}",
            err.description()
        );
    }
}

fn get_timer(req: &mut Request) -> IronResult<Response> {
    let request_state = get_request_state!(req);

    if let Err(e) = request_state.network_tx.send(NetworkCommand::Activate) {
        return exit_with_error(&request_state, e, ErrorKind::SendNetworkCommandActivate);
    }

    let time = TIMER.load(Ordering::Relaxed);
    Ok(Response::with((status::Ok, time.to_string())))
}

fn reset_dhcp(req: &mut Request) -> IronResult<Response> {
    debug!("Requested DHCP reset");

    let request_state = get_request_state!(req);

    let command = NetworkCommand::Reset {};

    if let Err(e) = request_state.network_tx.send(command) {
        exit_with_error(&request_state, e, ErrorKind::SendNetworkCommandReset)
    } else {
        Ok(Response::with(status::Ok))
    }
}
