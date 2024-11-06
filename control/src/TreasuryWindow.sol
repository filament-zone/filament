// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import {IERC20} from "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

import {IUniswapV2Pair} from "@univ2-core/contracts/interfaces/IUniswapV2Pair.sol";

import {UniswapV2OracleLibrary, FixedPoint} from "./libs/Uniswap.sol";
import {UniswapV2Library} from "./libs/UniswapV2Library.sol";

import {BaseSecurity, Unauthorized, InvalidManager, InvalidSecurityCouncil} from "./BaseSecurity.sol";

error BondNotFound(uint256 campaign);
error InvalidUniV2Pair();
error InsufficientFila(uint256 want, uint256 have);

contract TreasuryWindow is BaseSecurity {
    using SafeERC20 for IERC20;
    using FixedPoint for *;

    IERC20 public immutable fila;
    IERC20 public immutable usd;

    IUniswapV2Pair public immutable tradingPair; // XXX: immutable?

    uint256 public constant TWAP_PERIOD = 7 days;

    /// @notice The initialFilaPerUsd rate is only relevant in case there is no liquity
    ///         for the reference pair
    uint256 public initialFilaPerUsd;

    uint256 public lastFilaPerUsdCum;

    FixedPoint.uq112x112 public filaPerUsdTwap;

    uint256 public lastOracleUpdate;

    event FilaPerUsdTwapUpdated(uint256 oldPrice, uint256 newPrice);

    event FundsRescued(address indexed where, uint256 amountFila, uint256 amountUsd);

    constructor(
        address fila_,
        address usd_,
        address tradingPair_,
        uint256 initialFilaPerUsd_,
        address manager_,
        address securityCouncil_
    ) BaseSecurity(manager_, securityCouncil_) {
        fila = IERC20(fila_);
        usd = IERC20(usd_);
        tradingPair = IUniswapV2Pair(tradingPair_);

        if (UniswapV2Library.pairFor(tradingPair.factory(), fila_, usd_) != tradingPair_) revert InvalidUniV2Pair();

        initialFilaPerUsd = initialFilaPerUsd_;
    }

    /// @notice Allow the caller to convert `usdIn_` amount of tokens to FILA based
    ///         on the 7 day TWAP of `tradingPair`.
    /// @dev Allow the caller to convert `usdIn_` amount of tokens to FILA based
    ///      on the 7 day TWAP of `tradingPair`. It returns the amount of FILA that
    ///      were transfered.
    function getFila(uint256 usdIn_, uint256 minFilaOut_, address target_) external whenNotPaused returns (uint256) {
        this.updateOracle();

        uint256 filaOut_ = this.consultOracle(usdIn_);
        if (filaPerUsdTwap.decode() == 0) filaOut_ = initialFilaPerUsd * usdIn_;
        if (filaOut_ < minFilaOut_) return 0;

        uint256 myFila_ = fila.balanceOf(address(this));
        if (myFila_ < filaOut_) revert InsufficientFila(filaOut_, myFila_);

        usd.safeTransferFrom(msg.sender, address(this), usdIn_);
        fila.safeTransfer(target_, filaOut_);

        return filaOut_;
    }

    function updateOracle() external returns (bool) {
        (uint256 cum0_, uint256 cum1_,) = UniswapV2OracleLibrary.currentCumulativePrices(address(tradingPair));

        if ((block.timestamp - lastOracleUpdate) < TWAP_PERIOD) return false;

        uint256 currentFilaPerUsdCum_ = tradingPair.token0() == address(usd) ? cum0_ : cum1_;
        FixedPoint.uq112x112 memory old_ = filaPerUsdTwap;
        filaPerUsdTwap = computeTwap(lastFilaPerUsdCum, lastOracleUpdate, currentFilaPerUsdCum_, block.timestamp);
        lastOracleUpdate = block.timestamp;
        lastFilaPerUsdCum = currentFilaPerUsdCum_;

        emit FilaPerUsdTwapUpdated(uint256(old_.decode()), uint256(filaPerUsdTwap.decode()));

        return true;
    }

    ///// View

    function previewGetFila(uint256 usdIn_) external view returns (uint256) {
        uint256 filaOut_ = this.consultOracle(usdIn_);
        if (filaPerUsdTwap.decode() == 0) filaOut_ = initialFilaPerUsd * usdIn_;

        return filaOut_;
    }

    function consultOracle(uint256 usdAmount_) external view returns (uint256) {
        return filaPerUsdTwap.mul(usdAmount_).decode144();
    }

    ///// Internal

    function computeTwap(
        uint256 lastFilaPerUsdCum_,
        uint256 lastOracleUpdate_,
        uint256 currentFilaPerUsdCum_,
        uint256 currentTime_
    ) internal pure returns (FixedPoint.uq112x112 memory) {
        FixedPoint.uq112x112 memory cumAvg_ = FixedPoint.uq112x112(
            uint224((currentFilaPerUsdCum_ - lastFilaPerUsdCum_) / (currentTime_ - lastOracleUpdate_))
        );
        return cumAvg_;
    }

    ///// Permissioned

    function rescueFunds(address target_) external whenPaused {
        if (msg.sender != manager) revert Unauthorized();

        uint256 amountFila = fila.balanceOf(address(this));
        fila.safeTransfer(target_, amountFila);
        uint256 amountUsd = usd.balanceOf(address(this));
        usd.safeTransfer(target_, amountUsd);

        emit FundsRescued(target_, amountFila, amountUsd);
    }
}
