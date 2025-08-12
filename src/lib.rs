use scrypto::prelude::*;

#[derive(Display, ScryptoSbor, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum PermissionType {
    StakeAsOwner,
    Register,
    Unregister,
    UpdateKey,
    UpdateFee,
    LockOwnerStakeUnits,
    StartUnlockOwnerStakeUnits,
    FinishUnlockOwnerStakeUnits,
    UpdateAcceptDelegatedStake,
    SignalProtocolUpdateReadiness,

    CreateValidatorOwnerBadgeProof,
}

/// It is actually nice to have this as a struct with bools here, because it is easy
/// to inspect in the explorer. I thought about making it something simple like a u16
/// and doing byte level stuff but ultimately this is easier to inspect.
#[derive(ScryptoSbor, ManifestSbor)]
pub struct AccessKeyPermissions {
    /// These correspond to the validator component's interface
    pub stake_as_owner: bool,
    pub register: bool,
    pub unregister: bool,
    pub update_key: bool,
    pub update_fee: bool,
    pub lock_owner_stake_units: bool,
    pub start_unlock_owner_stake_units: bool,
    pub finish_unlock_owner_stake_units: bool,
    pub update_accept_delegated_stake: bool,
    pub signal_protocol_update_readiness: bool,

    // An extra permission that is quite powerful, and can do all of the above
    pub create_validator_owner_badge_proof: bool,
}

impl Default for AccessKeyPermissions {
    fn default() -> Self {
        Self {
            stake_as_owner: false,
            register: false,
            unregister: false,
            update_key: false,
            update_fee: false,
            lock_owner_stake_units: false,
            start_unlock_owner_stake_units: false,
            finish_unlock_owner_stake_units: false,
            update_accept_delegated_stake: false,
            signal_protocol_update_readiness: false,
            create_validator_owner_badge_proof: false,
        }
    }
}

impl AccessKeyPermissions {
    pub fn is_allowed(&self, permission: &PermissionType) -> bool {
        match permission {
            PermissionType::StakeAsOwner => self.stake_as_owner,
            PermissionType::Register => self.register,
            PermissionType::Unregister => self.unregister,
            PermissionType::UpdateKey => self.update_key,
            PermissionType::UpdateFee => self.update_fee,
            PermissionType::LockOwnerStakeUnits => self.lock_owner_stake_units,
            PermissionType::StartUnlockOwnerStakeUnits => {
                self.start_unlock_owner_stake_units
            }
            PermissionType::FinishUnlockOwnerStakeUnits => {
                self.finish_unlock_owner_stake_units
            }
            PermissionType::UpdateAcceptDelegatedStake => {
                self.update_accept_delegated_stake
            }
            PermissionType::SignalProtocolUpdateReadiness => {
                self.signal_protocol_update_readiness
            }
            PermissionType::CreateValidatorOwnerBadgeProof => {
                self.create_validator_owner_badge_proof
            }
        }
    }

    pub fn update_permission(
        &mut self,
        permission: PermissionType,
        allow: bool,
    ) {
        match permission {
            PermissionType::StakeAsOwner => self.stake_as_owner = allow,
            PermissionType::Register => self.register = allow,
            PermissionType::Unregister => self.unregister = allow,
            PermissionType::UpdateKey => self.update_key = allow,
            PermissionType::UpdateFee => self.update_fee = allow,
            PermissionType::LockOwnerStakeUnits => {
                self.lock_owner_stake_units = allow
            }
            PermissionType::StartUnlockOwnerStakeUnits => {
                self.start_unlock_owner_stake_units = allow
            }
            PermissionType::FinishUnlockOwnerStakeUnits => {
                self.finish_unlock_owner_stake_units = allow
            }
            PermissionType::UpdateAcceptDelegatedStake => {
                self.update_accept_delegated_stake = allow
            }
            PermissionType::SignalProtocolUpdateReadiness => {
                self.signal_protocol_update_readiness = allow
            }
            PermissionType::CreateValidatorOwnerBadgeProof => {
                self.create_validator_owner_badge_proof = allow
            }
        }
    }
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct NodeWardenOwnerBadgeData {
    pub node_warden_component_address: ComponentAddress,
}
#[derive(ScryptoSbor, NonFungibleData)]
pub struct AccessKeyBadgeData {
    pub node_warden_component_address: ComponentAddress,
    #[mutable] // permissions should be mutable, so the owner can update them
    pub permissions: AccessKeyPermissions,
}

#[derive(ScryptoSbor, NonFungibleData)]
pub struct ValidatorOwnerBadgeData {
    pub name: String,
    pub validator: ComponentAddress,
}

#[blueprint]
mod node_warden {
    enable_function_auth! {
        new => rule!(allow_all);
        new_with_address_reservation => rule!(allow_all);
    }
    enable_method_auth! {
        roles {
            component_owner => updatable_by: [];
            key_holder => updatable_by: [];
        },
        methods {
            // Administrative methods callable only by the owner
            deposit_validator_owner_badge => restrict_to: [component_owner];
            create_access_key_badge => restrict_to: [component_owner];
            recall_key_badge => restrict_to: [component_owner];
            burn_key_badge => restrict_to: [component_owner];
            update_access_key_badge_permissions => restrict_to: [component_owner];
            withdraw_validator_owner_badge => restrict_to: [component_owner];

            // These methods mimic the validator component's interface
            // These are public, because their access is not managed by
            // method auth, but in the methods themselves.
            stake_as_owner => PUBLIC;
            register =>  PUBLIC;
            unregister =>  PUBLIC;
            update_key =>  PUBLIC;
            update_fee => PUBLIC;
            lock_owner_stake_units => PUBLIC;
            start_unlock_owner_stake_units => PUBLIC;
            finish_unlock_owner_stake_units => PUBLIC;
            update_accept_delegated_stake => PUBLIC;
            signal_protocol_update_readiness => PUBLIC;

            // Additional method - warning: powerful
            create_validator_owner_badge_proof => PUBLIC;
        }
    }
    struct NodeWarden {
        // The vault holding the validator owner badge - the badge that controls the validator
        validator_owner_badge: NonFungibleVault,
        // The resource manager of the owner badge of this component
        node_warden_owner_badge_resource_manager: NonFungibleResourceManager,
        // The resource manager of the access key badges
        access_key_badge_resource_manager: NonFungibleResourceManager,
        // The component address of the validator component we are
        // currently managing. This can be None, if there is no validator
        // badge currently inside this component.
        validator_address: Option<ComponentAddress>,
    }
    impl NodeWarden {
        /// Creates a new NodeWarden instance.
        pub fn new(
            dapp_definition: Option<ComponentAddress>,
        ) -> (Global<NodeWarden>, NonFungibleBucket) {
            let (address_reservation, _component_address) =
                Runtime::allocate_component_address(NodeWarden::blueprint_id());
            Self::new_with_address_reservation(
                dapp_definition,
                address_reservation,
            )
        }

        /// Creates a new NodeWarden instance with a reserved address.
        pub fn new_with_address_reservation(
            dapp_definition: Option<ComponentAddress>,
            address_reservation: GlobalAddressReservation,
        ) -> (Global<NodeWarden>, NonFungibleBucket) {
            let global_address =
                Runtime::get_reservation_address(&address_reservation);
            let component_address = ComponentAddress::try_from_hex(
                global_address.to_hex().as_str(),
            )
            .expect("Should be able to create component address from global address");

            let node_warden_owner_badge = ResourceBuilder::new_ruid_non_fungible::<NodeWardenOwnerBadgeData>(OwnerRole::Fixed(rule!(require(global_caller(component_address)))))
                .metadata(metadata! (
                    roles {
                        metadata_locker => OWNER;
                        metadata_locker_updater => OWNER;
                        metadata_setter => OWNER;
                        metadata_setter_updater => OWNER;
                    },
                    init {
                        "node_warden_component_address" => component_address, locked;
                        "name" => "NodeWarden Owner Badge", locked;
                        "description" => "NodeWarden Owner badge belongs to the owner of the auth badge which this NodeWarden component is managing", locked;
                        "tags" => vec!["Badge", "Access Control", "Owner Badge", "Validator"], locked;
                    }
                ))
                .mint_roles(mint_roles! (
                    minter => rule!(require(global_caller(component_address)));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles! (
                    burner => rule!(require(global_caller(component_address)));
                    burner_updater => rule!(deny_all);
                ))
                .withdraw_roles(withdraw_roles! (
                    withdrawer => rule!(allow_all);
                    withdrawer_updater => rule!(deny_all);
                ))
                .deposit_roles(deposit_roles! (
                    depositor => rule!(allow_all);
                    depositor_updater => rule!(deny_all);
                ))
                .recall_roles(recall_roles! (
                    recaller => rule!(deny_all);
                    recaller_updater => rule!(deny_all);
                ))
                .freeze_roles(freeze_roles! (
                    freezer => rule!(deny_all);
                    freezer_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles! (
                    non_fungible_data_updater => rule!(require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .mint_initial_supply(vec![NodeWardenOwnerBadgeData {
                node_warden_component_address: component_address,
            }]);

            if let Some(dapp_definition) = dapp_definition {
                node_warden_owner_badge
                    .resource_manager()
                    .set_metadata::<_, Vec<GlobalAddress>>(
                        "dapp_definitions",
                        vec![dapp_definition.into()],
                    );
                node_warden_owner_badge
                    .resource_manager()
                    .lock_metadata("dapp_definitions");
            }

            let access_key_badge_resource_manager = ResourceBuilder::new_ruid_non_fungible::<AccessKeyBadgeData>(OwnerRole::Fixed(rule!(require(global_caller(component_address)))))
                .metadata(metadata! (
                    roles {
                        metadata_locker => OWNER;
                        metadata_locker_updater => OWNER;
                        metadata_setter => OWNER;
                        metadata_setter_updater => OWNER;
                    },
                    init {
                        "node_warden_component_address" => component_address, locked;
                        "name" => "NodeWarden Access Key Badge", locked;
                        "description" => "NodeWarden Access Key badge is the badge used to create proof for the auth badge that the NodeWarden component is managing", locked;
                        "tags" => vec!["Badge", "Access Control", "Key Badge"], locked;
                    }
                ))
                .mint_roles(mint_roles! (
                    minter => rule!(require(global_caller(component_address)) || require(node_warden_owner_badge.resource_address()));
                    minter_updater => rule!(deny_all);
                ))
                .burn_roles(burn_roles! (
                    burner => rule!(allow_all);
                    burner_updater => rule!(deny_all);
                ))
                .withdraw_roles(withdraw_roles! (
                    withdrawer => rule!(require(node_warden_owner_badge.resource_address()));
                    withdrawer_updater => rule!(deny_all);
                ))
                .deposit_roles(deposit_roles! (
                    depositor => rule!(require(node_warden_owner_badge.resource_address()));
                    depositor_updater => rule!(deny_all);
                ))
                .recall_roles(recall_roles! (
                    recaller => rule!(require(global_caller(component_address)) || require(node_warden_owner_badge.resource_address()));
                    recaller_updater => rule!(deny_all);
                ))
                .freeze_roles(freeze_roles! (
                    freezer => rule!(deny_all);
                    freezer_updater => rule!(deny_all);
                ))
                .non_fungible_data_update_roles(non_fungible_data_update_roles! (
                    non_fungible_data_updater => rule!(require(global_caller(component_address)));
                    non_fungible_data_updater_updater => rule!(deny_all);
                ))
                .create_with_no_initial_supply();

            if let Some(dapp_definition) = dapp_definition {
                access_key_badge_resource_manager
                    .set_metadata::<_, Vec<GlobalAddress>>(
                        "dapp_definitions",
                        vec![dapp_definition.into()],
                    );
                access_key_badge_resource_manager
                    .lock_metadata("dapp_definitions");
            }

            let component = Self {
                    validator_owner_badge: NonFungibleVault::new(VALIDATOR_OWNER_BADGE),
                    node_warden_owner_badge_resource_manager: node_warden_owner_badge.resource_manager(),
                    access_key_badge_resource_manager: access_key_badge_resource_manager,
                    validator_address: None
                }
                .instantiate()
                .prepare_to_globalize(OwnerRole::Updatable(rule!(require(global_caller(component_address)))))
                .metadata(metadata! (
                    roles {
                        metadata_locker => OWNER;
                        metadata_locker_updater => OWNER;
                        metadata_setter => OWNER;
                        metadata_setter_updater => OWNER;
                    },
                    init {
                        "name" => "NodeWarden Component", locked;
                        "description" => "A proxy for managing Radix validator components with more advanced authorization features", locked;
                    }
                ))
                .roles(roles! (
                    component_owner => rule!(require(node_warden_owner_badge.resource_address()));
                    key_holder => rule!(require(access_key_badge_resource_manager.address()));
                ))
                .with_address(address_reservation)
                .globalize();

            if let Some(dapp_definition) = dapp_definition {
                component.set_metadata::<_, GlobalAddress>(
                    "dapp_definition",
                    dapp_definition.into(),
                );
                component.lock_metadata("dapp_definition");
            }

            (component, node_warden_owner_badge)
        }

        /// Create a new access key badge with the given permissions.
        pub fn create_access_key_badge(
            &self,
            permissions: AccessKeyPermissions,
        ) -> NonFungibleBucket {
            let access_key_badge_data = AccessKeyBadgeData {
                node_warden_component_address: Runtime::global_address(),
                permissions,
            };
            self.access_key_badge_resource_manager
                .mint_ruid_non_fungible(access_key_badge_data)
        }

        /// Update a permissions of an existing access key badge.
        ///
        /// * `access_key_badge_local_id`: The local ID of the access key badge to update.
        /// * `permission`: The permission to update. This is a snake-case string resembling the permission as
        ///   defined in the [`AccessKeyPermissions`] struct.
        /// * `allow`: Whether to allow or deny the permission.
        pub fn update_access_key_badge_permissions(
            &self,
            access_key_badge_local_id: NonFungibleLocalId,
            permission: String,
            allow: bool,
        ) {
            let mut access_key_badge_data = self
                .access_key_badge_resource_manager
                .get_non_fungible_data::<AccessKeyBadgeData>(
                    &access_key_badge_local_id,
                );
            access_key_badge_data.permissions.update_permission(
                PermissionType::from_str(&permission)
                    .expect("Invalid permission type"),
                allow,
            );

            self.access_key_badge_resource_manager
                .update_non_fungible_data(
                    &access_key_badge_local_id,
                    "permissions",
                    access_key_badge_data.permissions,
                );
        }

        /// Recalls the key badge from the specified vault.
        pub fn recall_key_badge(
            &self,
            vault_address: InternalAddress,
        ) -> NonFungibleBucket {
            let recalled_bucket: Bucket =
                scrypto_decode(&ScryptoVmV1Api::object_call_direct(
                    vault_address.as_node_id(),
                    VAULT_RECALL_IDENT,
                    scrypto_args!(Decimal::ONE),
                ))
                .unwrap();

            recalled_bucket.as_non_fungible()
        }

        /// Burns the given key badge.
        pub fn burn_key_badge(&self, key_badge: NonFungibleBucket) {
            key_badge.burn();
        }

        /// Deposit the validator owner badge into this component.
        ///
        /// # Panics
        /// Panics if the component already has a validator owner badge or if the
        /// given bucket does not contain exactly one validator owner badge.
        pub fn deposit_validator_owner_badge(
            &mut self,
            bucket: NonFungibleBucket,
        ) {
            assert!(
                bucket.resource_address() == VALIDATOR_OWNER_BADGE,
                "The deposited bucket must be a validator owner badge"
            );
            assert!(
                self.validator_owner_badge.is_empty(),
                "There should be no validator owner badge already deposited."
            );
            assert!(
                bucket.amount() == Decimal::ONE,
                "The deposited bucket must contain exactly one validator owner badge."
            );
            self.validator_owner_badge.put(bucket);

            // Update this component to reflect the new validator address
            let nft_data: ValidatorOwnerBadgeData =
                self.validator_owner_badge.non_fungible().data();
            self.validator_address = Some(nft_data.validator);
        }

        /// Withdraw the validator owner badge from this component, leaving it unable to control the validator.
        ///
        /// # Returns
        ///
        /// The withdrawn validator owner badge.
        ///
        /// # Panics
        /// Panics if there is no validator owner badge to withdraw.
        ///
        pub fn withdraw_validator_owner_badge(&mut self) -> NonFungibleBucket {
            // There should be exactly one validator owner badge to withdraw.
            assert!(
                self.validator_owner_badge.amount() == Decimal::ONE,
                "There is no validator owner badge to withdraw"
            );
            // Set the validator address to None - it's no longer managed by this component.
            self.validator_address = None;
            self.validator_owner_badge.take(1)
        }

        // ##############################################################
        // ##### Methods that correspond to the Validator component #####
        // ##############################################################

        /// Stakes an amount of XRD to the Validator in exchange for Stake Units.
        ///
        /// This method is different from the regular `stake` method, in the
        /// fact that it will go through even if the validator does NOT
        /// accept delegated stake.
        ///
        /// * `proof` - The proof of authorization.
        /// * `stake` - The amount of XRD to stake.
        ///
        /// # Returns
        ///
        /// A bucket containing the Validator’s Stake Unit resource
        ///
        /// This function is a thin wrapper of the `stake_as_owner` method on the `Validator` component.
        pub fn stake_as_owner(
            &self,
            proof: NonFungibleProof,
            stake: FungibleBucket,
        ) -> FungibleBucket {
            self.check_proof(proof, PermissionType::StakeAsOwner);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.stake_as_owner(stake)
            })
        }

        /// Registers the Validator to be available to validate and propose transactions in Consensus
        ///
        /// * `proof` - The proof of authorization.
        ///
        /// This function is a thin wrapper of the `register` method on the `Validator` component.
        pub fn register(&self, proof: NonFungibleProof) {
            self.check_proof(proof, PermissionType::Register);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.register()
            })
        }

        /// Unregisters the validator.
        ///
        /// * `proof` - The proof of authorization.
        ///
        /// This function is a thin wrapper of the `unregister` method on the `Validator` component.
        pub fn unregister(&self, proof: NonFungibleProof) {
            self.check_proof(proof, PermissionType::Unregister);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.unregister()
            })
        }

        /// Updates the public key of the Validator.
        ///
        /// * `proof` - The proof of authorization.
        /// * `key` - The public key to replace the Validator’s Consensus public key with.
        ///
        /// This function is a thin wrapper of the `update_key` method on the `Validator` component.
        pub fn update_key(
            &self,
            proof: NonFungibleProof,
            key: Secp256k1PublicKey,
        ) {
            self.check_proof(proof, PermissionType::UpdateKey);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.update_key(key)
            })
        }

        /// Changes the fee for the validator.
        ///
        /// * `proof` - The proof of authorization.
        /// * `new_fee_factor` - A decimal >= 0.0 and <= 1.0 representing the new fee fraction.
        ///
        /// This function is a thin wrapper of the `update_fee` method on the `Validator` component.
        pub fn update_fee(
            &self,
            proof: NonFungibleProof,
            new_fee_factor: Decimal,
        ) {
            self.check_proof(proof, PermissionType::UpdateFee);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.update_fee(new_fee_factor)
            })
        }

        /// Locks the given Stake Units in an internal “delayed withdrawal”
        /// vault (as a way of showing the Owner’s commitment to running the Validator).
        ///
        /// * `proof` - The proof of authorization.
        /// * `stake_unit_bucket` - A bucket of Stake Units
        ///
        /// This function is a thin wrapper of the `lock_owner_stake_units` method on the `Validator` component.
        pub fn lock_owner_stake_units(
            &self,
            proof: NonFungibleProof,
            stake_unit_bucket: FungibleBucket,
        ) {
            self.check_proof(proof, PermissionType::LockOwnerStakeUnits);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.lock_owner_stake_units(stake_unit_bucket)
            })
        }

        /// Begins the process of unlocking the Owner’s Stake Units.
        ///
        /// The requested amount of Stake Units (if available) will be ready for
        /// withdrawal after the Network-configured number of Epochs is reached.
        ///
        /// * `proof` - The proof of authorization.
        /// * `requested_stake_unit_amount` - The amount of Stake Units to start unlocking.
        ///
        /// This function is a thin wrapper of the `start_unlock_owner_stake_units` method on the `Validator` component.
        pub fn start_unlock_owner_stake_units(
            &self,
            proof: NonFungibleProof,
            requested_stake_unit_amount: Decimal,
        ) {
            self.check_proof(proof, PermissionType::StartUnlockOwnerStakeUnits);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator
                    .start_unlock_owner_stake_units(requested_stake_unit_amount)
            })
        }

        /// Finishes the process of unlocking the Owner’s Stake Units by withdrawing all the
        /// pending amounts which have reached their target Epoch and
        /// thus are already available - potentially none.
        ///
        /// * `proof` - The proof of authorization.
        ///
        /// # Returns
        /// A bucket of Stake Units
        ///
        /// This function is a thin wrapper of the `finish_unlock_owner_stake_units` method on the `Validator` component.
        pub fn finish_unlock_owner_stake_units(
            &self,
            proof: NonFungibleProof,
        ) -> FungibleBucket {
            self.check_proof(
                proof,
                PermissionType::FinishUnlockOwnerStakeUnits,
            );
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.finish_unlock_owner_stake_units()
            })
        }

        /// Updates the flag deciding whether the Validator should accept delegated stake.
        ///
        /// * `proof` - The proof of authorization.
        /// * `accept_delegated_stake` - Whether to accept delegated stake.
        ///
        /// This function is a thin wrapper of the `update_accept_delegated_stake` method on the `Validator` component.
        pub fn update_accept_delegated_stake(
            &self,
            proof: NonFungibleProof,
            accept_delegated_stake: bool,
        ) {
            self.check_proof(proof, PermissionType::UpdateAcceptDelegatedStake);
            self.do_with_validator_owner_badge_proof(|validator| {
                validator.update_accept_delegated_stake(accept_delegated_stake)
            })
        }

        /// Signals on ledger what protocol version to potentially change to. Used by Consensus to coordinate protocol updates.
        ///
        /// * `proof` - The proof of authorization.
        /// * `protocol_version_name` - The protocol version to signal readiness for.
        ///
        /// This function is a thin wrapper of the `signal_protocol_update_readiness` method on the `Validator` component.
        pub fn signal_protocol_update_readiness(
            &self,
            proof: NonFungibleProof,
            protocol_version_name: String,
        ) {
            self.check_proof(
                proof,
                PermissionType::SignalProtocolUpdateReadiness,
            );
            self.do_with_validator_owner_badge_proof(|validator| {
                validator
                    .signal_protocol_update_readiness(protocol_version_name)
            })
        }

        // #########################################################
        // ##### Additional methods ################################
        // #########################################################

        /// Create a proof of the validator owner badge.
        /// The proof can be used to call any of the validator component's methods.
        ///
        /// # Warning
        /// By enabling the permission for this method, you allow the caller to create a proof of the validator owner badge.
        /// This essentially gives the caller the ability to perform any action that requires the validator owner badge, including
        /// things like changing fees and withdrawing LSU from the validator's vault.
        pub fn create_validator_owner_badge_proof(
            &self,
            proof: NonFungibleProof,
        ) -> NonFungibleProof {
            self.check_proof(
                proof,
                PermissionType::CreateValidatorOwnerBadgeProof,
            );
            self.validator_owner_badge.create_proof_of_non_fungibles(
                &self.validator_owner_badge.non_fungible_local_ids(1),
            )
        }

        // #########################################################
        // ##### Some non-public helpers ###########################
        // #########################################################

        /// Executes a closure with permission of the validator owner badge.
        ///
        /// * `f` - The closure to execute with authorization of the validator owner badge.
        ///     This closure takes a mutable reference to the `Validator` component, allows
        ///     you to perform actions on the validator.
        ///
        // This is not marked pub, as it is an internal helper
        fn do_with_validator_owner_badge_proof<F, O>(&self, f: F) -> O
        where
            F: FnOnce(&mut Global<Validator>) -> O,
        {
            let mut validator: Global<Validator> = self
                .validator_address
                .expect("The component address of the validator component should be known at this point")
                .into();
            let non_fungible_id =
                self.validator_owner_badge.non_fungible_local_id();
            self.validator_owner_badge.authorize_with_non_fungibles(
                &indexset!(non_fungible_id),
                || f(&mut validator),
            )
        }

        /// Checks the proof against the required permission type.
        ///
        /// * `proof` - The incoming proof to check.
        /// * `permission_type` - The permission type to check for.
        ///
        /// # Panics
        ///
        /// Will panic if the proof is not valid for t`he given permission type.
        /// This happens if:
        /// - The proof is not from the owner badge or the access key badge.
        /// - The access key badge does not have the required permission.
        ///
        // This is not marked pub, as it is an internal helper
        fn check_proof(
            &self,
            proof: NonFungibleProof,
            permission_type: PermissionType,
        ) {
            // It should always be either the owner badge or the access key badge in the proof.
            assert!(
                proof.resource_address()
                    == self.node_warden_owner_badge_resource_manager.address()
                    || proof.resource_address()
                        == self.access_key_badge_resource_manager.address()
            );

            // If the proof is from the access key badge, we need to do the additional permission checks.
            if proof.resource_address()
                == self.access_key_badge_resource_manager.address()
            {
                // Skip the check, we already validated the proof and we know its address here.
                let check_skipped = proof.skip_checking();
                let access_key_badge_data =
                    check_skipped.non_fungible::<AccessKeyBadgeData>();

                // Only allow if the access key badge has the required permission.
                assert!(
                    access_key_badge_data
                        .data()
                        .permissions
                        .is_allowed(&permission_type),
                    "Access key badge does not have permission for: {}",
                    permission_type.to_string()
                );
            }

            // Else, the proof must be from the owner badge. In that case, let it go through.
            // Owner badge doesn't need explicit permissions.
        }
    }
}
