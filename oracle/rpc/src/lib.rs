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
pub trait OracleApi<BlockHash, ProviderId, AccountId, Key, Value> {
	#[rpc(name = "oracle_get")]
	fn get(
		&self,
		provider_id: ProviderId,
		key: Key,
		at: Option<BlockHash>,
	) -> Result<Option<Value>>;

	#[rpc(name = "oracle_get_polkafoundry")]
	fn get_polkafoundry(
		&self,
		provider_id: ProviderId,
		key: Key,
		feeder: AccountId,
		at: Option<BlockHash>,
	) -> Result<Option<Value>>;

	#[rpc(name = "oracle_get_concrete")]
	fn get_concrete(
		&self,
		provider_id: ProviderId,
		key: Key,
		feeder: AccountId,
		at: Option<BlockHash>,
	) -> Result<Option<Value>>;

	#[rpc(name = "oracle_get_all_values")]
	fn get_all_values(
		&self,
		provider_id: ProviderId,
		at: Option<BlockHash>,
	) -> Result<Vec<(Key, Option<Value>)>> ;
}

impl<C, Block, ProviderId, AccountId, Key, Value> OracleApi<<Block as BlockT>::Hash, ProviderId, AccountId, Key, Value> for Oracle<C, Block> where
	Block: BlockT,
	C: Send + Sync + 'static + ProvideRuntimeApi<Block> + HeaderBackend<Block>,
	C::Api: OracleRuntimeApi<Block, ProviderId, AccountId, Key, Value>,
	ProviderId: Codec,
	AccountId: Codec,
	Key: Codec,
	Value: Codec,
{
	fn get(
		&self,
		provider_id: ProviderId,
		key: Key,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Option<Value>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get(&at, provider_id, key).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_polkafoundry(
		&self,
		provider_id: ProviderId,
		key: Key,
		feeder: AccountId,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Option<Value>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_polkafoundry(&at, provider_id, key, feeder).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get PolkaFoundry value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_concrete(
		&self,
		provider_id: ProviderId,
		key: Key,
		feeder: AccountId,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Option<Value>> {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_concrete(&at, provider_id, key, feeder).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get concrete value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}

	fn get_all_values(
		&self,
		provider_id: ProviderId,
		at: Option<<Block as BlockT>::Hash>
	) -> Result<Vec<(Key, Option<Value>)>>  {
		let api = self.client.runtime_api();
		let at = BlockId::hash(at.unwrap_or(
			// If the block hash is not supplied assume the best block.
			self.client.info().best_hash,
		));
		api.get_all_values(&at, provider_id).map_err(|e| RpcError {
			code: ErrorCode::ServerError(Error::RuntimeError.into()),
			message: "Unable to get all value.".into(),
			data: Some(format!("{:?}", e).into()),
		})
	}
}
