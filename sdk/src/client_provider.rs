use crate::client::Client;
use crate::client_error::ClientError;
use crate::http::client::HttpClient;
use crate::http::config::HttpClientConfig;
use crate::quic::client::QuicClient;
use crate::quic::config::QuicClientConfig;
use crate::tcp::client::TcpClient;
use crate::tcp::config::TcpClientConfig;
use std::sync::Arc;

const QUIC_TRANSPORT: &str = "quic";
const HTTP_TRANSPORT: &str = "http";
const TCP_TRANSPORT: &str = "tcp";

pub struct ClientProviderConfig {
    pub transport: String,
    pub http: Option<Arc<HttpClientConfig>>,
    pub quic: Option<Arc<QuicClientConfig>>,
    pub tcp: Option<Arc<TcpClientConfig>>,
}

impl ClientProviderConfig {
    pub fn from_args(args: crate::args::Args) -> Result<Self, ClientError> {
        let transport = args.transport;
        let mut config = Self {
            transport,
            http: None,
            quic: None,
            tcp: None,
        };
        match config.transport.as_str() {
            QUIC_TRANSPORT => {
                config.quic = Some(Arc::new(QuicClientConfig {
                    client_address: args.quic_client_address,
                    server_address: args.quic_server_address,
                    server_name: args.quic_server_name,
                    response_buffer_size: args.quic_response_buffer_size,
                    max_concurrent_bidi_streams: args.quic_max_concurrent_bidi_streams,
                    datagram_send_buffer_size: args.quic_datagram_send_buffer_size,
                    initial_mtu: args.quic_initial_mtu,
                    send_window: args.quic_send_window,
                    receive_window: args.quic_receive_window,
                    keep_alive_interval: args.quic_keep_alive_interval,
                    max_idle_timeout: args.quic_max_idle_timeout,
                }));
            }
            HTTP_TRANSPORT => {
                config.http = Some(Arc::new(HttpClientConfig {
                    api_url: args.http_api_url,
                    retries: args.http_retries,
                }));
            }
            TCP_TRANSPORT => {
                config.tcp = Some(Arc::new(TcpClientConfig {
                    server_address: args.tcp_server_address,
                }));
            }
            _ => return Err(ClientError::InvalidTransport(config.transport.clone())),
        }

        Ok(config)
    }
}

pub async fn get_client(config: Arc<ClientProviderConfig>) -> Result<Box<dyn Client>, ClientError> {
    let transport = config.transport.clone();
    match transport.as_str() {
        QUIC_TRANSPORT => {
            let quic_config = config.quic.as_ref().unwrap();
            let mut client = QuicClient::create(quic_config.clone())?;
            client.connect().await?;
            Ok(Box::new(client))
        }
        HTTP_TRANSPORT => {
            let http_config = config.http.as_ref().unwrap();
            let client = HttpClient::create(http_config.clone())?;
            Ok(Box::new(client))
        }
        TCP_TRANSPORT => {
            let tcp_config = config.tcp.as_ref().unwrap();
            let mut client = TcpClient::create(tcp_config.clone())?;
            client.connect().await?;
            Ok(Box::new(client))
        }
        _ => Err(ClientError::InvalidTransport(transport)),
    }
}
