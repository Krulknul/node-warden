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

Note: I haven't provided RTM examples for all the possible methods on the validator component, but if you can call one, you can easily call the others. The only difference with calling the validator component directly is that you have to produce the proof and pass it along with the other parameters.