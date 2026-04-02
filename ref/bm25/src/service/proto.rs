include!(concat!(env!("OUT_DIR"), "/bm25.rs"));

pub use bm25_service_server::Bm25Service;
pub use bm25_service_server::Bm25ServiceServer;
