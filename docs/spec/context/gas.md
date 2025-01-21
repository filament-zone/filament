# Gas Token \$FILUM

### Sybil Resistance

In order to prevent spam on permission-less systems, blockchains employ a gas mechanism to impose economic cost on users who submit transactions. Gas cost provide permission-less systems with sybil resistance. The Filament Hub is a permission less system which requires a sybil resistance. The problem is the design of the Filament Hub has no tokens within it's computational domain, instead tokens are stored on outposts. One solution would be to bridge those tokens to the Filament Hub but that hurts UX by fragmenting balances across state machines. Instead Filament Hub should have a max number of transactions per day and represent each interaction with a local gas token \$FILUM

### \$FILUM

\$FILUM is the gas token of the Filament Hub. \$FILUM is non transferable and not controllable by users. Instead \$FILUM is assigned to users periodically by the Filament Hub. Delegates and Campaigners have bonded on the Outpost. Every epoch,  `FILUM_EMISSIONS`  of FILUM is allocated to each user who has `MIN_BOND` in their account. The maximum number of \$FILUM in a given account never exceeds `MAX_FILUM_BALANCE`. `\$FILUM_EMISSIONS` , `MIN_BOND` and `MAX_FILUM_BALANCE` are governance parameters.

## \$FILUM in Practice

\$FILUM will ensure that users of the system have ~20 or so max transactions per day. The user should see how many transactions they have left and be informed when they are running out.  The number should be adjusted so it's effectively never an issue for any user but not so high as it would spam the sequencer or DA layers.
