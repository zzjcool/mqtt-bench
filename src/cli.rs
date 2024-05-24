use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "mqtt-bench", author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Clone, Args)]
pub struct Common {
    #[arg(long)]
    pub host: String,

    #[arg(short = 'p', long)]
    pub port: Option<u16>,

    #[arg(short = 'u', long)]
    pub user_name: String,

    #[arg(short = 'P', long)]
    pub password: String,

    #[arg(short = 's', long)]
    pub ssl: bool,

    #[arg(short, long)]
    pub verify: bool,

    #[arg(short, long)]
    pub auth_server_certificate: bool,

    #[arg(short = 'q', long, default_value_t = 0)]
    pub qos: i32,

    /// Total number of client to create
    #[arg(long, default_value_t = 16)]
    pub total: usize,

    /// The number of clients to create in parallel for each iteration
    #[arg(short = 'c', long, default_value_t = 4)]
    pub concurrency: usize,

    /// The interval between each message publishing for each client in milliseconds.
    #[arg(short = 'i', long, default_value_t = 100)]
    pub interval: u64,

    /// The duration of the test in seconds.
    #[arg(long, default_value_t = 60)]
    pub time: usize,

    #[arg(long, default_value_t = String::from("bench-client-%d"))]
    pub client_id: String,

    #[arg(long)]
    pub show_statistics: bool,
}

impl Common {
    pub fn connection_string(&self) -> String {
        if self.ssl {
            format!("ssl://{}:{}", self.host, self.port.unwrap_or(8883))
        } else {
            format!("tcp://{}:{}", self.host, self.port.unwrap_or(1883))
        }
    }
}

#[derive(Debug, Clone, Args)]
pub struct PubOptions {
    #[arg(long)]
    pub topic: String,

    #[arg(long, default_value_t = 64)]
    pub message_size: u32,

    #[arg(long)]
    pub payload: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Connect {
        #[command(flatten)]
        common: Common,
    },

    Pub {
        #[command(flatten)]
        common: Common,

        #[command(flatten)]
        pub_options: PubOptions,
    },

    Sub {
        #[command(flatten)]
        common: Common,

        #[arg(long)]
        topic: String,
    },
}
