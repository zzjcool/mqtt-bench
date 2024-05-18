use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "mqtt-bench", author, version, about, long_about = None, disable_help_flag = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(long)]
    pub help: bool,
}

#[derive(Debug, Args)]
pub struct Common {
    #[arg(short = 'h', long)]
    pub host: String,

    #[arg(short = 'p', long)]
    pub port: Option<u16>,

    #[arg(short = 'u', long)]
    pub user_name: String,

    #[arg(short = 'P', long)]
    pub password: String,

    #[arg(short = 's', long)]
    pub ssl: bool,

    #[arg(short = 'q', long, default_value_t = 0)]
    pub qos: i32,

    #[arg(short = 'c', long, default_value_t = 16)]
    pub concurrency: usize,

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

#[derive(Subcommand, Debug)]
pub enum Commands {
    Connect {
        #[command(flatten)]
        common: Common,
    },

    Pub {
        #[command(flatten)]
        common: Common,

        #[arg(long)]
        topic: String,

        #[arg(long, default_value_t = 64)]
        message_size: u32,

        #[arg(long)]
        payload: Option<String>,
    },

    Sub {
        #[command(flatten)]
        common: Common,

        #[arg(long)]
        topic: String,
    },
}
