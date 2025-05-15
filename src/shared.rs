use std::sync::OnceLock;

static PORT: OnceLock<u16> = OnceLock::new();

pub fn get_or_init_port() -> u16 {
    *PORT.get_or_init(|| {
        std::net::TcpListener::bind("127.0.0.1:0")
            .expect("Failed to bind to ephemeral port")
            .local_addr()
            .unwrap()
            .port()
    })
}
