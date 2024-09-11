# Filament Ethereum Control

## Token grants

The implementation of token grants is a modified version of the
[starknet token](https://github.com/starknet-io/StarkNet-Token).

## Developement

Install foundry:

```Bash
curl -L https://foundry.paradigm.xyz | bash
```

To install all the required depedencies run `forge install` from with the `control`
directory.

## Deploy

All commands listed are assumed to be run from within the directory that this file
is located in.

### Local

Deploying contracts locally makes use of `anvil`.

After starting anvil it will print some accounts and private keys which can be
used to author transactions. With the default configuration the server will
listen on `localhost:8545`.

#### Token

The following will deploy the token contract and mint tokens to the account
specified via the `TOKEN_RECEIVER` variable.

```Bash
PRIVATE_KEY=0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80 TOKEN_RECEIVER=0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266 TOKEN_MINT_AMOUNT=10000000000000000000000 forge script script/FilamentToken.s.sol:DeployFilaTokenScript --rpc-url http://localhost:8545 --broadcast
```

The output of the above command will include the contract address. Alternatively
if `jq` is installed the contract address can be extracted via

```Bash
jq -r '.transactions[] | select(.contractName | contains("FilamentToken")) | select(.transactionType | contains("CREATE")) | .contractAddress' <broadcast/FilamentToken.s.sol/31337/run-latest.json
```

assuming no other broadcast has happend since then.
