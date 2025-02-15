# $FILUM (Gas Token)

FILUM is a non-transferable gas token used to pay for computation on the Filament Hub. It plays a crucial role in preventing spam and ensuring fair access to the Hub's resources. Unlike FILA, the network's native token, FILUM *cannot* be bought, sold, or transferred between users. It is solely used internally to measure and limit resource consumption.

## Key Features

*   **Non-Transferable:** FILUM is *not* a typical cryptocurrency. It cannot be traded on exchanges or sent between wallets. This is a critical design choice that prevents the creation of a secondary market for FILUM and ensures its sole purpose is to pay for computation.
*   **Periodic Allocation:** Users who have a minimum amount of FILA *bonded* in their account receive a periodic allocation of FILUM. This allocation replenishes automatically, allowing active users to interact with the Hub without needing to acquire FILUM externally. The allocation are not cumulative.
*   **Usage-Based Consumption:** Every interaction with the Filament Hub that requires computation (e.g., creating a campaign, submitting a proposal, voting, posting a segment) consumes a certain amount of FILUM. The amount of FILUM consumed depends on the complexity of the operation.
*   **Sybil Resistance:**  Because acquiring FILUM requires bonding FILA, and FILUM itself is non-transferable, it becomes economically infeasible for malicious actors to spam the network with a large number of requests. This protects the Hub's resources and ensures fair access for legitimate users.
*   **Prevents Denial-of-Service:**  FILUM limits the rate at which users can interact with the Hub, preventing denial-of-service (DoS) attacks that could overwhelm the system.

## How FILUM Works

1.  **Bonding FILA:** To be eligible for FILUM, users must have a minimum amount of FILA bonded in their account.  This bond demonstrates a commitment to the network and helps prevent Sybil attacks.  The specific minimum bond amount (`MIN_BOND`) is a governance parameter.
2.  **Periodic Allocation:**  Every *epoch* (a defined period, e.g., 24 hours), eligible users receive a fixed amount of FILUM. This amount (`FILUM_EMISSIONS`) is also a governance parameter. The amount is set to provide a reasonable number of interactions.
3.  **Maximum Balance:**  There is a maximum limit on the amount of FILUM an account can hold (`MAX_FILUM_BALANCE`). This prevents users from hoarding FILUM and ensures a fair distribution.
4.  **Consumption:** When a user interacts with the Hub, the required FILUM is deducted from their balance. If a user does not have enough FILUM, the transaction will fail.
5.  **Replenishment:**  At the start of each epoch, eligible users' FILUM balances are replenished up to the `MAX_FILUM_BALANCE`.  Any unused FILUM from the previous epoch is effectively "lost" (it does *not* accumulate).

## Governance Parameters

The following parameters, which are subject to change through governance, control the FILUM mechanism:

*   **`MIN_BOND`:** The minimum amount of FILA that must be bonded to be eligible for FILUM allocations.
*   **`FILUM_EMISSIONS`:** The amount of FILUM allocated to eligible users each epoch.
*   **`MAX_FILUM_BALANCE`:** The maximum amount of FILUM an account can hold.
*   **`EPOCH_LENGTH`:**  The duration of an epoch (e.g., 24 hours).

## Example

Let's say:

*   `MIN_BOND` = 100 FILA
*   `FILUM_EMISSIONS` = 50 FILUM
*   `MAX_FILUM_BALANCE` = 50 FILUM
*   `EPOCH_LENGTH` = 24 hours

A user who has 150 FILA bonded would receive 50 FILUM at the start of each epoch. If they use 20 FILUM during the day, their balance will be 30 FILUM. At the start of the next epoch, their balance will be replenished back to 50 FILUM (not 80 FILUM). A user who has only 50 FILA bonded would *not* receive any FILUM.

## Why a Separate Gas Token?

Using a separate, non-transferable gas token like FILUM offers several advantages over using FILA directly for gas:

*   **Price Stability:** The cost of interacting with the Hub is not directly tied to the price of FILA, which can be volatile. This makes it easier for users to predict and budget for their interactions.
*   **Prevents Hoarding:**  Because FILUM is non-transferable and replenishes periodically, there is no incentive to hoard it. This ensures a fair distribution and prevents a small number of users from monopolizing access to the Hub.
*   **Simplified User Experience:** Users don't need to worry about acquiring FILUM on the open market. As long as they have a sufficient FILA bond, they will automatically receive enough FILUM to interact with the Hub.

## Conclusion

FILUM is a vital component of the Filament protocol, providing a robust and user-friendly mechanism for managing computational resources and preventing abuse. Its non-transferable nature and periodic allocation ensure fair access and prevent the creation of a secondary market, keeping its focus solely on its intended purpose: paying for computation on the Filament Hub.
