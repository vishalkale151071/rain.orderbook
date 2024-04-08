// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {Vm} from "forge-std/Vm.sol";
import {OrderBookExternalRealTest} from "test/util/abstract/OrderBookExternalRealTest.sol";
import {Reenteroor} from "test/util/concrete/Reenteroor.sol";
import {
    IOrderBookV4,
    OrderConfigV3,
    OrderV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3,
    ClearConfig
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {IParserV1} from "rain.interpreter.interface/interface/IParserV1.sol";
import {IERC3156FlashBorrower} from "rain.orderbook.interface/interface/ierc3156/IERC3156FlashBorrower.sol";
import {IERC20} from "openzeppelin-contracts/contracts/token/ERC20/IERC20.sol";
import {LibTestAddOrder} from "test/util/lib/LibTestAddOrder.sol";
import {SignedContextV1} from "rain.interpreter.interface/interface/IInterpreterCallerV2.sol";

/// @title OrderBookV3FlashLenderReentrant
/// Test that flash borrowers can reenter the orderbook, which is necessary for
/// trading etc. against it while the loan is active.
contract OrderBookV3FlashLenderReentrant is OrderBookExternalRealTest {
    function checkFlashLoanNotRevert(Reenteroor borrower, bytes memory encodedCall, uint256 loanAmount) internal {
        borrower.reenterWith(encodedCall);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(borrower), loanAmount),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.approve.selector, address(iOrderbook), loanAmount),
            abi.encode(true)
        );
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, borrower, address(iOrderbook), loanAmount),
            abi.encode(true)
        );

        // Create a flash loan.
        iOrderbook.flashLoan(IERC3156FlashBorrower(address(borrower)), address(iToken0), loanAmount, "");
    }

    /// Can reenter and read vault balances from within a flash loan.
    function testReenterReadVaultBalances(uint256 vaultId, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV4.vaultBalance.selector, address(borrower), address(iToken0), vaultId),
            loanAmount
        );
    }

    /// Can reenter and check if an order exists from within a flash loan.
    function testReenterCheckOrderExists(bytes32 orderHash, uint256 loanAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV4.orderExists.selector, orderHash), loanAmount
        );
    }

    /// Can reenter and deposit from within a flash loan.
    function testReenterDeposit(uint256 vaultId, uint256 loanAmount, uint256 depositAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        depositAmount = bound(depositAmount, 1, type(uint256).max);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transferFrom.selector, borrower, address(iOrderbook), depositAmount),
            abi.encode(true)
        );
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV4.deposit.selector, address(iToken0), vaultId, depositAmount),
            loanAmount
        );
    }

    /// Can reenter and withdraw from within a flash loan.
    function testReenterWithdraw(uint256 vaultId, uint256 loanAmount, uint256 withdrawAmount) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        withdrawAmount = bound(withdrawAmount, 1, type(uint256).max);
        vm.mockCall(
            address(iToken0),
            abi.encodeWithSelector(IERC20.transfer.selector, address(borrower), withdrawAmount),
            abi.encode(true)
        );
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(IOrderBookV4.withdraw.selector, address(iToken0), vaultId, withdrawAmount),
            loanAmount
        );
    }

    /// Can reenter and add an order from within a flash loan.
    function testReenterAddOrder(uint256 loanAmount, OrderConfigV3 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iParser)).parse("_ _:max-int-value() 1e18;:;");
        config.evaluableConfig.bytecode = bytecode;
        config.evaluableConfig.constants = constants;
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(borrower, abi.encodeWithSelector(IOrderBookV4.addOrder.selector, config), loanAmount);
    }

    /// Can reenter and remove an order from within a flash loan.
    function testReenterRemoveOrder(uint256 loanAmount, OrderV3 memory order) external {
        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        order.owner = address(borrower);
        checkFlashLoanNotRevert(borrower, abi.encodeWithSelector(IOrderBookV4.removeOrder.selector, order), loanAmount);
    }

    /// Can reenter and take orders.
    function testReenterTakeOrder(uint256 loanAmount, OrderConfigV3 memory config) external {
        LibTestAddOrder.conformConfig(config, iInterpreter, iStore);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iParser)).parse("_ _:max-int-value() 1e18;:;");
        config.evaluableConfig.bytecode = bytecode;
        config.evaluableConfig.constants = constants;

        vm.recordLogs();
        iOrderbook.addOrder(config);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, OrderV3 memory order,) = abi.decode(entries[2].data, (address, address, OrderV3, bytes32));

        TakeOrderConfigV3[] memory orders = new TakeOrderConfigV3[](1);
        orders[0] = TakeOrderConfigV3(order, 0, 0, new SignedContextV1[](0));
        TakeOrdersConfigV3 memory takeOrdersConfig =
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, "");

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower, abi.encodeWithSelector(IOrderBookV4.takeOrders.selector, takeOrdersConfig), loanAmount
        );
    }

    /// Can reenter and clear orders.
    function testReenterClear(
        uint256 loanAmount,
        address alice,
        address bob,
        OrderConfigV3 memory aliceConfig,
        OrderConfigV3 memory bobConfig
    ) external {
        vm.assume(alice != bob);

        LibTestAddOrder.conformConfig(aliceConfig, iInterpreter, iStore);
        (bytes memory bytecode, uint256[] memory constants) =
            IParserV1(address(iParser)).parse("_ _:max-int-value() 1e18;:;");
        aliceConfig.evaluableConfig.bytecode = bytecode;
        aliceConfig.evaluableConfig.constants = constants;

        LibTestAddOrder.conformConfig(bobConfig, iInterpreter, iStore);
        (bytecode, constants) = IParserV1(address(iParser)).parse("_ _:max-int-value() 1e18;:;");
        bobConfig.evaluableConfig.bytecode = bytecode;
        bobConfig.evaluableConfig.constants = constants;

        bobConfig.validInputs[0] = aliceConfig.validOutputs[0];
        aliceConfig.validInputs[0] = bobConfig.validOutputs[0];

        vm.recordLogs();
        vm.prank(alice);
        iOrderbook.addOrder(aliceConfig);
        Vm.Log[] memory entries = vm.getRecordedLogs();
        (,, OrderV3 memory aliceOrder,) = abi.decode(entries[2].data, (address, address, OrderV3, bytes32));

        vm.recordLogs();
        vm.prank(bob);
        iOrderbook.addOrder(bobConfig);
        entries = vm.getRecordedLogs();
        (,, OrderV3 memory bobOrder,) = abi.decode(entries[2].data, (address, address, OrderV3, bytes32));

        ClearConfig memory clearConfig = ClearConfig(0, 0, 0, 0, 0, 0);

        // Create a flash borrower.
        Reenteroor borrower = new Reenteroor();
        checkFlashLoanNotRevert(
            borrower,
            abi.encodeWithSelector(
                IOrderBookV4.clear.selector,
                aliceOrder,
                bobOrder,
                clearConfig,
                new SignedContextV1[](0),
                new SignedContextV1[](0)
            ),
            loanAmount
        );
    }
}
