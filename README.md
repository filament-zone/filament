# Filament

## Development

Install [rustup](https://rustup.rs/) first, afterwards run:

```sh
make install-dev-tools
```

Run the hub locally by invoking:

```sh
make run-local-hub
```

Output the Core json schema with:

```sh
make generate-core-schema
```

### Test accounts

Currently it's required that any acocunt that wants to interact with the Hub
has to have some gas tokens. As we assume interactions through ETH signing
agents like Metamask the following steps will prepare the Hub at genesis to
have the appropriate balances.

Get the address and credential id derived from an ETH address:

``` sh
$ env SKIP_GUEST_BUILD=1 cargo run -p filament-hub-cli --bin eth-to-hub -- 0x01ed3152fC4C092faA6C16Fa3AFc9B8D0BDC2491
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.34s
     Running `target/debug/eth-to-hub 0x01ed3152fC4C092faA6C16Fa3AFc9B8D0BDC2491`
eth: 0x01ed3152fC4C092faA6C16Fa3AFc9B8D0BDC2491
hub: sov1q8knz5hufsyjl2nvzmar4lym359acfy3qqqqqqqqqqqqqqqqqqqqd26pxp
credential id: 0x9769e91a5cbde876ab1b04da61bffeab7cb048d93727e6c5d60654582bcd8a90
```

Add the a new account to `test-data/genesis/mock/accounts.json`:

``` diff
     {
       "credential_id": "0x186ff0616d6ce0f2a387b7afd94f1ef0b1b0297b4bb7cc0d7a7951f88f066a43",
       "address": "sov1x2elck0qdwxn03exmr2d9h4355h62uckqqqqqqqqqqqqqqqqqqqqyqvyuk"
+    },
+    {
+      "credential_id": "0x9769e91a5cbde876ab1b04da61bffeab7cb048d93727e6c5d60654582bcd8a90",
+      "address": "sov1q8knz5hufsyjl2nvzmar4lym359acfy3qqqqqqqqqqqqqqqqqqqqd26pxp"
     }
   ]
 }
```

Mint a decent amount of gas tokens for the newly added account in
`test-data/genesis/mock/bank.json`:

``` diff
       [
         "sov1x2elck0qdwxn03exmr2d9h4355h62uckqqqqqqqqqqqqqqqqqqqqyqvyuk",
         1000000000
+      ],
+      [
+        "sov1q8knz5hufsyjl2nvzmar4lym359acfy3qqqqqqqqqqqqqqqqqqqqd26pxp",
+        1000000000
       ]
     ],
     "authorized_minters": [
```
