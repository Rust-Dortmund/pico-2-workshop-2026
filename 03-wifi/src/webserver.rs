//! This module contains the webserver that can be used to control the LED's color.

use defmt::info;
use embassy_time::Duration;
use picoserve::{
    AppRouter, AppWithStateBuilder,
    extract::State,
    make_static,
    routing::{PathRouter, parse_path_segment, post},
};

use crate::{led::Color, led_controller::ColorSender};

/// Defines the routes supported by the webserver by implementing the [`AppWithStateBuilder`] trait.
struct AppProps;
impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        picoserve::Router::new().route(
            // On POST to `/color/<something>` we expect `something` to parse as a `Color`.
            ("/color", parse_path_segment()),
            post(
                |color: Color, State(sender): State<ColorSender>| async move {
                    info!("[Webserver] Setting led to {}", color);

                    todo!("Notify the LED controller of the new color.");

                    todo!("Return a JSON body containing the new color.");
                    ()
                },
            ),
        )
    }
}

/// A runner for driving a webserver connection.
pub(crate) struct WebserverRunner {
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
}
impl WebserverRunner {
    const PORT: u16 = 80;

    pub(crate) async fn run(self) -> ! {
        // Reserve some stack space for sending and receiving requests / responses.
        let mut tcp_rx_buffer = [0; 1024];
        let mut tcp_tx_buffer = [0; 1024];
        let mut http_buffer = [0; 2048];

        let server = todo!("Create a `picoserve` server on top of the `embassy` net stack.");

        todo!("Listen for incoming requests that the server will then handle.");
    }
}

/// Factory to create [`WebserverRunner`]s.
pub(crate) struct WebserverRunnerFactory {
    next_id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
}

impl WebserverRunnerFactory {
    pub(crate) fn new_runner(&mut self) -> WebserverRunner {
        let runner = WebserverRunner {
            id: self.next_id,
            stack: self.stack,
            app: self.app,
            config: self.config,
            state: self.state.clone(),
        };
        self.next_id += 1;
        runner
    }
}

/// Initializes the webserver, returning a factory that can be used to create runners for network connections.
///
/// The channel for `sender` needs to be connected to the LED controller so the webserver can send new color values.
pub(crate) fn initialize(
    stack: embassy_net::Stack<'static>,
    sender: ColorSender,
) -> WebserverRunnerFactory {
    // Create the initial state and global router.
    let state = AppState {sender};
    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());

    // Configure some default values for request timeouts.
    let config = make_static!(
        picoserve::Config::<Duration>,
        picoserve::Config::new(picoserve::Timeouts {
            start_read_request: Some(Duration::from_secs(5)),
            persistent_start_read_request: Some(Duration::from_secs(1)),
            read_request: Some(Duration::from_secs(1)),
            write: Some(Duration::from_secs(1)),
        })
        .keep_connection_alive()
    );

    // Create the factory.
    WebserverRunnerFactory {
        next_id: 0,
        stack,
        app,
        config,
        state,
    }
}

/// Webserver state.
/// In our case, we just need a channel connection for sending color requests to the LED controller.
#[derive(Clone)]
struct AppState {
    sender: ColorSender,
}
impl picoserve::extract::FromRef<AppState> for ColorSender {
    fn from_ref(state: &AppState) -> Self {
        state.sender.clone()
    }
}