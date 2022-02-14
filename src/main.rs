use battlemon_indexer::{startup, telemetry, Result};

fn main() -> Result<()> {
    openssl_probe::init_ssl_cert_env_vars();
    let subscriber = telemetry::get_subscriber("battlemon_indexer".into(), "info".into());
    telemetry::init_subscriber(subscriber);
    let args: Vec<String> = std::env::args().collect();
    let command = args
        .get(1)
        .map(|arg| arg.as_str())
        .expect("You need to provide a command: `init` or `run` as arg");
    let home_dir = near_indexer::get_default_home();
    match command {
        "init" => startup::init_indexer(home_dir).expect("Couldn't init indexer"),
        "run" => startup::run_indexer(home_dir).expect("Couldn't run indexer"),
        _ => panic!("You have to pass `init` or `run` arg"),
    }

    Ok(())
}
