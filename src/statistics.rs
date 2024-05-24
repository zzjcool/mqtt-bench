use prometheus::{linear_buckets, Encoder, Histogram, HistogramOpts, Registry, TextEncoder};

#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    pub connect: Histogram,
    pub publish: Histogram,
    pub subscribe: Histogram,
}

pub struct Statistics {
    pub registry: Registry,
    pub latency: LatencyHistogram,
}

impl Statistics {
    pub fn new() -> Self {
        let r = Registry::new();
        let conn_histogram_opts = HistogramOpts::new("conn_histogram", "Connect Latency Histogram")
            .buckets(linear_buckets(0.0, 100.0, 10).unwrap());
        let connect = Histogram::with_opts(conn_histogram_opts).unwrap();
        r.register(Box::new(connect.clone())).unwrap();

        let pub_histogram_opts = HistogramOpts::new("pub_histogram", "Publish MQTT Message Latency")
            .buckets(linear_buckets(0.0, 10.0, 20).unwrap());
        let publish = Histogram::with_opts(pub_histogram_opts).unwrap();
        r.register(Box::new(publish.clone())).unwrap();

        let sub_histogram_opts = HistogramOpts::new("sub_histogram", "E2E MQTT Message Delivery Latency")
            .buckets(linear_buckets(0.0, 10.0, 20).unwrap());
        let subscribe = Histogram::with_opts(sub_histogram_opts).unwrap();
        r.register(Box::new(subscribe.clone())).unwrap();

        let latency_histogram = LatencyHistogram {
            connect,
            publish,
            subscribe,
        };

        Self {
            registry: r,
            latency: latency_histogram,
        }
    }
    pub fn show_statistics(&self) {
        let mut buffer = vec![];
        let encoder = TextEncoder::new();
        let metric_families = self.registry.gather();
        encoder.encode(&metric_families, &mut buffer).unwrap();
        println!("{}", String::from_utf8(buffer).unwrap());
    }
}

impl Default for Statistics
{
    fn default() -> Self {
        Self::new()
    }
}
