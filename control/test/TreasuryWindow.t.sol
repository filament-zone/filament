// SPDX-License-Identifier: Apache-2.0.
pragma solidity ^0.8.16;

import {Test, console} from "forge-std/Test.sol";
import {SafeERC20} from "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {Pausable} from "@openzeppelin/contracts/utils/Pausable.sol";

// import {IUniswapV2Factory} from "../src/interfaces/IUniswapV2Factory.sol";
import {IUniswapV2Factory} from "@univ2-core/contracts/interfaces/IUniswapV2Factory.sol";
import {IUniswapV2Pair} from "@univ2-core/contracts/interfaces/IUniswapV2Pair.sol";
import {IUniswapV2Router02} from "@univ2-periphery/contracts/interfaces/IUniswapV2Router02.sol";

import {MockERC20, MockUSD20} from "./mock/MockERC20.sol";

import {FixedPoint} from "../src/libs/Uniswap.sol";

import {
    TreasuryWindow,
    Unauthorized,
    BondNotFound,
    InsufficientFila,
    InvalidManager,
    InvalidSecurityCouncil,
    InvalidUniV2Pair
} from "../src/TreasuryWindow.sol";

contract TreasuryWindowTest is Test {
    address private manager;
    address private securityCouncil;

    MockERC20 private fila;
    MockUSD20 private usd;

    IUniswapV2Factory private uniFactory;
    IUniswapV2Router02 private uniRouter;
    IUniswapV2Pair private pair;

    TreasuryWindow private window;

    function readByteCode(string memory file_) internal view returns (bytes memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, string.concat("/test/deployed/", file_));
        string memory json = vm.readFile(path);
        return vm.parseJsonBytes(json, ".evm.deployedBytecode.object");
    }

    function initUniswap() internal {
        uniFactory = IUniswapV2Factory(0x5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f);
        uniRouter = IUniswapV2Router02(0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D);

        // https://unpkg.com/@uniswap/v2-core@1.0.1/build/UniswapV2Factory.json
        bytes memory factoryCode = readByteCode("UniswapV2Factory.json");
        vm.etch(address(uniFactory), factoryCode);
        // emit log_bytes(address(uf).code);

        // https://unpkg.com/@uniswap/v2-periphery@1.1.0-beta.0/build/UniswapV2Router02.json
        bytes memory routerCode = readByteCode("UniswapV2Router02.json");
        vm.etch(address(uniRouter), routerCode);
    }

    function setUp() public {
        initUniswap();

        manager = makeAddr("manager");
        securityCouncil = makeAddr("securityCouncil");

        fila = new MockERC20("Filamock Token", "FILA");
        usd = new MockUSD20("Utopian Solar Denom", "USD");

        pair = IUniswapV2Pair(uniFactory.createPair(address(fila), address(usd)));

        fila.mint(address(pair), 1_000_000 ether);
        usd.mint(address(pair), 0.01 ether); // 10**10 usd
        pair.sync();

        (uint112 r0_, uint112 r1_,) = pair.getReserves();
        uint256 initial_ = pair.token0() == address(usd) ? uint256(r1_ / r0_) : uint256(r0_ / r1_);
        // emit log_uint(initial_);

        window = new TreasuryWindow(address(fila), address(usd), address(pair), initial_ * 2, manager, securityCouncil);

        fila.mint(address(window), 100_000_000 ether);
    }

    function test_Constructor() public {
        TreasuryWindow tw = new TreasuryWindow(address(fila), address(usd), address(pair), 0, manager, securityCouncil);
    }

    function testFuzz_GetFila_InitialFilaPerUsd(uint256 amount_) public {
        vm.assume(amount_ < type(uint112).max);
        vm.assume(amount_ * window.initialFilaPerUsd() <= fila.balanceOf(address(window)));

        assertEq(window.lastOracleUpdate(), 0);
        assertEq(fila.balanceOf(address(this)), 0);

        usd.mint(address(this), 1 ether);
        usd.approve(address(window), 1 ether);

        window.getFila(amount_, amount_, address(this));

        assertEq(fila.balanceOf(address(this)), amount_ * window.initialFilaPerUsd());
    }

    function testFuzz_GetFila_NotEnoughOut(uint256 out_) public {
        vm.assume(out_ < type(uint112).max);

        uint256 filaBefore_ = fila.balanceOf(address(window));
        uint256 usdIn_ = out_ / window.initialFilaPerUsd();
        vm.assume(usdIn_ > 0);

        usd.mint(address(this), usdIn_);
        usd.approve(address(window), usdIn_);

        // request one more than we should get out
        uint256 filaOut_ = window.getFila(usdIn_, out_ + 1, address(this));

        assertEq(filaOut_, 0);
        assertEq(fila.balanceOf(address(this)), 0);
        assertEq(fila.balanceOf(address(window)), filaBefore_);
    }

    function testFuzz_UpdateOracle_NoChangeBeforePeriod(uint256 offset_) public {
        vm.assume(offset_ < window.TWAP_PERIOD() - 1); // first block.timestamp is 1

        (,, uint32 last_) = pair.getReserves();
        assertEq(last_, block.timestamp); // sync happens in setUp()

        vm.warp(block.timestamp + offset_);
        assert(!window.updateOracle());
    }

    function testFuzz_UpdateOracle(uint256 amount_) public {
        vm.assume(amount_ < type(uint112).max);
        vm.assume(amount_ > 0);

        uint256 beforeUpdate_ = window.previewGetFila(amount_);
        vm.warp(block.timestamp + window.TWAP_PERIOD() / 2);
        fila.mint(address(pair), 1000 ether);
        pair.sync();

        vm.warp(block.timestamp + window.TWAP_PERIOD() / 2);
        assert(window.updateOracle());

        uint256 afterUpdate_ = window.previewGetFila(amount_);
        assertGt(beforeUpdate_, afterUpdate_);
    }

    function test_Constructor_InvalidUniV2Pair() public {
        MockERC20 wrongToken_ = new MockERC20("Wrong Token", "WRONG");
        address wrongPair_ = uniFactory.createPair(address(usd), address(wrongToken_));

        vm.expectRevert(InvalidUniV2Pair.selector);
        TreasuryWindow tw = new TreasuryWindow(address(fila), address(usd), wrongPair_, 0, manager, securityCouncil);
    }

    function test_RescueFunds_ExpectedPause() public {
        vm.startPrank(securityCouncil);
        vm.expectRevert(Pausable.ExpectedPause.selector);
        window.rescueFunds(address(this));
        vm.stopPrank();
    }

    function test_GetFila_EnforcedPaused() public {
        vm.prank(securityCouncil);
        window.pause();

        vm.expectRevert(Pausable.EnforcedPause.selector);
        window.getFila(100, 0, address(this));
    }

    function testFuzz_Pause_Unauthorized(address who_) public {
        vm.assume(who_ != securityCouncil);

        vm.startPrank(who_);
        vm.expectRevert(Unauthorized.selector);
        window.pause();
        vm.stopPrank();
    }

    function testFuzz_Unpause_Unauthorized(address who_) public {
        vm.assume(who_ != securityCouncil);

        vm.prank(securityCouncil);
        window.pause();

        vm.startPrank(who_);
        vm.expectRevert(Unauthorized.selector);
        window.unpause();
        vm.stopPrank();
    }
}
