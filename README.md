# NodeWarden
NodeWarden is a Scrypto blueprint that intends to wrap validator owner badges, such that the node owner can create new badges with only specific privileges enabled: This allows node runners to do certain operations while conforming to the principle of least privilege.

# Warning: this is a work in progress. Use at your own risk

# Terminology
**Validator Owner Badge**: The NFT that is used to control a validator component

**NodeWarden Owner Badge**: The owner badge of the NodeWarden component - gives access to administrative methods and allows for depositing or withdrawing a validator owner badge.

**Access Key Badge**: a badge issued by the person holding the "NodeWarden Owner Badge" to be given to the delegates

**Delegate**: The delegate is the user that the owner of the "NodeWarden Owner Badge" NFT desires to **delegate** some permissions over the validator component to.

# Delegation Methodology Explained
By simply creating a component of this access manager blueprint, a "NodeWarden Owner Badge" is given in return, which in turn allows the instantiator to issue and send "Access Key Badges" to selected people, and recall those "Access Key Badges" when/if necessary.
After a "Validator Owner Badge" NFT has been deposited into the component by its owner, the holder of an "Access Key Badge" will have access to the methods that the owner has enabled on that "Access Key Badge". The methods available to the holder correspond to the methods that exist on the Radix validator component.

# Usage

todo
<!-- ## Create Access Manager Component
To create an access manager component, use the following transaction manifest syntax
```
CALL_FUNCTION
    Address("${package}")
    "AccessManager"
    "new"
    Address("${auth_badge}")
    Address("${dApp_account_address}");

CALL_METHOD Address("${account}") "deposit_batch" Expression("ENTIRE_WORKTOP");
```
After creating the access manager component, an "Access Manager Owner Badge" is minted and returned to the caller, we will use this badge to perform privileged actions later
**Deployed packages addresses:**
**Stokenet v1.0.0:** package_tdx_2_1p54xl6f3d7leetxpp85j0ua3ll2qfx4xxjcrdvsdgchr00t8qspmnq
**Mainnet v1.0.0:** package_rdx1p4m04kkm8tw3fefwrf7zvgxjw8k0n9t30vawgq2kl90q3r77nf59w8
You should use your own dApp account address, but if you don't have one, you can always use **RadixPlanet dApp account address:**
**Stokenet:** account_tdx_2_128ly7s6494uasmggf9rxy6th2e6zu53hj7p0uxgq2ucdmzf43gqkus
**Mainnet:** account_rdx12xjdx9ntkjl60r7fuv9az8uzmad0d05mqmjstrpkpvtcew87crahw6
## Create Access Manager Component with address reservation
Sometimes you need to create the component with address reservation on the transaction manifest level, to do so, use the following transaction manifest syntax
```
ALLOCATE_GLOBAL_ADDRESS
    Address("${package}")
    "AccessManager"
    AddressReservation("address_reservation")
    NamedAddress("component_address");

CALL_FUNCTION
    Address("${package}")
    "AccessManager"
    "new_with_address_reservation"
    Address("${auth_badge}")
    Address("${dApp_account_address}")
    AddressReservation("address_reservation");

CALL_METHOD Address("${account}") "deposit_batch" Expression("ENTIRE_WORKTOP");
```
## Depositing The Auth Badge
After creating the access manager component, you need to deposit the auth badge into it for the component to be able to create proof of that Auth Badge, to do so, use the following transaction manifest syntax
```
CALL_METHOD Address("${account}") "withdraw_non_fungibles" Address("${auth_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${auth_badge_id}"));
TAKE_NON_FUNGIBLES_FROM_WORKTOP Address("${auth_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${auth_badge_id}")) Bucket("auth_badge_bucket");

CALL_METHOD Address("${account}") "create_proof_of_non_fungibles" Address("${access_manager_owner_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${access_manager_owner_badge_id}"));

CALL_METHOD
    Address("${component}")
    "deposit_auth_badge"
    Bucket("auth_badge_bucket");
```
## Creating (Minting) an access key badge
The access manager owner can create an "Access Key Badge" and give it to the delegate person, using the following transaction manifest syntax
```
CALL_METHOD Address("${account}") "create_proof_of_non_fungibles" Address("${access_manager_owner_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${access_manager_owner_badge_id}"));

CALL_METHOD
    Address("${component}")
    "create_access_key_badge";

CALL_METHOD Address("${delegate_account}") "try_deposit_batch_or_abort" Expression("ENTIRE_WORKTOP") None;
```
**Note**: You can use the direct manifest mint instructions directly without calling the component as the "mint" permission is given to both the "Access Manager Owner Badge" and the component itself, the component "create_access_key_badge" method is provided for completion
**Note**: the created key only be moved between accounts after it is given to the delegate by the owner of the NFT, by creating a proof of the "Access Manager Owner Badge" in the transaction manifest, after that, if the "Access Key Badge" exists in his own account, he can withdraw it normally, if not, he can recall the "Access Key Badge" from the vault it is in, and then deposit it normally to anyne else (given that he passes other deposit restrictions the receiver has in place)
## Recall and Burn an Access Key Badge
to recall a previously issued Access Key Badge, use the following transaction manifest syntax
```
CALL_METHOD Address("${account}") "create_proof_of_non_fungibles" Address("${access_manager_owner_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${access_manager_owner_badge_id}"));

CALL_METHOD
    Address("${component}")
    "recall_key_badge"
    Address("${access_key_badge_vault_address}");

TAKE_NON_FUNGIBLES_FROM_WORKTOP Address("${access_key_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${access_key_badge_id}")) Bucket("access_key_badge_bucket");

CALL_METHOD
    Address("${component}")
    "burn_key_badge"
    Bucket("access_key_badge_bucket");
```
**Note**: You can use the direct manifest recall instructions directly without calling the component as the "recall" permission is given to both the "Access Manager Owner Badge" and the component itself, the component "recall_key_badge" method is provided for completion
**Note**: You can use the direct manifest burn instructions directly without calling the component as the "burn" permission is set to "allow_all" so that anyone can burn the access key badge in his custody, the component "recall_key_badge" method is provided for completion
**Note**: By allowing any access key badge holder to burn the key in his custody this simply means that the delegate can give up the delegated authority/permission whenever he desires, but in order for him to "re-gain" the permission, the access manager owner needs to mint a new access key badge and give it to him
## Create Auth Badge Proof
This method allows both the "Access Manager Owner Badge" holder and the "Access Key Badge" holder to create a proof of the "Auh Badge" to be used in privileged actions in the same transaction manifest
To create a proof of the "Auth Badge" held inside the "Access Manager" component, use the following syntax
```
CALL_METHOD Address("${account}") "create_proof_of_non_fungibles" Address("${access_key_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${access_key_badge_id}"));

CALL_METHOD
    Address("${component}")
    "create_auth_badge_proof";
```
**Note**: it's assumed in the above manifest that the holder of the "Access Key Badge" is the one requesting the "Auth Badge" proof, but the permission is given to both "Access Manager Owner Badge" and the "Access Key Badge", so the access manager owner can also create a proof from the "Auth Badge" without the need to create a separate "Access Key Badge"
## Withdraw Auth Badge
At any time, the owner of the access manager component can withdraw the "Auth Badge" from the component, after this action, the access manager component will no longer be able to create a proof for the "Auth Badge"
To withdraw the "Auth Badge" from the access manager component, use the following syntax
```
CALL_METHOD Address("${account}") "create_proof_of_non_fungibles" Address("${access_manager_owner_badge}") Array<NonFungibleLocalId>(NonFungibleLocalId("${access_manager_owner_badge_id}"));

CALL_METHOD
    Address("${component}")
    "withdraw_auth_badge";

CALL_METHOD Address("${account}") "deposit_batch" Expression("ENTIRE_WORKTOP");
``` -->