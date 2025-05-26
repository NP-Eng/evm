mod mock;
use evm::{
	backend::{OverlayedBackend},
	interpreter::error::ExitError,
	standard::{Config, Etable, EtableResolver, Invoker, TransactArgs, TransactValue},
};
use mock::{MockAccount, MockBackend};
use primitive_types::{H160, H256, U256};
use sha3::{Keccak256, Digest};


fn transact(
	config: &Config,
	args: TransactArgs,
	overlayed_backend: &mut OverlayedBackend<MockBackend>,
) -> Result<TransactValue, ExitError> {
	let gas_etable = Etable::single(evm::standard::eval_gasometer);
	let exec_etable = Etable::runtime();
	let etable = (gas_etable, exec_etable);
	let resolver = EtableResolver::new(config, &(), &etable);
	let invoker = Invoker::new(config, &resolver);

	evm::transact(args.clone(), Some(4), overlayed_backend, &invoker)
}

#[test]
fn shielding_transaction() {
	let mut backend = MockBackend::default();
	backend.state.insert(
		H160::from_low_u64_be(1),
		MockAccount {
			balance: U256::from(1_000_000_000),
			code: vec![],
			nonce: U256::one(),
			storage: Default::default(),
			transient_storage: Default::default(),
		},
	);
	let config = Config::frontier();
	let mut overlayed_backend = OverlayedBackend::new(backend, Default::default(), &config);

	let mut hasher = Keccak256::new();
        hasher.update(b"test");
        let test_note = H256::from_slice(&hasher.finalize());
	let args = TransactArgs::Call { 
		caller: H160::from_low_u64_be(1), 
		address: config.shielding_pool_address,
		value: config.shielding_unit,
		data: test_note.0.to_vec(),
		gas_limit: U256::from(400_000),
		gas_price: U256::from(1), 
		access_list: vec![] 
	};


	let result = transact(&config, args, &mut overlayed_backend);

	// Apply overlayed changeset
	let (mut backend, changeset) = overlayed_backend.deconstruct();
	backend.apply_overlayed(&changeset);

	// Verify insertion of note in the merkle tree
	assert!(result.is_ok());
	assert_eq!(backend.merkle_tree.size(), 1);
}

