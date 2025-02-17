# Treasury Window

The Treasury Window is a critical component of the Filament protocol's economic model, providing a mechanism for Campaigners to acquire FILA tokens at a predictable and potentially discounted price. This facility addresses the challenges of price volatility and potential liquidity issues when obtaining FILA on the open market, ensuring that Campaigners can reliably fund their campaigns.

## Purpose

The primary purposes of the Treasury Window are:

1.  **Price Stability for Campaigners:**  Campaigners need FILA to create campaign bonds and potentially cover other fees.  However, the price of FILA on the open market can be volatile. The Treasury Window offers a stable and predictable price, allowing Campaigners to budget effectively.
2.  **Controlled FILA Supply:**  The Treasury Window allows the protocol to manage the release of FILA into circulation, helping to maintain a healthy token economy.
3.  **Acquire USDC:** The treasury can acquire USDC to finance protocol development without putting price pressure on \$FILA.
4. **Bootstrapping Campaigns:** By offering a discount, new campaign creation is incentivized.

## Key Concepts

*   **Exchange Rate:** The Treasury Window offers FILA at a defined exchange rate, typically based on a Time-Weighted Average Price (TWAP) of FILA/USDC.
*   **Discount:** The Treasury Window may offer FILA at a *discount* relative to the TWAP. This discount incentivizes Campaigners to use the Treasury Window and helps bootstrap the network. The discount level is a governance parameter.
*   **TWAP (Time-Weighted Average Price):**  A measure of the average price of FILA over a specific period (e.g., 7 days). This helps smooth out short-term price fluctuations.  An oracle (e.g., Chainlink) is typically used to provide the TWAP.
*   **`LIMIT`:**  A limit on the amount of FILA that can be purchased through the Treasury Window within a given period (e.g., per epoch or per day). This prevents excessive dilution of the FILA supply.
*   **Bonded FILA:**  The FILA purchased through the Treasury Window is *bonded* directly to the campaign. This means it is locked and can only be used for campaign-related purposes (primarily the campaign bond).

## How it Works

1.  **Access:** Campaigners interact with the Treasury Window contract (likely through the `Paymaster`).
2.  **Quote:** The Campaigner requests a quote for a specific amount of FILA.  The Treasury Window contract calculates the required USDC amount based on the current exchange rate and discount.
    *   `Exchange Rate = TWAP($FILA/USDC) * (1 - Discount)`
3.  **Payment:**  The Campaigner sends the required USDC to the Treasury Window contract.
4.  **FILA Issuance:**  The Treasury Window contract mints the requested amount of FILA *and directly bonds it to the Campaigner's specified campaign*. This is a crucial point: the FILA is *not* sent to the Campaigner's wallet as freely transferable tokens.
5.  **Limit Enforcement:**  The Treasury Window contract checks if the `LIMIT` has been reached. If so, the transaction is rejected (or potentially queued, depending on the implementation).

## Example

Let's say:

*   TWAP($FILA/USDC) = $0.10 (FILA is trading at 10 cents on average)
*   Discount = 20%
*   Campaigner wants to acquire 10,000 FILA for a campaign bond.

1.  The exchange rate is calculated: $0.10 * (1 - 0.20) = $0.08
2.  The required USDC is: 10,000 FILA * $0.08/FILA = $800
3.  The Campaigner sends 800 USDC to the Treasury Window contract.
4.  The Treasury Window contract mints 10,000 FILA and bonds it to the Campaigner's campaign.

## Governance Parameters

The following parameters are likely controlled by governance:

*   **`DISCOUNT`:** The discount percentage applied to the TWAP.
*   **`LIMIT`:** The maximum amount of FILA that can be purchased through the Treasury Window within a given period.
*   **`TWAP_PERIOD`:** The time period over which the TWAP is calculated (e.g., 7 days).
*   **Oracle Address**: The contract to source the TWAP price.

## Integration with Paymaster

The `Paymaster` contract likely plays a key role in integrating the Treasury Window:

*   **Funding Source:** When a Campaigner initiates a payment (e.g., to cover Delegate fees), the `Paymaster` can check if the Campaigner has sufficient funds in their campaign bond. If not, it can offer the option to purchase additional FILA through the Treasury Window.
*   **Atomic Operations:** The `Paymaster` can ensure that the purchase of FILA through the Treasury Window and the subsequent payment are performed atomically (in a single transaction). This prevents race conditions and ensures that the Campaigner has sufficient funds.

## Code Example (Conceptual Solidity)

```solidity
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IOracle {
    function getPrice(string memory symbol) external view returns (uint256);
}

contract TreasuryWindow {
    IERC20 public filaToken;
    IERC20 public usdcToken;
    IOracle public oracle;

    uint256 public discount; // e.g., 20 for 20%
    uint256 public limit;
    uint256 public twapPeriod; // in seconds
		address public admin;

    // ... (Other state variables)

		constructor(address _filaToken, address _usdcToken, address _oracle, uint256 _twapPeriod) {
        filaToken = IERC20(_filaToken);
        usdcToken = IERC20(_usdcToken);
        oracle = IOracle(_oracle);
        twapPeriod = _twapPeriod;
    }

    function getRequiredUsdcForFila(uint256 filaAmount) public view returns (uint256) {
        uint256 twap = getTwap(); // Implement TWAP calculation using the oracle
        uint256 exchangeRate = twap * (100 - discount) / 100;
        return filaAmount * exchangeRate / 1e18; // Assuming FILA has 18 decimals
    }

    function buyFilaWithUsdc(uint256 usdcAmount) public returns (uint256) {
        // 1. Check limit
        //require(currentPeriodSales + filaAmount <= limit, "Treasury Window limit reached");

        // 2. Calculate FILA amount based on TWAP and discount
        uint256 filaAmount = usdcAmount * 1e18 * 100 / (getTwap() * (100- discount));

        // 3. Transfer USDC from user
        usdcToken.transferFrom(msg.sender, address(this), usdcAmount);

        // 4. Mint FILA *directly to the Paymaster, bonded to the campaign* (Requires interaction with Paymaster)
        //    This is a KEY difference from a typical exchange.
        //    The Paymaster contract would have a function like `receiveBondedFila(campaignId, amount)`
        //    We assume the Paymaster address is stored in a variable called `paymaster`
        IPaymaster(paymaster).receiveBondedFila(msg.sender, filaAmount); // NOTE: Hypothetical function

        // 5. Update sales for current period
        //currentPeriodSales += amount;

        return filaAmount;
    }

		function getTwap() public view returns (uint256){
			return oracle.getPrice("FILA");
		}

    // ... (Functions for setting parameters, potentially by governance)
		function setDiscount(uint256 _discount) external {
			require(msg.sender == admin, "Only admin");
			discount = _discount;
		}
		function setLimit(uint256 _limit) external{
			require(msg.sender == admin, "Only admin");
			limit = _limit;
		}

}

interface IPaymaster {
		function receiveBondedFila(address campaigner, uint256 amount) external;
}

```

## Benefits

*   **Price Predictability:** Campaigners can budget effectively, knowing the cost of acquiring FILA.
*   **Liquidity Support:**  Provides a reliable source of FILA, even if market liquidity is low.
*   **Controlled Inflation:** The `LIMIT` parameter helps manage the supply of FILA.
*   **Bootstrapping:** The discount incentivizes early adoption of the Filament protocol.
* **Funds Protocol:** Provides a mechanism for the treasury to accumulate USDC.

## Conclusion

The Treasury Window is a vital part of the Filament economic ecosystem. It provides a stable and predictable way for Campaigners to acquire the FILA needed to run campaigns, while also helping to manage the overall FILA supply. Its integration with the `Paymaster` contract ensures a seamless and secure user experience. This mechanism addresses a key challenge in many token-based systems – the volatility and potential illiquidity of the native token – making Filament more accessible and attractive to Campaigners.
