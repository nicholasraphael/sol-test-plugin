use crate::{accounts_selector, transaction_selector};

use log::*;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::{fs::File, io::Read};

use solana_accountsdb_plugin_interface::accountsdb_plugin_interface::{
    AccountsDbPlugin, ReplicaAccountInfoVersions, ReplicaTransactionInfoVersions, Result,
    SlotStatus,
};
use solana_program::{entrypoint};
use solana_logger;
use solana_measure::measure::Measure;
use solana_metrics::*;

#[derive(Default)]
struct AccountsDbPluginTest {
    accounts_selector: Option<accounts_selector::AccountsSelector>,
    transaction_selector: Option<transaction_selector::TransactionSelector>,
}

impl std::fmt::Debug for AccountsDbPluginTest {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

impl AccountsDbPlugin for AccountsDbPluginTest {
    fn name(&self) -> &'static str {
        "AccountsDbPluginPostgres"
    }

    /// Do initialization for the PostgreSQL plugin.
    ///
    /// # Format of the config file:
    /// * The `accounts_selector` section allows the user to controls accounts selections.
    /// "accounts_selector" : {
    ///     "accounts" : \["pubkey-1", "pubkey-2", ..., "pubkey-n"\],
    /// }
    /// or:
    /// "accounts_selector" = {
    ///     "owners" : \["pubkey-1", "pubkey-2", ..., "pubkey-m"\]
    /// }
    /// Accounts either satisyfing the accounts condition or owners condition will be selected.
    /// When only owners is specified,
    /// all accounts belonging to the owners will be streamed.
    /// The accounts field support wildcard to select all accounts:
    /// "accounts_selector" : {
    ///     "accounts" : \["*"\],
    /// }
    /// None of the transction is stored.
    /// "transaction_selector" : {
    ///     "mentions" : \["pubkey-1", "pubkey-2", ..., "pubkey-n"\],
    /// }
    /// The `mentions` field support wildcard to select all transaction or all 'vote' transactions:
    /// For example, to select all transactions:
    /// "transaction_selector" : {
    ///     "mentions" : \["*"\],
    /// }
    /// To select all vote transactions:
    /// "transaction_selector" : {
    ///     "mentions" : \["all_votes"\],
    /// }
    /// # Examples
    ///
    /// {
    ///    "libpath": "/home/solana/target/release/libsolana_accountsdb_plugin_postgres.so",
    ///    "accounts_selector" : {
    ///       "owners" : ["9oT9R5ZyRovSVnt37QvVoBttGpNqR3J7unkb567NP8k3"]
    ///    }
    /// }

    fn on_load(&mut self, config_file: &str) -> Result<()> {
        solana_logger::setup_with_default("info");
        info!(
            "Loading plugin {:?} from config_file {:?}",
            self.name(),
            config_file
        );
        let mut file = File::open(config_file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let result: serde_json::Value = serde_json::from_str(&contents).unwrap();
        self.accounts_selector = Some(Self::create_accounts_selector_from_config(&result));
        self.transaction_selector = Some(Self::create_transaction_selector_from_config(&result));

        Ok(())
    }

    fn on_unload(&mut self) {
        info!("Unloading plugin: {:?}", self.name());
    }

    fn update_account(
        &mut self,
        account: ReplicaAccountInfoVersions,
        slot: u64,
        is_startup: bool,
    ) -> Result<()> {
        let mut measure_all = Measure::start("accountsdb-plugin-postgres-update-account-main");
        match account {
            ReplicaAccountInfoVersions::V0_0_1(account) => {
                let mut measure_select =
                    Measure::start("accountsdb-plugin-postgres-update-account-select");
                if let Some(accounts_selector) = &self.accounts_selector {
                    if !accounts_selector.is_account_selected(account.pubkey, account.owner) {
                        return Ok(());
                    }
                } else {
                    return Ok(());
                }
                measure_select.stop();
                inc_new_counter_debug!(
                    "accountsdb-plugin-postgres-update-account-select-us",
                    measure_select.as_us() as usize,
                    100000,
                    100000
                );

                debug!(
                    "Updating account {:?} with owner {:?} at slot {:?} using account selector {:?}",
                    bs58::encode(account.pubkey).into_string(),
                    bs58::encode(account.owner).into_string(),
                    slot,
                    self.accounts_selector.as_ref().unwrap()
                );

            }
        }

        measure_all.stop();

        inc_new_counter_debug!(
            "accountsdb-plugin-postgres-update-account-main-us",
            measure_all.as_us() as usize,
            100000,
            100000
        );

        Ok(())
    }

    fn update_slot_status(
        &mut self,
        slot: u64,
        parent: Option<u64>,
        status: SlotStatus,
    ) -> Result<()> {
        info!("Updating slot {:?} at with status {:?}", slot, status);

        Ok(())
    }

    fn notify_end_of_startup(&mut self) -> Result<()> {
        info!("Notifying the end of startup for accounts notifications");

        Ok(())
    }

    fn notify_transaction(
        &mut self,
        transaction_info: ReplicaTransactionInfoVersions,
        slot: u64,
    ) -> Result<()> {

        println!("Logging transaction info");

        Ok(())
    }

    /// Check if the plugin is interested in account data
    /// Default is true -- if the plugin is not interested in
    /// account data, please return false.
    fn account_data_notifications_enabled(&self) -> bool {
        self.accounts_selector
            .as_ref()
            .map_or_else(|| false, |selector| selector.is_enabled())
    }

    /// Check if the plugin is interested in transaction data
    fn transaction_notifications_enabled(&self) -> bool {
        self.transaction_selector
            .as_ref()
            .map_or_else(|| false, |selector| selector.is_enabled())
    }
}

impl AccountsDbPluginTest {
    fn create_accounts_selector_from_config(config: &serde_json::Value) -> accounts_selector::AccountsSelector {
        let accounts_selector = &config["accounts_selector"];

        if accounts_selector.is_null() {
            accounts_selector::AccountsSelector::default()
        } else {
            let accounts = &accounts_selector["accounts"];
            let accounts: Vec<String> = if accounts.is_array() {
                accounts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            let owners = &accounts_selector["owners"];
            let owners: Vec<String> = if owners.is_array() {
                owners
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            accounts_selector::AccountsSelector::new(&accounts, &owners)
        }
    }

    fn create_transaction_selector_from_config(config: &serde_json::Value) -> transaction_selector::TransactionSelector {
        let transaction_selector = &config["transaction_selector"];

        if transaction_selector.is_null() {
            transaction_selector::TransactionSelector::default()
        } else {
            let accounts = &transaction_selector["mentions"];
            let accounts: Vec<String> = if accounts.is_array() {
                accounts
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|val| val.as_str().unwrap().to_string())
                    .collect()
            } else {
                Vec::default()
            };
            transaction_selector::TransactionSelector::new(&accounts)
        }
    }

    pub fn new() -> Self {
        Self::default()
    }
}

#[no_mangle]
#[allow(improper_ctypes_definitions)]
/// # Safety
///
/// This function returns the AccountsDbPluginPostgres pointer as trait AccountsDbPlugin.
pub unsafe extern "C" fn _create_plugin() -> *mut dyn AccountsDbPlugin {
    let plugin = AccountsDbPluginTest::new();
    let plugin: Box<dyn AccountsDbPlugin> = Box::new(plugin);
    Box::into_raw(plugin)
}