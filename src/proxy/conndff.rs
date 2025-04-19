
use crate::config::Config;
use crate::dns::resolve;
use std::pin::Pin;
use std::task::{Context, Poll};
use bytes::{BufMut, BytesMut};
use futures_util::Stream;
use pin_project_lite::pin_project;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use worker::*;
use tokio::time::{sleep, Duration};

pin_project! {
    pub struct ProxyStream<'a> {
        pub config: Config,
        pub ws: &'a WebSocket,
        pub buffer: BytesMut,
        #[pin]
        pub events: EventStream<'a>,
    }
}

impl<'a> ProxyStream<'a> {
    pub fn new(config: Config, ws: &'a WebSocket, events: EventStream<'a>) -> Self {
        let buffer = BytesMut::new();
        Self {
            config,
            ws,
            buffer,
            events,
        }
    }

    // Fungsi untuk menjaga koneksi WebSocket tetap hidup dengan mengirimkan ping secara periodik
    pub async fn keep_alive(&mut self) {
        loop {
            sleep(Duration::from_secs(30)).await;
            match self.ws.send_with_bytes(b"ping") {
                Ok(_) => {
                    console_log!("Ping terkirim untuk menjaga koneksi tetap hidup");
                }
                Err(e) => {
                    console_error!("Ping gagal: {}", e);
                    break;
                }
            }
        }
    }

    pub async fn fill_buffer_until(&mut self, n: usize) -> std::io::Result<()> {
        use futures_util::StreamExt;

        while self.buffer.len() < n {
            match self.events.next().await {
                Some(Ok(WebsocketEvent::Message(msg))) => {
                    if let Some(data) = msg.bytes() {
                        self.buffer.put_slice(&data);
                    }
                }
                Some(Ok(WebsocketEvent::Close(_))) => {
                    break;
                }
                Some(Err(e)) => {
                    return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
                }
                None => {
                    break;
                }
            }
        }

        Ok(())
    }

    pub fn peek_buffer(&self, n: usize) -> &[u8] {
        let len = self.buffer.len().min(n);
        &self.buffer[..len]
    }

    pub async fn process(&mut self) -> Result<()> {
        self.fill_buffer_until(62).await?;
        let peeked_buffer = self.peek_buffer(62);

        if peeked_buffer[0] == 0 {
            console_log!("VLESS detected!");
            self.process_vless().await
        } else if peeked_buffer[0] == 1 || peeked_buffer[0] == 3 {
            console_log!("Shadowsocks detected!");
            self.process_shadowsocks().await
        } else if peeked_buffer[56] == 13 && peeked_buffer[57] == 10 {
            console_log!("Trojan detected!");
            self.process_trojan().await
        } else {
            console_log!("Vmess detected!");
            self.process_vmess().await
        }
    }

    pub async fn handle_tcp_outbound(&mut self, addr: String, port: u16) -> Result<()> {
        console_log!("connecting to upstream {}:{}", addr, port);

        let addr_ip = if addr.parse::<std::net::IpAddr>().is_ok() {
            addr.clone()
        } else {
            resolve(&addr).await.map_err(|e| worker::Error::RustError(format!("resolve failed: {e}")))?
        };

        let mut remote_socket = Socket::builder().connect(addr_ip, port).map_err(|e| {
            Error::RustError(e.to_string())
        })?;

        remote_socket.opened().await.map_err(|e| {
            Error::RustError(e.to_string())
        })?;

        tokio::io::copy_bidirectional(self, &mut remote_socket)
            .await
            .map_err(|e| {
                Error::RustError(e.to_string())
            })?;
        Ok(())
    }

    pub async fn handle_udp_outbound(&mut self) -> Result<()> {
        let mut buff = vec![0u8; 65535];
        let n = self.read(&mut buff).await?;
        let data = &buff[..n];

        let is_dns_query = data.len() >= 12 && (data[2] & 0x80) == 0;

        if is_dns_query {
            match crate::dns::doh(data).await {
                Ok(resp) => {
                    self.write(&resp).await?; 
                    return Ok(());
                }
                Err(e) => {
                    console_error!("DoH gagal: {}", e);
                }
            }
        }

        console_log!("UDP bukan DNS, atau format tidak valid");
        Ok(())
    }
}

impl<'a> AsyncRead for ProxyStream<'a> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<tokio::io::Result<()>> {
        let mut this = self.project();

        loop {
            let size = std::cmp::min(this.buffer.len(), buf.remaining());
            if size > 0 {
                buf.put_slice(&this.buffer.split_to(size));
                return Poll::Ready(Ok(()));
            }

            match this.events.as_mut().poll_next(cx) {
                Poll::Ready(Some(Ok(WebsocketEvent::Message(msg)))) => {
                    msg.bytes().iter().for_each(|x| this.buffer.put_slice(&x));
                }
                Poll::Pending => return Poll::Pending,
                _ => return Poll::Ready(Ok(())),
            }
        }
    }
}

impl<'a> AsyncWrite for ProxyStream<'a> {
    fn poll_write(
        self: Pin<&mut Self>,
        _: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<tokio::io::Result<usize>> {
        return Poll::Ready(
            self.ws
                .send_with_bytes(buf)
                .map(|_| buf.len())
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string())),
        );
    }

    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<tokio::io::Result<()>> {
        unimplemented!()
    }
}
