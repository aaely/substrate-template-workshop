use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use sp_api::ProvideRuntimeApi;
use std::sync::Arc;
use sp_blockchain::HeaderBackend;
use social_media_query_runtime_api::AnalyticalQueryRPC as AnalyticalQueryRPCAPI;

#[rpc]
pub trait AnalyticalQueryRPC<BlockHash> {

    #[rpc(name = "three_search_handle")]
    fn get_three_accts(&self, at: Option<BlockHash>) -> Result<(Vec<u8>, u128)>;

    #[rpc(name = "get_fifty_accounts_by_text")]
    fn get_fifty_accts(&self, at: Option<BlockHash>) -> Result<(Vec<u8>, u128)>;

}

pub struct Queries<C, M> {
    client: Arc<C>,
	_marker: std::marker::PhantomData<M>,
}

impl<C, M> Queries<C, M> {
    pub fn new(client: Arc<C>) -> Self {
		Self {
			client,
			_marker: Default::default(),
		}
	}
}

impl<C, Block> AnalyticalQueryRPC<<Block as BlockT>::Hash> for Queries<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: AnalyticalQueryRPCAPI<Block>,
{
    fn get_three_accts(&self, at: Option<<Block as BlockT>::Hash>) ->Result<(Vec<u8> ,u128)>  {
        let api = self.client.runtime_api();
        let at = BlockId::Hash(at.unwrap_or_else(||
            self.client.info().best_hash));
        let runtime_api_result = api.get_three_accts(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1234),
            message: "Something went wrong!".into(),
            data: Some(format!("{:?}",e)),
        })
    }

    fn get_fifty_accts(&self, at: Option<<Block as BlockT>::Hash>) ->Result<(Vec<u8> ,u128)>  {
        let api = self.client.runtime_api();
        let at = BlockId::Hash(at.unwrap_or_else(||
            self.client.info().best_hash));
        let runtime_api_result = api.get_fifty_accts(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(1234),
            message: "Something went wrong!".into(),
            data: Some(format!("{:?}",e)),
        })
    }
}