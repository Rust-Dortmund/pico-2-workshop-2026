use defmt::info;
use embassy_time::Duration;
use picoserve::{
    AppRouter, AppWithStateBuilder,
    extract::State,
    make_static,
    routing::{PathRouter, parse_path_segment, post},
};

use crate::{led::Color, led_controller::ColorSender};

#[derive(Clone)]
struct AppState {
    sender: ColorSender<2>,
}

impl picoserve::extract::FromRef<AppState> for ColorSender<2> {
    fn from_ref(state: &AppState) -> Self {
        state.sender.clone()
    }
}

struct AppProps;

impl AppWithStateBuilder for AppProps {
    type State = AppState;
    type PathRouter = impl PathRouter<AppState>;

    fn build_app(self) -> picoserve::Router<Self::PathRouter, Self::State> {
        picoserve::Router::new().route(
            ("/color", parse_path_segment()),
            post(
                |color: Color, State(sender): State<ColorSender<2>>| async move {
                    info!("[Webserver] Setting led to {}", color);
                    sender.send(color);
                    match color {
                        Color::Red => r#"{"color":"red"}"#,
                        Color::Green => r#"{"color":"green"}"#,
                        Color::Blue => r#"{"color":"blue"}"#,
                    }
                },
            ),
        )
    }
}

pub(crate) struct WebserverRunner {
    id: usize,
    stack: embassy_net::Stack<'static>,
    app: &'static AppRouter<AppProps>,
    config: &'static picoserve::Config<Duration>,
    state: AppState,
}

impl WebserverRunner {
    pub(crate) async fn run(self) -> ! {
        let port = 80;
        let mut tcp_rx_buffer = [0; 1024];
        let mut tcp_tx_buffer = [0; 1024];
        let mut http_buffer = [0; 2048];

        picoserve::listen_and_serve_with_state(
            self.id,
            self.app,
            self.config,
            self.stack,
            port,
            &mut tcp_rx_buffer,
            &mut tcp_tx_buffer,
            &mut http_buffer,
            &self.state,
        )
        .await
    }
}

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

pub(crate) fn initialize(
    stack: embassy_net::Stack<'static>,
    sender: ColorSender<2>,
) -> WebserverRunnerFactory {
    let app = make_static!(AppRouter<AppProps>, AppProps.build_app());

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

    WebserverRunnerFactory {
        next_id: 0,
        stack,
        app,
        config,
        state: AppState { sender },
    }
}
