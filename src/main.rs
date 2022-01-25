use actix::System;
use near_indexer::{
    indexer_init_configs, AwaitForNodeSyncedEnum, Indexer, IndexerConfig, InitConfigArgs,
    SyncModeEnum,
};
use indexer::{listen_blocks, Result};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let home_dir = near_indexer::get_default_home();

    let command = args
        .get(1)
        .map(|arg| arg.as_str())
        .expect("You need to provide a command: `init` or `run` as arg");

    match command {
        "init" => {
            let config_args = InitConfigArgs {
                chain_id: Some("localnet".to_string()),
                account_id: None,
                test_seed: None,
                num_shards: 1,
                fast: false,
                genesis: None,
                download_genesis_url: None,
                download_config: false,
                download_config_url: None,
                boot_nodes: None,
                download_genesis: false,
                max_gas_burnt_view: None,
            };
            indexer_init_configs(&home_dir, config_args)?;
        }
        "run" => {
            let indexer_config = IndexerConfig {
                home_dir: near_indexer::get_default_home(),
                sync_mode: SyncModeEnum::FromInterruption,
                await_for_node_synced: AwaitForNodeSyncedEnum::WaitForFullSync,
            };
            let sys = System::new();
            sys.block_on(async move {
                let indexer = Indexer::new(indexer_config);
                let stream = indexer.streamer();
                actix::spawn(async {
                    if let Err(e) = listen_blocks(stream).await {
                        println!("`listen_blocks` is terminated with error: {:#}", e)
                    }
                });
            });
            sys.run()?;
        }
        _ => panic!("You have to pass `init` or `run` arg"),
    }

    Ok(())
}
