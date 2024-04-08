// SPDX-License-Identifier: CAL
pragma solidity =0.8.19;

import {RouteProcessorOrderBookV4ArbOrderTakerTest} from
    "test/util/abstract/RouteProcessorOrderBookV4ArbOrderTakerTest.sol";
import {
    OrderV3,
    EvaluableV3,
    IExpressionDeployerV3,
    TakeOrderConfigV3,
    TakeOrdersConfigV3
} from "rain.orderbook.interface/interface/unstable/IOrderBookV4.sol";
import {
    RouteProcessorOrderBookV4ArbOrderTaker,
    OrderBookV4ArbConfigV1,
    MinimumOutput
} from "src/concrete/arb/RouteProcessorOrderBookV4ArbOrderTaker.sol";

contract RouteProcessorOrderBookV4ArbOrderTakerSenderTest is RouteProcessorOrderBookV4ArbOrderTakerTest {
    function testRouteProcessorTakeOrdersSender(OrderV3 memory order, uint256 inputIOIndex, uint256 outputIOIndex)
        public
    {
        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        RouteProcessorOrderBookV4ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV2(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))), 0
        );
    }

    function testRouteProcessorMinimumOutput(
        OrderV3 memory order,
        uint256 inputIOIndex,
        uint256 outputIOIndex,
        uint256 minimumOutput,
        uint256 mintAmount
    ) public {
        mintAmount = bound(mintAmount, 0, type(uint256).max - 1);
        minimumOutput = bound(minimumOutput, mintAmount + 1, type(uint256).max);
        iTakerOutput.mint(iArb, mintAmount);

        TakeOrderConfigV3[] memory orders = buildTakeOrderConfig(order, inputIOIndex, outputIOIndex);

        vm.expectRevert(abi.encodeWithSelector(MinimumOutput.selector, minimumOutput, mintAmount));
        RouteProcessorOrderBookV4ArbOrderTaker(iArb).arb(
            TakeOrdersConfigV3(0, type(uint256).max, type(uint256).max, orders, abi.encode(bytes("0x00"))),
            minimumOutput
        );
    }
}
