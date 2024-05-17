use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Args)]
pub struct Common {
    #[arg(long)]
    pub host: String,

    #[arg(long)]
    pub port: Option<u16>,

    #[arg(short, long)]
    pub user_name: String,

    #[arg(long)]
    pub password: String,

    #[arg(short, long)]
    pub ssl: bool,

    #[arg(long, default_value_t = 0)]
    pub qos: i32,
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
