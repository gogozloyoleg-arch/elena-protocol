//! Точка входа узла сети «Елена».

use clap::{Parser, Subcommand};
use elena_core::crypto::KeyPair;
use elena_core::node::{ElenaNode, NodeConfig};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

fn default_admin() -> String {
    "127.0.0.1:9190".to_string()
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = ".elena")]
    data_dir: PathBuf,

    #[arg(short, long, default_value = "/ip4/0.0.0.0/tcp/9000")]
    listen: String,

    #[arg(short, long)]
    peers: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Запустить узел
    Run {
        #[arg(long, default_value_t = 1000)]
        balance: u64,

        #[arg(long)]
        enable_chaff: bool,

        #[arg(long, default_value_t = 0.05)]
        chaff_prob: f64,

        /// Использовать существующий кошелёк (файл data_dir/wallets/<name>.key)
        #[arg(long)]
        wallet: Option<String>,

        /// Адрес локального RPC для команд send/stats
        #[arg(long, default_value_t = default_admin())]
        admin: String,

        /// Интервал осаждения в секундах (0 = выключено)
        #[arg(long, default_value_t = 0)]
        emission_interval: u64,
    },

    /// Создать кошелёк и сохранить в data_dir/wallets/<name>.key
    Wallet { name: String },

    /// Отправить платёж (узел должен быть запущен с --admin)
    Send {
        #[arg(long)]
        to: String,
        #[arg(long)]
        amount: u64,
        #[arg(long, default_value_t = default_admin())]
        admin: String,
    },

    /// Статистика узла (узел должен быть запущен с --admin)
    Stats {
        #[arg(long, default_value_t = default_admin())]
        admin: String,
    },

    /// Публичный ключ узла (hex) для команд send
    Pubkey {
        #[arg(long, default_value_t = default_admin())]
        admin: String,
    },

    /// Заморозить долю репутации для стейкинга (0.0 .. 0.5)
    Stake {
        #[arg(long)]
        amount: f64,
        #[arg(long, default_value_t = default_admin())]
        admin: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            balance,
            enable_chaff,
            chaff_prob,
            wallet,
            admin,
            emission_interval,
        } => {
            println!("Запуск узла Елена");
            println!("Директория: {}", cli.data_dir.display());
            println!("Слушаем: {}", cli.listen);

            let keypair = if let Some(name) = wallet {
                let path = cli.data_dir.join("wallets").join(format!("{}.key", name));
                match KeyPair::load_from_path(&path) {
                    Ok(kp) => {
                        println!("Кошелёк загружен: {}", path.display());
                        Some(kp)
                    }
                    Err(e) => {
                        eprintln!("Не удалось загрузить кошелёк {}: {}", path.display(), e);
                        return Err(e.to_string().into());
                    }
                }
            } else {
                None
            };

            let config = NodeConfig {
                initial_balance: balance,
                initial_reputation: 0.5,
                enable_chaff,
                chaff_probability: chaff_prob,
                data_dir: cli.data_dir.to_string_lossy().to_string(),
                admin_listen: Some(admin),
                emission_interval_secs: emission_interval,
            };

            let mut node = ElenaNode::new(config, keypair).await?;
            node.p2p.listen_on(&cli.listen).await?;
            for peer in &cli.peers {
                println!("Подключаемся к {}", peer);
                node.p2p.dial(peer).await?;
            }
            node.run().await?;
        }

        Commands::Wallet { name } => {
            let keypair = KeyPair::generate();
            let path = cli.data_dir.join("wallets").join(format!("{}.key", name));
            keypair.save_to_path(&path).map_err(|e| e.to_string())?;
            println!("Кошелёк '{}' создан: {}", name, path.display());
            println!("Публичный ключ: {}", hex::encode(keypair.public_key()));
        }

        Commands::Send { to, amount, admin } => {
            admin_send(&admin, &to, amount).await?;
        }

        Commands::Stats { admin } => {
            admin_stats(&admin).await?;
        }

        Commands::Pubkey { admin } => {
            admin_pubkey(&admin).await?;
        }

        Commands::Stake { amount, admin } => {
            admin_stake(&admin, amount).await?;
        }
    }

    Ok(())
}

async fn admin_stats(admin: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = tokio::net::TcpStream::connect(admin).await?;
    stream.write_all(b"stats\n").await?;
    stream.flush().await?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).await?;
    println!("{}", buf.trim());
    Ok(())
}

async fn admin_pubkey(admin: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = tokio::net::TcpStream::connect(admin).await?;
    stream.write_all(b"pubkey\n").await?;
    stream.flush().await?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).await?;
    println!("{}", buf.trim());
    Ok(())
}

async fn admin_stake(
    admin: &str,
    fraction: f64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = tokio::net::TcpStream::connect(admin).await?;
    let cmd = format!("stake {}\n", fraction);
    stream.write_all(cmd.as_bytes()).await?;
    stream.flush().await?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).await?;
    let line = buf.trim();
    if line == "ok" {
        println!("Стейкинг установлен: {} (доля репутации)", fraction);
    } else {
        eprintln!("{}", line);
        return Err(line.to_string().into());
    }
    Ok(())
}

async fn admin_send(
    admin: &str,
    to: &str,
    amount: u64,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut stream = tokio::net::TcpStream::connect(admin).await?;
    let cmd = format!("send {} {}\n", to.trim(), amount);
    stream.write_all(cmd.as_bytes()).await?;
    stream.flush().await?;
    let mut buf = String::new();
    stream.read_to_string(&mut buf).await?;
    let line = buf.trim();
    if line.starts_with("ok ") {
        println!("Платёж отправлен, tx id: {}", line.strip_prefix("ok ").unwrap_or(""));
    } else {
        eprintln!("{}", line);
        return Err(line.to_string().into());
    }
    Ok(())
}
