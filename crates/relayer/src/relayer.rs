use crate::cli::CliCommand;
use crate::config::Config;
use crate::database::DatabaseTrait;
use crate::error::Error;
use crate::ethereum::{CloneableEthereumClient, DelegateSetChangedEvent};
use crate::hub::CloneableHubClient;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tracing::{error, info};

pub struct Relayer {
    pub config: Config,
    pub ethereum_client: Box<dyn CloneableEthereumClient>, // Change to Box
    pub hub_client: Box<dyn CloneableHubClient>,           // Change to Box
    pub database: Arc<dyn DatabaseTrait>,                  // Change to Box
}

impl Relayer {
    pub fn new(
        config: Config,
        ethereum_client: Box<dyn CloneableEthereumClient>, // Change to Box
        hub_client: Box<dyn CloneableHubClient>,           // Change to Box
        database: Arc<dyn DatabaseTrait>,                  // Change to Box
    ) -> Self {
        Self {
            config,
            ethereum_client,
            hub_client,
            database,
        }
    }

    pub fn start(&self, start_block: Option<u64>) -> Result<(), Error> {
        // Initialize database with genesis block if necessary.
        self.database.initialize(self.config.genesis_block)?;
        // Determine where to start syncing from.  Use provided start_block,
        // last processed block, or genesis block, in that order of precedence.
        let mut from_block = match start_block {
            Some(block) => block,
            None => {
                match self.database.get_last_processed_block()? {
                    Some(block) => block + 1,          // Start from the *next* block
                    None => self.config.genesis_block, // Fallback to genesis
                }
            },
        };

        // Create channels for communication between threads.
        let (event_sender, event_receiver): (
            Sender<DelegateSetChangedEvent>,
            Receiver<DelegateSetChangedEvent>,
        ) = channel();

        let (tx_sender, tx_receiver) = channel();

        // Clone or create new instances of clients and database for each thread
        let ethereum_client_clone = self.ethereum_client.clone_box();
        let hub_client_clone = self.hub_client.clone_box();
        let process_database_clone = self.database.clone();
        let send_database_clone = self.database.clone();
        let config_clone = self.config.clone();

        // Polling Thread
        let poll_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
            let _enter = rt.enter();

            loop {
                rt.block_on(async {
                    // Get the latest block number
                    let latest_block = match ethereum_client_clone.get_latest_block_number().await {
                        Ok(n) => n,
                        Err(e) => {
                            error!("Failed to get latest block number: {}", e);
                            tokio::time::sleep(Duration::from_secs(
                                config_clone.retry_backoff_seconds,
                            ))
                            .await;
                            return;
                        },
                    };

                    // Avoid querying beyond the latest block.
                    let to_block =
                        std::cmp::min(from_block + config_clone.batch_size - 1, latest_block);

                    if from_block <= to_block {
                        // Fetch events
                        match ethereum_client_clone
                            .get_delegate_set_changed_events(from_block, to_block)
                            .await
                        {
                            Ok(events) => {
                                if !events.is_empty() {
                                    info!("Found {} new events", events.len());
                                }
                                for event in events {
                                    // Send each event to the processing thread.
                                    if let Err(e) = event_sender.send(event.clone()) {
                                        error!("Failed to send event to processing thread: {}", e);
                                        // Consider if you want to stop or continue to the next event.
                                        return;
                                    }
                                }
                                from_block = to_block + 1; // Prepare for the next batch.
                            },
                            Err(e) => {
                                error!("Failed to get events: {}", e);
                                // Consider implementing a retry mechanism here.
                                tokio::time::sleep(Duration::from_secs(
                                    config_clone.retry_backoff_seconds,
                                ))
                                .await;
                                //	return;
                            },
                        };
                    } else {
                        // If from_block is greater than the latest block, wait before checking again.
                        info!(
                            "Waiting for new blocks. Current block: {}, Latest block: {}",
                            from_block, latest_block
                        );
                    }
                    tokio::time::sleep(Duration::from_secs(config_clone.polling_interval_seconds))
                        .await;
                });
            }
        });

        // Processing Thread
        let process_handle = thread::spawn(move || {
            // TODO(shelbyd): Use try_iter to process the current queue
            // https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_iter
            while let Ok(event) = event_receiver.recv() {
                info!("Processing event: {:?}", event);
                for delegate in event.delegates.iter() {
                    // Update the local database *within a transaction*
                    let power = match process_database_clone.get_delegate_power(delegate) {
                        Ok(opt) => match opt {
                            Some(p) => p,
                            None => 0,
                        },
                        Err(e) => {
                            error!("Failed to retrieve delegate power: {}", e);
                            // Decide how to handle the error, possibly retry or exit
                            // I assume you have implemented a method `get_power` in your Database
                            continue;
                        },
                    };
                    if let Err(e) = tx_sender.send((
                        delegate.clone(),
                        power,
                        event.block_number,
                        event.transaction_hash.clone(),
                    )) {
                        error!("Failed to send event to send thread {}", e);
                        return; // if the send thread has failed we should stop
                    }
                }
            }
        });

        // Sending Thread
        let send_handle = thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().expect("Unable to create Runtime");
            let _enter = rt.enter();
            // TODO(shelbyd): Use try_iter to process the current queue
            // https://doc.rust-lang.org/std/sync/mpsc/struct.Receiver.html#method.try_iter
            while let Ok((delegate, power, block_number, transaction_hash)) = tx_receiver.recv() {
                //	info!("Sending transaction to update power for delegate: {:?}", delegate);
                rt.block_on(async {
			// Send the transaction to the Hub
				match hub_client_clone.update_voting_power(format!("{:?}",delegate), power).await {
						Ok(tx_hash) => {
								// Wait for confirmation before updating the database
								match hub_client_clone.await_transaction_confirmation(&tx_hash, config_clone.max_retries, config_clone.retry_backoff_seconds).await {
										Ok(_) => {
												// Update the last processed block number *after* successful confirmation.
												if let Err(e) = send_database_clone.save_last_processed_block(block_number) {
														error!("Failed to update last processed block: {}", e);
														// Consider whether to panic or continue here.
														return;
												}
												info!("Successfully updated voting power for delegate {:?} at block {}", delegate, block_number);
										}
										Err(e) => {
												error!("Failed to confirm transaction: {}", e);
												// Consider implementing a retry mechanism or adding to a dead-letter queue.

										}
								}
						},
						Err(e) => {
								error!("Failed to send transaction: {}", e);
								// Consider implementing a retry mechanism or adding to a dead-letter queue.
						}
				}
			})
            }
        });

        // Optionally, join the threads (if you want the main thread to wait)
        poll_handle.join().expect("Polling thread panicked");
        process_handle.join().expect("Processing thread panicked");
        send_handle.join().expect("Sending thread panicked");
        Ok(())
    }

    pub fn run_command(&self, command: CliCommand) -> Result<(), Error> {
        match command {
            CliCommand::Query => {
                // Display the current synchronized state
                let last_processed_block = self.database.get_last_processed_block()?;
                let delegate_powers = self.database.get_all_delegate_powers()?;

                println!("Last Processed Block: {:?}", last_processed_block);
                println!("Delegate Voting Powers:");
                for (delegate, power) in delegate_powers {
                    println!("  {:?}: {}", delegate, power);
                }
            },
            CliCommand::Reset => {
                // Reset the synchronized state (clear the database)
                self.database.clear_database()?;
                println!("Database reset successfully.");
            },
            CliCommand::Start { block_number } => {
                // Start the synchronization process (optionally from a specific block)
                self.start(block_number)?;
            },
        }
        Ok(())
    }
}
