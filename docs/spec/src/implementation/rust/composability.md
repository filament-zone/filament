# Composability

Pulzaard provides an implementation of an ABCI application. To enable
extensibility, maintainability and logical isolation of different parts of the
state machine within that application, three parts are important to look at:
[App](#app), [Component](#component) and [Handler](#handler). To help illustrate
the flow of ABCI calls through the application consult the following diagram:

![ABCI flow](/assets/composability.svg)

## App

The App is structured as a stack of [components](#component). Breaking it apart
in that way will allow for well-scoped, self-contained building blocks, while
allowing one of those building blocks to offer general/specialised
smart-contract execution, if it should become a requirement in the future.

*Component ordering matters and needs to be considered when the `App` is
constructed.*

``` rust, ignore
{{#include ../../../../../app/src/app.rs:8:13}}
```

## Component

Components are responsible for chain-level ([`InitChain`][abci-initchain]) &
block-level ([`BeginBlock`][abci-beginblock], [`EndBlock`][abci-endblock]) calls
of the ABCI. Core network features will be encapsulated here, e.g. banking,
governance, staking, etc.

``` rust, ignore
{{#include ../../../../../app/src/component.rs:8:48}}
```

## Handler

Handlers control the execution flow of transactions and the actions they carry.
This necessitates that the trait is implemented both on the `Transaction` as
well as the `Action` type.

``` rust, ignore
{{#include ../../../../../app/src/handler.rs:10:15}}
```

[abci-initchain]: https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#initchain
[abci-beginblock]: https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#beginblock
[abci-endblock]: https://github.com/tendermint/tendermint/blob/main/spec/abci/abci.md#beginblock
