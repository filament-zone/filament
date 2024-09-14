use ethereum_types::H160;
use ethers_core::{
    abi::Address,
    k256::ecdsa::SigningKey,
    types::{
        transaction::eip2718::TypedTransaction,
        Block,
        Eip1559TransactionRequest,
        Transaction,
        TransactionRequest,
        TxHash,
    },
};
use ethers_middleware::SignerMiddleware;
use ethers_providers::{Http, Middleware, PendingTransaction, Provider};
use ethers_signers::Wallet;
use futures::StreamExt;
use jsonrpsee::{core::client::ClientT, rpc_params};
use reth_primitives::Bytes;
use sov_ledger_json_client::WsSubscription;
use sov_test_utils::{ApiClient, SimpleStorageContract, TestSpec, TEST_DEFAULT_MAX_FEE};

const GAS: u64 = 900000u64;

pub(crate) struct TestClient {
    pub(crate) chain_id: u64,
    pub(crate) from_addr: Address,
    contract: SimpleStorageContract,
    client: SignerMiddleware<Provider<Http>, Wallet<SigningKey>>,
    node_client: ApiClient,
}

impl TestClient {
    #[allow(dead_code)]
    pub(crate) async fn new(
        chain_id: u64,
        key: Wallet<SigningKey>,
        from_addr: Address,
        contract: SimpleStorageContract,
        rpc_addr: std::net::SocketAddr,
        rest_addr: std::net::SocketAddr,
    ) -> Self {
        let provider =
            Provider::try_from(&format!("http://127.0.0.1:{}", rpc_addr.port())).unwrap();
        let client = SignerMiddleware::new_with_provider_chain(provider, key)
            .await
            .unwrap();

        let node_client = ApiClient::new(rpc_addr.port(), rest_addr.port())
            .await
            .unwrap();

        Self {
            chain_id,
            from_addr,
            contract,
            client,
            node_client,
        }
    }

    pub(crate) async fn send_publish_batch_request(&self) {
        let _: String = self
            .node_client
            .rpc
            .request("eth_publishBatch", rpc_params![])
            .await
            .unwrap();
    }

    pub(crate) async fn deploy_contract(
        &self,
    ) -> Result<PendingTransaction<'_, Http>, Box<dyn std::error::Error>> {
        let req = Eip1559TransactionRequest::new()
            .from(self.from_addr)
            .chain_id(self.chain_id)
            .nonce(0u64)
            .max_priority_fee_per_gas(10u64)
            .max_fee_per_gas(TEST_DEFAULT_MAX_FEE)
            .gas(GAS)
            .data(self.contract.byte_code());

        let typed_transaction = TypedTransaction::Eip1559(req);

        let receipt_req = self
            .client
            .send_transaction(typed_transaction, None)
            .await?;

        Ok(receipt_req)
    }

    pub(crate) async fn deploy_contract_call(&self) -> Result<Bytes, Box<dyn std::error::Error>> {
        let req = Eip1559TransactionRequest::new()
            .from(self.from_addr)
            .chain_id(self.chain_id)
            .nonce(0u64)
            .max_priority_fee_per_gas(10u64)
            .max_fee_per_gas(TEST_DEFAULT_MAX_FEE)
            .gas(GAS)
            .data(self.contract.byte_code());

        let typed_transaction = TypedTransaction::Eip1559(req);

        let receipt_req = self.eth_call(typed_transaction, None).await?;

        Ok(receipt_req)
    }

    pub(crate) async fn set_value_unsigned(
        &self,
        contract_address: H160,
        set_arg: u32,
    ) -> PendingTransaction<'_, Http> {
        // Tx without gas_limit should estimate and include it in send_transaction endpoint
        // Tx without nonce should fetch and include it in send_transaction endpoint
        let req = Eip1559TransactionRequest::new()
            .from(self.from_addr)
            .to(contract_address)
            .chain_id(self.chain_id)
            .data(self.contract.set_call_data(set_arg))
            .max_priority_fee_per_gas(10u64)
            .max_fee_per_gas(TEST_DEFAULT_MAX_FEE);

        let typed_transaction = TypedTransaction::Eip1559(req);

        self.eth_send_transaction(typed_transaction).await
    }

    pub(crate) async fn set_values(
        &self,
        contract_address: H160,
        set_args: Vec<u32>,
        max_priority_fee_per_gas: Option<u64>,
        max_fee_per_gas: Option<u64>,
    ) -> Vec<PendingTransaction<'_, Http>> {
        let mut requests: Vec<_> = Vec::with_capacity(set_args.len());
        let nonce = self.eth_get_transaction_count(self.from_addr).await;

        for (i, set_arg) in set_args.into_iter().enumerate() {
            let req = Eip1559TransactionRequest::new()
                .from(self.from_addr)
                .to(contract_address)
                .chain_id(self.chain_id)
                .nonce(nonce + (i as u64))
                .data(self.contract.set_call_data(set_arg))
                .max_priority_fee_per_gas(max_priority_fee_per_gas.unwrap_or(10u64))
                .max_fee_per_gas(max_fee_per_gas.unwrap_or(TEST_DEFAULT_MAX_FEE))
                .gas(GAS);

            let typed_transaction = TypedTransaction::Eip1559(req);

            requests.push(
                self.client
                    .send_transaction(typed_transaction, None)
                    .await
                    .unwrap(),
            );
        }
        requests
    }

    pub(crate) async fn set_value(
        &self,
        contract_address: H160,
        set_arg: u32,
        max_priority_fee_per_gas: Option<u64>,
        max_fee_per_gas: Option<u64>,
    ) -> PendingTransaction<'_, Http> {
        let nonce = self.eth_get_transaction_count(self.from_addr).await;
        tracing::info!(from = %self.from_addr, nonce, "SmartContract::set_value");

        let req = Eip1559TransactionRequest::new()
            .from(self.from_addr)
            .to(contract_address)
            .chain_id(self.chain_id)
            .nonce(nonce)
            .data(self.contract.set_call_data(set_arg))
            .max_priority_fee_per_gas(max_priority_fee_per_gas.unwrap_or(10u64))
            .max_fee_per_gas(max_fee_per_gas.unwrap_or(TEST_DEFAULT_MAX_FEE))
            .gas(GAS);

        let typed_transaction = TypedTransaction::Eip1559(req);

        self.client
            .send_transaction(typed_transaction, None)
            .await
            .unwrap()
    }

    pub(crate) async fn set_value_call(
        &self,
        contract_address: H160,
        set_arg: u32,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let nonce = self.eth_get_transaction_count(self.from_addr).await;

        // Any type of transaction can be used for eth_call
        let req = TransactionRequest::new()
            .from(self.from_addr)
            .to(contract_address)
            .chain_id(self.chain_id)
            .nonce(nonce)
            .data(self.contract.set_call_data(set_arg))
            .gas_price(10u64);

        let typed_transaction = TypedTransaction::Legacy(req.clone());

        // Estimate gas on rpc
        let gas = self
            .eth_estimate_gas(typed_transaction, Some("latest".to_owned()))
            .await;

        // Call with the estimated gas
        let req = req.gas(gas);
        let typed_transaction = TypedTransaction::Legacy(req);

        let response = self
            .eth_call(typed_transaction, Some("latest".to_owned()))
            .await?;

        Ok(response)
    }

    pub(crate) async fn failing_call(
        &self,
        contract_address: H160,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        let nonce = self.eth_get_transaction_count(self.from_addr).await;

        // Any type of transaction can be used for eth_call
        let req = Eip1559TransactionRequest::new()
            .from(self.from_addr)
            .to(contract_address)
            .chain_id(self.chain_id)
            .nonce(nonce)
            .data(self.contract.failing_function_call_data())
            .max_priority_fee_per_gas(10u64)
            .max_fee_per_gas(TEST_DEFAULT_MAX_FEE)
            .gas(GAS);

        let typed_transaction = TypedTransaction::Eip1559(req);

        self.eth_call(typed_transaction, Some("latest".to_owned()))
            .await
    }

    pub(crate) async fn query_contract(
        &self,
        contract_address: H160,
    ) -> Result<ethereum_types::U256, Box<dyn std::error::Error>> {
        let nonce = self.eth_get_transaction_count(self.from_addr).await;

        let req = Eip1559TransactionRequest::new()
            .from(self.from_addr)
            .to(contract_address)
            .chain_id(self.chain_id)
            .nonce(nonce)
            .data(self.contract.get_call_data())
            .gas(GAS);

        let typed_transaction = TypedTransaction::Eip1559(req);

        let response = self.client.call(&typed_transaction, None).await?;

        let resp_array: [u8; 32] = response.to_vec().try_into().unwrap();
        Ok(ethereum_types::U256::from(resp_array))
    }

    pub(crate) async fn eth_accounts(&self) -> Vec<Address> {
        self.node_client
            .rpc
            .request("eth_accounts", rpc_params![])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_send_transaction(
        &self,
        tx: TypedTransaction,
    ) -> PendingTransaction<'_, Http> {
        self.client
            .provider()
            .send_transaction(tx, None)
            .await
            .unwrap()
    }

    pub(crate) async fn eth_chain_id(&self) -> u64 {
        let chain_id: ethereum_types::U64 = self
            .node_client
            .rpc
            .request("eth_chainId", rpc_params![])
            .await
            .unwrap();

        chain_id.as_u64()
    }

    pub(crate) async fn eth_get_balance(&self, address: Address) -> ethereum_types::U256 {
        self.node_client
            .rpc
            .request("eth_getBalance", rpc_params![address, "latest"])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_get_storage_at(
        &self,
        address: Address,
        index: ethereum_types::U256,
    ) -> ethereum_types::U256 {
        self.node_client
            .rpc
            .request("eth_getStorageAt", rpc_params![address, index, "latest"])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_get_code(&self, address: Address) -> Bytes {
        self.node_client
            .rpc
            .request("eth_getCode", rpc_params![address, "latest"])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_get_transaction_count(&self, address: Address) -> u64 {
        let count: ethereum_types::U64 = self
            .node_client
            .rpc
            .request("eth_getTransactionCount", rpc_params![address, "latest"])
            .await
            .unwrap();

        count.as_u64()
    }

    pub(crate) async fn eth_gas_price(&self) -> ethereum_types::U256 {
        self.node_client
            .rpc
            .request("eth_gasPrice", rpc_params![])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_get_block_by_number(
        &self,
        block_number: Option<String>,
    ) -> Block<TxHash> {
        self.node_client
            .rpc
            .request("eth_getBlockByNumber", rpc_params![block_number, false])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_get_block_by_number_with_detail(
        &self,
        block_number: Option<String>,
    ) -> Block<Transaction> {
        self.node_client
            .rpc
            .request("eth_getBlockByNumber", rpc_params![block_number, true])
            .await
            .unwrap()
    }

    pub(crate) async fn eth_call(
        &self,
        tx: TypedTransaction,
        block_number: Option<String>,
    ) -> Result<Bytes, Box<dyn std::error::Error>> {
        self.node_client
            .rpc
            .request("eth_call", rpc_params![tx, block_number])
            .await
            .map_err(|e| e.into())
    }

    pub(crate) async fn eth_estimate_gas(
        &self,
        tx: TypedTransaction,
        block_number: Option<String>,
    ) -> u64 {
        let gas: ethereum_types::U64 = self
            .node_client
            .rpc
            .request("eth_estimateGas", rpc_params![tx, block_number])
            .await
            .unwrap();

        gas.as_u64()
    }

    pub(crate) async fn subscribe_for_slots(&self) -> WsSubscription<u64> {
        Ok(self
            .node_client
            .ledger
            .subscribe_slots()
            .await?
            .map(|s| s.map(|s| s.number))
            .boxed())
    }

    pub(crate) async fn send_transactions_and_wait_slot(
        &self,
        transactions: &[sov_modules_api::transaction::Transaction<TestSpec>],
    ) -> anyhow::Result<()> {
        let mut slot_subscription = self.node_client.ledger.subscribe_slots().await?;

        self.node_client
            .sequencer
            .publish_batch_with_serialized_txs(transactions)
            .await?;

        let _ = slot_subscription.next().await;

        Ok(())
    }
}