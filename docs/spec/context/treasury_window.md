# Treasury Window

Campaigners need bonded \$FILA to conduct campaigns. Among other functions, bonded FILA provides Sybil resistance where campaigners incur some cost and thus can’t burden the delegate set with spam campaigns. The problem is getting \$FILA to conduct the campaign. In cases of low liquidity, buying a reasonable amount of \$FILA could incur slippage. In general campaigners should not be exposed to price volatility when purchasing services of the Filament Hub. The treasury window is facility that solves this problem by selling bonded FILA at a reliable price in exchange for USDC.

### How Treasury Window Works

The treasury window is a smart contract let’s anyone buy bonded FILA at a discount. Campaigners can send USDC and receive \$FILA controlled exchange rate `EXCHANGE_RATE`. `EXCHANGE_RATE` should be:

```rust,ignore
let EXCHANGE_RATE = TWAP($FILA, "7days") * DISCOUNT;
```

A campaign should cost ~ 1000 USDC and so a 50% discount would provide \$2,000 worth of FILA, a 90% discount would provide \$10,000 of \$FILA, etc. Initially `DISCOUNT` should be set to high and reduce over time.

The treasury window contract should have `LIMIT` on how many bonds it issues at a given time. `LIMIT` should be controlled by the admin keys.

### Benefits

The treasury window is a facility in which the treasury can sell

**Reliable campaign pricing:**  Campaigns should have a reasonable price in USDC terms. Campaigners looking to acquire the services of the Filament Hub may not necessarily be interested in taking price exposure (even if delegates/delegators) are.  The treasury can sell FILA at fixed cost even if \$FILA liquidity is low and acquiring FILA would  incur significant slippage.

**Aquire USDC:**  The treasury can acquire USDC to finance protocol development without putting price pressure on \$FILA.

### Understanding `DISCOUT`

`DISCOUNT` functions as a money multiplayer which increases the amount of circulating \$FILA.  Since \$FILA is fixed supply, we can think about the treasury window facility similar to inflation in other networks.  The treasury can thus use the `DISCOUNT` parameter to control the supply of \$FILA.
