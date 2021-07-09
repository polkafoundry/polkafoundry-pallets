use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{
	generic::BlockId,
	traits::{Block as BlockT},
};
pub use pkfp_oracle_runtime_api::OracleApi as OracleRuntimeApi;

pub enum Error {
	RuntimeError,
}

impl From<Error> for i64 {
	fn from(e: Error) -> i64 {
		match e {
			Error::RuntimeError => 1,
		}
	}
}

/// An implementation of contract specific RPC methods.
pub struct Oracle<C, B> {
	client: Arc<C>,
	_marker: std::marker::PhantomData<B>,
}

impl<C, B> Oracle<C, B> {
	/// Create new `Contracts` with the given reference to the client.
	pub fn new(client: Arc<C>) -> Self {
		Oracle {
			client,
			_marker: Default::default(),
		}
	}
}

#[rpc]
pub trait OracleApi<BlockHash, AccountId, Key, Value> {
	#[rpc(name = "oracle_get")]
	fn get(
		&self,
		key: Key,
		at: Option<BlockHash>,
	) -> Result<Option<Value>>;

	#[rpc(name = "oracle_get_polkafoundry")]
	fn get_polkafoundry(
		&self,
		key: Key,
		at: Option<BlockHash>,
	) -> Result<Option<Value>>;

	#[rpc(name = "oracle_get_concrete")]
	fn get_concrete(
		&self,
		key: Key,
		feeder: AccountId,
		at: Option<BlockHash>,
	) -> Result<Option<Value>>;

	#[rpc(name = "oracle_get_all_values")]
	fn get_all_values(
		&self,
		at: Option<BlockHash>,
	) -> Result<Vec<(Key, Option<Value>)>> ;
}

impl<C, Block, AccountId, Key, Value> OracleApi<<Block as BlockT>::Hash, AccountId, Key, Value> for Oracle<C, Block> where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: OracleRuntimeApi<Block, AccountId, Key, Value>,
	AccountId: Codec,
	Key: Codec,
	Value: Codec,
{
	fn get(
		&self,
		key: Key,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Option<Value>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get(&at, key).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_polkafoundry(
		&self,
		key: Key,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Option<Value>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_polkafoundry(&at, key).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get PolkaFoundry value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_concrete(
		&self,
		key: Key,
		feeder: AccountId,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Option<Value>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_concrete(&at, key, feeder).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get concrete value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_all_values(
		&self,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Vec<(Key, Option<Value>)>>  {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_all_values(&at).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get all value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
