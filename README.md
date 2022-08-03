# Substrate Node Template for DPOS

## run the node

```bash
./scripts/start_node.sh --alice
./scripts/start_node.sh --bob
./scripts/start_node.sh --charlie
```

the script runs the node with _--log runtime::dpos_ so it is possible to see the dpos pallet traces

## generate a new network configuration 
```bash
./scripts/generate_spec.sh
```

 ## usage
 At the beginning there's no validator bonded in the DPOS staking pallet: the BP are the one created in genesis.
 
 * **bond**(ALICE_STASH): ALICE_STASH will start producing blocks alone

* **bond**(BOB_STASH): ALICE_STASH and BOB_STASH will produce blocks

* **set_maximum_validators**(1): the validator with more stake will produce blocks

* **vote**(ALICE_STASH or BOB_STASH): to change the winner

other extrinsics for **set_minimum_validators, unbond, unvote** are provided.

Have fun!

