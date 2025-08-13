# NodeWarden
NodeWarden is a Scrypto blueprint that intends to wrap validator owner badges, such that the node owner can create new badges with only specific privileges enabled: This allows node runners to do certain operations while conforming to the principle of least privilege.

# Warning: This blueprint is provided on an as-is basis. Use at your own risk.

# Terminology
**Validator Owner Badge**: The NFT that is used to control a validator component

**NodeWarden Owner Badge**: The owner badge of the NodeWarden component - gives access to administrative methods and allows for depositing or withdrawing a validator owner badge.

**Access Key Badge**: a badge issued by the person holding the "NodeWarden Owner Badge" to be given to the delegates

**Delegate**: The delegate is the user that the owner of the "NodeWarden Owner Badge" NFT desires to **delegate** some permissions over the validator component to.

# Delegation Methodology Explained
By simply creating a component of this access manager blueprint, a "NodeWarden Owner Badge" is given in return, which in turn allows the instantiator to issue and send "Access Key Badges" to selected people, and recall those "Access Key Badges" when/if necessary.
After a "Validator Owner Badge" NFT has been deposited into the component by its owner, the holder of an "Access Key Badge" will have access to the methods that the owner has enabled on that "Access Key Badge". The methods available to the holder correspond to the methods that exist on the Radix validator component.

# Usage
See the `manifests` directory for some example Radix transaction manifests.

The usage flow can be described in this way:

1. Create a new NodeWarden component using the `new` function
2. Deposit a validator owner badge into the component using the `deposit_validator_owner_badge` method
3. Create access key badges for delegates, and deposit them in their account(s)
    - At this point, delegates can control the validator component within the permissions granted by their access key badges.
4. (optional) The owner of the NodeWarden component can update the permissions of the deployed access key badges at any time, and in-place.
5. (optional) The owner can revoke and/or destroy access key badges from delegates using the `recall_access_key_badge` and `burn_access_key_badge` methods.
6. When the owner of the NodeWarden component wants to retire the component, they can simply withdraw the validator owner badge using the `withdraw_validator_owner_badge` method. After withdrawing the badge, the component will be essentially disabled and it can no longer control the validator component.

Note: I haven't provided RTM examples for all the possible methods on the validator component, but if you know how to call one of them (like `update_key`, which does have an RTM example), you can easily call the others. The only difference with calling the validator component directly is that you have to produce a proof of either a NodeWarden owner badge or a NodeWarden access key badge and pass it along with the other parameters.

# Configurable permissions:

```rust
pub struct AccessKeyPermissions {
    /// These correspond to the validator component's interface

    // Allows the holder to stake, even when the
    // validator component is configured to
    // NOT accept delegated stake
    pub stake_as_owner: bool,
    // Allows the holder to register
    // as a validator for consensus
    pub register: bool,
    // Allows the holder to unregister
    // from being a validator in consensus
    //
    // It's generally considered to be
    // good manners to unregister in the case
    // a validator is expected to experience
    // issues for a longer period of time, and this
    // permission facilitates this.
    pub unregister: bool,
    // Allows the holder to update the
    // Secp256k1 public key that is used
    // by the Radix node that is hosting the validator.
    //
    // This is useful when rotating to backup nodes in case
    // of an outage on a main server. By updating the key to
    // the node key of a backup node, you can switch servers
    // without interruption.
    pub update_key: bool,
    // Allows the holder to update the fee percentage
    // that is charged by the validator.
    pub update_fee: bool,
    // Allows the holder to lock stake units inside of the validator's vault,
    // which can be used to show commitment.
    pub lock_owner_stake_units: bool,
    // Allows the holder to start the process of unlocking stake units
    // from the validator's vault. This does not give permissions to eventually
    // withdraw the stake units though, which is handled by `finish_unlock_owner_stake_units`.
    pub start_unlock_owner_stake_units: bool,
    // Allows the holder to finish the process of unlocking stake units,
    // essentially withdrawing the stake units from the vault and being able to
    // deposit them anywhere they like.
    pub finish_unlock_owner_stake_units: bool,
    // Allows the holder to update the configuration for accepting delegated stake.
    pub update_accept_delegated_stake: bool,
    // Allows the holder to signal readiness for protocol updates.
    // Read more about protocol updates: https://docs.radixdlt.com/docs/node-protocol-updates
    pub signal_protocol_update_readiness: bool,

    // Some permissions for updating metadata on the validator component

    // Allows the holder to set pieces of metadata on the validator component
    pub set_metadata: bool,
    // Allows the holder to remove pieces of metadata from the validator component
    pub remove_metadata: bool,
    // Allows the holder to lock pieces of metadata on the validator component,
    // after which they cannot be changed or removed.
    pub lock_metadata: bool,

    // An extra permission that is quite powerful, and can do all of the above
    // By creating a proof of the validator owner badge, the holder can
    // gain access to all of the permissions associated with being a validator owner,
    // without actually owning the badge itself.
    pub create_validator_owner_badge_proof: bool,
}
```