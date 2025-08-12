use node_warden::{AccessKeyBadgeData, AccessKeyPermissions};
use scrypto_test::prelude::*;

#[derive(Clone, Debug)]
pub struct Account {
    public_key: Secp256k1PublicKey,
    account_address: ComponentAddress,
}

impl Into<Account>
    for (Secp256k1PublicKey, Secp256k1PrivateKey, ComponentAddress)
{
    fn into(self) -> Account {
        Account {
            public_key: self.0,
            account_address: self.2,
        }
    }
}

struct CustomTestEnvironment {
    runner: DefaultLedgerSimulator,
    accounts: Vec<Account>,
    package_address: PackageAddress,
}

struct NodeWardenInstantiateResult {
    component_address: ComponentAddress,
    owner_badge: NonFungibleGlobalId,
    access_key_resource: ResourceAddress,
}
impl CustomTestEnvironment {
    fn new() -> Self {
        let mut test_runner =
            LedgerSimulatorBuilder::new().without_kernel_trace().build();

        let mut accounts: Vec<Account> = vec![];
        for _ in 0..=5 {
            accounts.push(test_runner.new_allocated_account().into());
        }

        let package_address = test_runner.compile_and_publish(this_package!());

        CustomTestEnvironment {
            runner: test_runner,
            accounts: accounts,
            package_address,
        }
    }

    fn create_validator_component(
        &mut self,
        account: &Account,
    ) -> Result<(ComponentAddress, NonFungibleGlobalId), RuntimeError> {
        let private_key = Secp256k1PrivateKey::from_u64(1).unwrap();
        let public_key = private_key.public_key();
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .get_free_xrd_from_faucet()
            .take_all_from_worktop(XRD, "payment")
            .create_validator(public_key, dec!(0.1), "payment")
            .deposit_entire_worktop(account.account_address)
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &self.accounts[0].public_key,
            )],
        );

        let commit_success = receipt.expect_commit_success();
        let component_address = commit_success.new_component_addresses()[0];
        let validator_owner_badge = commit_success.new_resource_addresses();
        println!("Validator Owner Badge: {:?}", validator_owner_badge);

        println!(
            "Vault Balance Changes: {:?}",
            commit_success.vault_balance_changes()
        );

        let owner_badge = commit_success
            .vault_balance_changes()
            .iter()
            .find(|(_, (resource_address, _))| {
                *resource_address == VALIDATOR_OWNER_BADGE
            })
            .unwrap();

        println!("Owner Badge ID: {:?}", owner_badge);

        let mut owner_badge_id = owner_badge.1 .1.clone();
        let owner_badge_nft_id =
            owner_badge_id.added_non_fungibles().first().unwrap();

        println!("Owner Badge NFT_ID: {:?}", owner_badge_nft_id);

        let owner_badge_global_id = NonFungibleGlobalId::new(
            VALIDATOR_OWNER_BADGE,
            owner_badge_nft_id.clone(),
        );

        Ok((component_address, owner_badge_global_id))
    }

    fn instantiate_node_warden(
        &mut self,
        account: &Account,
    ) -> NodeWardenInstantiateResult {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .call_function(
                self.package_address,
                "NodeWarden",
                "new",
                manifest_args!(None::<ComponentAddress>),
            )
            .deposit_entire_worktop(account.account_address)
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );
        let commit_success = receipt.expect_commit_success();

        println!(
            "created resources: {:#?}",
            commit_success.new_resource_addresses()
        );

        for resource_address in commit_success.new_resource_addresses() {
            let encoded_address =
                AddressBech32Encoder::new(&NetworkDefinition::simulator())
                    .encode(&resource_address.to_vec())
                    .unwrap();
            println!("Resource Address: {}", encoded_address);
            let metadata = self
                .runner
                .get_metadata(resource_address.clone().into(), "name")
                .unwrap();
            println!("Resource Metadata: {:#?}", metadata);
        }

        let node_warden_component_address =
            commit_success.new_component_addresses()[0];

        let node_warden_owner_badge_resource =
            commit_success.new_resource_addresses()[0];
        let access_key_badge_resource =
            commit_success.new_resource_addresses()[1];

        let owner_key_local_id = commit_success
            .vault_balance_changes()
            .iter()
            .find(|(_, (resource_address, _))| {
                *resource_address == node_warden_owner_badge_resource
            })
            .unwrap();
        let mut owner_key_local_id = owner_key_local_id.1 .1.clone();

        let owner_key_local_id =
            owner_key_local_id.added_non_fungibles().first().unwrap();

        let owner_key_global_id = NonFungibleGlobalId::new(
            node_warden_owner_badge_resource,
            owner_key_local_id.clone(),
        );

        NodeWardenInstantiateResult {
            component_address: node_warden_component_address,
            owner_badge: owner_key_global_id,
            access_key_resource: access_key_badge_resource,
        }
    }

    fn create_access_key_badge(
        &mut self,
        component_address: ComponentAddress,
        owner_non_fungible_id: NonFungibleGlobalId,
        by_account: &Account,
        to_account: &Account,
        permissions: AccessKeyPermissions,
        access_key_resource: ResourceAddress,
    ) -> Result<NonFungibleGlobalId, RuntimeError> {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                by_account.account_address,
                owner_non_fungible_id,
            )
            .call_method(
                component_address,
                "create_access_key_badge",
                manifest_args!(permissions),
            )
            .deposit_entire_worktop(to_account.account_address)
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![
                NonFungibleGlobalId::from_public_key(&by_account.public_key),
                NonFungibleGlobalId::from_public_key(&to_account.public_key),
            ],
        );

        let commit_success = receipt.expect_commit_success();

        // get the local nft id of the newly created access key badge
        let access_key = commit_success.vault_balance_changes();
        println!("Access Key Badge: {:?}", access_key);

        let access_key_badge_nft_id = commit_success
            .vault_balance_changes()
            .iter()
            .find(|(_, (resource_address, _))| {
                *resource_address == access_key_resource
            })
            .unwrap();

        let mut access_key_badge_nft_id = access_key_badge_nft_id.1 .1.clone();
        let access_key_local_id = access_key_badge_nft_id
            .added_non_fungibles()
            .first()
            .unwrap();

        let access_key_global_id = NonFungibleGlobalId::new(
            access_key_resource,
            access_key_local_id.clone(),
        );

        Ok(access_key_global_id)
    }

    fn deposit_validator_owner_badge(
        &mut self,
        component_address: ComponentAddress,
        account: &Account,
        validator_owner_badge: NonFungibleGlobalId,
        node_warden_owner_badge: NonFungibleGlobalId,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .withdraw_non_fungible_from_account(
                account.account_address,
                validator_owner_badge.clone(),
            )
            .take_non_fungibles_from_worktop(
                validator_owner_badge.resource_address(),
                vec![validator_owner_badge.local_id().clone()],
                "badge",
            )
            .create_proof_from_account_of_non_fungible(
                account.account_address,
                node_warden_owner_badge,
            )
            .call_method_with_name_lookup(
                component_address,
                "deposit_validator_owner_badge",
                |lookup| manifest_args!(lookup.bucket("badge")),
            )
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        println!("Transaction Receipt: {:?}", receipt);
        let commit_success: &CommitResult = receipt.expect_commit_success();
        println!(
            "Balance Changes: {:?}",
            commit_success.vault_balance_changes()
        );
    }

    fn withdraw_validator_owner_badge(
        &mut self,
        component_address: ComponentAddress,
        account: &Account,
        node_warden_owner_badge: NonFungibleGlobalId,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                account.account_address,
                node_warden_owner_badge,
            )
            .call_method(
                component_address,
                "withdraw_validator_owner_badge",
                manifest_args!(),
            )
            .deposit_entire_worktop(account.account_address)
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&account.public_key)],
        );

        println!("Transaction Receipt: {:?}", receipt);
        let commit_success: &CommitResult = receipt.expect_commit_success();
        println!(
            "Balance Changes: {:?}",
            commit_success.vault_balance_changes()
        );
    }

    fn update_fee(
        &mut self,
        component_address: ComponentAddress,
        admin_account: &Account,
        access_key_global_id: NonFungibleGlobalId,
        fee: Decimal,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                admin_account.account_address,
                access_key_global_id.clone(),
            )
            .pop_from_auth_zone("proof")
            .call_method_with_name_lookup(
                component_address,
                "update_fee",
                |lookup| manifest_args!(lookup.proof("proof"), fee),
            )
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        println!("Transaction Receipt: {:?}", receipt);
        receipt.expect_commit_success();
    }

    fn update_access_key_badge_permissions(
        &mut self,
        component_address: ComponentAddress,
        owner_account: &Account,
        access_key_global_id: NonFungibleGlobalId,
        admin_badge_global_id: NonFungibleGlobalId,
        permission: &str,
        allow: bool,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                owner_account.account_address,
                admin_badge_global_id.clone(),
            )
            .call_method(
                component_address,
                "update_access_key_badge_permissions",
                manifest_args!(
                    access_key_global_id.local_id(),
                    permission.to_string(),
                    allow
                ),
            )
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &owner_account.public_key,
            )],
        );

        println!("Transaction Receipt: {:?}", receipt);
        receipt.expect_commit_success();
    }

    fn register(
        &mut self,
        component_address: ComponentAddress,
        admin_account: &Account,
        access_key_global_id: NonFungibleGlobalId,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                admin_account.account_address,
                access_key_global_id.clone(),
            )
            .pop_from_auth_zone("proof")
            .call_method_with_name_lookup(
                component_address,
                "register",
                |lookup| manifest_args!(lookup.proof("proof")),
            )
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        println!("Transaction Receipt: {:?}", receipt);
        receipt.expect_commit_success();
    }
    fn unregister(
        &mut self,
        component_address: ComponentAddress,
        admin_account: &Account,
        access_key_global_id: NonFungibleGlobalId,
    ) {
        let manifest = ManifestBuilder::new()
            .lock_fee_from_faucet()
            .create_proof_from_account_of_non_fungible(
                admin_account.account_address,
                access_key_global_id.clone(),
            )
            .pop_from_auth_zone("proof")
            .call_method_with_name_lookup(
                component_address,
                "unregister",
                |lookup| manifest_args!(lookup.proof("proof")),
            )
            .build();

        let receipt = self.runner.execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(
                &admin_account.public_key,
            )],
        );

        println!("Transaction Receipt: {:?}", receipt);
        receipt.expect_commit_success();
    }
}

struct SimpleSetupStuff {
    env: CustomTestEnvironment,
    validator_component: ComponentAddress,
    validator_owner: Account,
    admin1: Account,
    node_warden_results: NodeWardenInstantiateResult,
    admin1_access_key: NonFungibleGlobalId,
}

fn simple_setup(permissions: AccessKeyPermissions) -> SimpleSetupStuff {
    let mut env = CustomTestEnvironment::new();
    let validator_owner = env.accounts[0].clone();
    let (validator_component, owner_badge_nft_id) =
        env.create_validator_component(&validator_owner).unwrap();

    let node_warden_results = env.instantiate_node_warden(&validator_owner);

    let admin1 = env.accounts[1].clone();

    let access_key_global_id = env
        .create_access_key_badge(
            node_warden_results.component_address.clone(),
            node_warden_results.owner_badge.clone(),
            &validator_owner,
            &admin1,
            permissions,
            node_warden_results.access_key_resource.clone(),
        )
        .unwrap();

    env.deposit_validator_owner_badge(
        node_warden_results.component_address.clone(),
        &validator_owner,
        owner_badge_nft_id,
        node_warden_results.owner_badge.clone(),
    );
    return SimpleSetupStuff {
        env,
        validator_owner,
        validator_component,
        admin1,
        node_warden_results,
        admin1_access_key: access_key_global_id,
    };
}

#[test]
fn create_env() {
    let SimpleSetupStuff {
        mut env,
        validator_owner,
        admin1,
        node_warden_results,
        admin1_access_key,
        validator_component,
    } = simple_setup(AccessKeyPermissions {
        update_fee: true,
        ..Default::default()
    });

    env.update_fee(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key.clone(),
        dec!(0.069),
    );

    let validator_info = env.runner.get_validator_info(validator_component);
    assert!(
        validator_info
            .validator_fee_change_request
            .unwrap()
            .new_fee_factor
            == dec!(0.069)
    );
}

#[test]
fn successfully_update_fee() {
    let SimpleSetupStuff {
        mut env,
        validator_owner: _,
        admin1,
        node_warden_results,
        admin1_access_key,
        validator_component,
    } = simple_setup(AccessKeyPermissions {
        update_fee: true,
        ..Default::default()
    });

    env.update_fee(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key.clone(),
        dec!(0.069),
    );

    let validator_info = env.runner.get_validator_info(validator_component);
    assert!(
        validator_info
            .validator_fee_change_request
            .unwrap()
            .new_fee_factor
            == dec!(0.069)
    );
}

#[test]
#[should_panic(
    expected = "Access key badge does not have permission for: unregister"
)]
fn unsuccessfully_unregister() {
    let SimpleSetupStuff {
        mut env,
        validator_owner: _,
        admin1,
        node_warden_results,
        admin1_access_key,
        validator_component: _,
    } = simple_setup(AccessKeyPermissions {
        update_fee: true,
        unregister: false, // not allowed to unregister
        ..Default::default()
    });

    env.unregister(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key.clone(),
    );
}

#[test]
fn update_fee_then_withdraw_validator_owner_badge() {
    let SimpleSetupStuff {
        mut env,
        validator_owner,
        admin1,
        node_warden_results,
        admin1_access_key,
        validator_component: _,
    } = simple_setup(AccessKeyPermissions {
        update_fee: true,
        unregister: false, // not allowed to unregister
        ..Default::default()
    });

    env.update_fee(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key.clone(),
        dec!(0.069),
    );

    env.withdraw_validator_owner_badge(
        node_warden_results.component_address,
        &validator_owner,
        node_warden_results.owner_badge.clone(),
    );
}

#[test]
#[should_panic(
    expected = "Expected success but was failure: Failure(SystemModuleError(AuthError(Unauthorized(Unauthorized { failed_access_rules"
)]
fn admin_unsuccessfully_withdraw_validator_owner_badge() {
    let SimpleSetupStuff {
        mut env,
        validator_owner,
        admin1,
        node_warden_results,
        admin1_access_key,
        validator_component: _,
    } = simple_setup(AccessKeyPermissions {
        update_fee: true,
        unregister: false, // not allowed to unregister
        ..Default::default()
    });

    env.withdraw_validator_owner_badge(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key,
    );
}

#[test]
fn enable_unregister_and_successfully_do_it() {
    let SimpleSetupStuff {
        mut env,
        validator_owner,
        admin1,
        node_warden_results,
        admin1_access_key,
        validator_component,
    } = simple_setup(AccessKeyPermissions {
        update_fee: true,
        unregister: false, // don't allowed unregister AT FIRST
        register: false,   // don't allowed register AT FIRST
        ..Default::default()
    });

    env.update_access_key_badge_permissions(
        node_warden_results.component_address,
        &validator_owner,
        admin1_access_key.clone(),
        node_warden_results.owner_badge.clone(),
        "unregister",
        true,
    );

    let non_fungible_data: AccessKeyBadgeData =
        env.runner.get_non_fungible_data(
            admin1_access_key.resource_address(),
            admin1_access_key.local_id().clone(),
        );
    assert!(
        non_fungible_data.permissions.unregister,
        "Unregister permission should now be true"
    );

    env.unregister(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key.clone(),
    );

    let validator_info = env.runner.get_validator_info(validator_component);

    assert!(
        !validator_info.is_registered,
        "Validator should not be registered after unregistering"
    );

    // now enable the register permission

    env.update_access_key_badge_permissions(
        node_warden_results.component_address,
        &validator_owner,
        admin1_access_key.clone(),
        node_warden_results.owner_badge.clone(),
        "register",
        true,
    );

    let non_fungible_data: AccessKeyBadgeData =
        env.runner.get_non_fungible_data(
            admin1_access_key.resource_address(),
            admin1_access_key.local_id().clone(),
        );
    assert!(
        non_fungible_data.permissions.register,
        "Register permission should now be true"
    );

    env.register(
        node_warden_results.component_address,
        &admin1,
        admin1_access_key.clone(),
    );

    let validator_info = env.runner.get_validator_info(validator_component);

    assert!(
        validator_info.is_registered,
        "Validator should be registered after registering"
    );
}
