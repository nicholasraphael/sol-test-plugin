use log::*;
use std::collections::HashSet;
use solana_sdk::pubkey::Pubkey;

pub struct TransactionSelector {
    pub mentioned_addresses: HashSet<Vec<u8>>,
    pub select_all_transactions: bool,
    pub select_all_vote_transactions: bool,
}

impl TransactionSelector {
    pub fn default() -> Self {
        Self {
            mentioned_addresses: HashSet::default(),
            select_all_transactions: false,
            select_all_vote_transactions: false,
        }
    }

    /// Create a selector based on the mentioned addresses
    /// To select all transactions use ["*"] or ["all"]
    /// To select all vote transactions, use ["all_votes"]
    /// To select transactions mentioning specific addresses use ["<pubkey1>", "<pubkey2>", ...]
    pub fn new(mentioned_addresses: &[String]) -> Self {
        info!(
            "Creating TransactionSelector from addresses: {:?}",
            mentioned_addresses
        );

        let select_all_transactions = mentioned_addresses
            .iter()
            .any(|key| key == "*" || key == "all");
        if select_all_transactions {
            return Self {
                mentioned_addresses: HashSet::default(),
                select_all_transactions,
                select_all_vote_transactions: true,
            };
        }
        let select_all_vote_transactions = mentioned_addresses.iter().any(|key| key == "all_votes");
        if select_all_vote_transactions {
            return Self {
                mentioned_addresses: HashSet::default(),
                select_all_transactions,
                select_all_vote_transactions: true,
            };
        }

        let mentioned_addresses = mentioned_addresses
            .iter()
            .map(|key| bs58::decode(key).into_vec().unwrap())
            .collect();

        Self {
            mentioned_addresses,
            select_all_transactions: false,
            select_all_vote_transactions: false,
        }
    }

    /// Check if a transaction is of interest.
    pub fn is_transaction_selected(
        &self,
        is_vote: bool,
        mentioned_addresses: Box<dyn Iterator<Item = &Pubkey> + '_>,
    ) -> bool {
        if !self.is_enabled() {
            return false;
        }

        if self.select_all_transactions || (self.select_all_vote_transactions && is_vote) {
            return true;
        }
        for address in mentioned_addresses {
            if self.mentioned_addresses.contains(address.as_ref()) {
                return true;
            }
        }
        false
    }

    /// Check if any transaction is of interest at all
    pub fn is_enabled(&self) -> bool {
        self.select_all_transactions
            || self.select_all_vote_transactions
            || !self.mentioned_addresses.is_empty()
    }
}