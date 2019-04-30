use bincode::{deserialize, serialize};
use failure::ResultExt;

use chain_core::init::coin::Coin;
use chain_core::tx::data::address::ExtendedAddr;
use chain_core::tx::data::input::TxoPointer;
use client_common::{ErrorKind, Result, Storage};

const KEYSPACE: &str = "index_unspent_transaction";
/// Exposes functionalities for managing unspent transactions
///
/// Stores `address -> [(TxoPointer, Coin)]` mapping
#[derive(Default, Clone)]
pub struct UnspentTransactionService<S: Storage> {
    storage: S,
}

impl<S> UnspentTransactionService<S>
where
    S: Storage,
{
    /// Creates a new instance of unspent transaction service
    pub fn new(storage: S) -> Self {
        Self { storage }
    }

    /// Retrieves all the unspent transactions for an address
    pub fn get(&self, address: &ExtendedAddr) -> Result<Vec<(TxoPointer, Coin)>> {
        let bytes = self.storage.get(
            KEYSPACE,
            serialize(address).context(ErrorKind::SerializationError)?,
        )?;

        match bytes {
            None => Ok(Default::default()),
            Some(bytes) => Ok(deserialize(&bytes).context(ErrorKind::DeserializationError)?),
        }
    }

    /// Adds an unspent transactions to storage
    pub fn add(
        &self,
        address: &ExtendedAddr,
        unspent_transaction: (TxoPointer, Coin),
    ) -> Result<()> {
        let mut unspent_transactions = self.get(address)?;
        unspent_transactions.push(unspent_transaction);

        self.storage
            .set(
                KEYSPACE,
                serialize(address).context(ErrorKind::SerializationError)?,
                serialize(&unspent_transactions).context(ErrorKind::SerializationError)?,
            )
            .map(|_| ())
    }

    /// Removes an unspent transaction for given address
    pub fn remove(&self, address: &ExtendedAddr, pointer: &TxoPointer) -> Result<()> {
        let mut unspent_transactions = self.get(address)?;
        let mut index = None;

        for (i, (tx_pointer, _)) in unspent_transactions.iter().enumerate() {
            if tx_pointer == pointer {
                index = Some(i);
                break;
            }
        }

        if index.is_some() {
            unspent_transactions.remove(index.unwrap());
        }

        self.storage
            .set(
                KEYSPACE,
                serialize(address).context(ErrorKind::SerializationError)?,
                serialize(&unspent_transactions).context(ErrorKind::SerializationError)?,
            )
            .map(|_| ())
    }
}