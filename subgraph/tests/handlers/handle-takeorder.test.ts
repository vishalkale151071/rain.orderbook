import {
  test,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
  assert,
} from "matchstick-as";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import {
  Evaluable,
  IO,
  createAddOrderEvent,
  createTakeOrderEvent,
} from "../event-mocks.test";
import { handleAddOrder } from "../../src/order";
import { handleTakeOrder } from "../../src/takeorder";

describe("Add and remove orders", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleTakeOrder()", () => {
    let event = createTakeOrderEvent(
      Address.fromString("0x1111111111111111111111111111111111111111"),
      Address.fromString("0x2222222222222222222222222222222222222222"),
      [
        new IO(
          Address.fromString("0x3333333333333333333333333333333333333333"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      [
        new IO(
          Address.fromString("0x4444444444444444444444444444444444444444"),
          BigInt.fromI32(18),
          BigInt.fromI32(1)
        ),
      ],
      Bytes.fromHexString("0x5555555555555555555555555555555555555555"),
      new Evaluable(
        Address.fromString("0x6666666666666666666666666666666666666666"),
        Address.fromString("0x7777777777777777777777777777777777777777"),
        Bytes.fromHexString("0x8888888888888888888888888888888888888888")
      ),
      BigInt.fromI32(1),
      BigInt.fromI32(1)
    );

    handleTakeOrder(event);

    // After this, we should have:
    // - 1 TakeOrder
    // - 2 Vaults
    // - 2 TradeVaultBalanceChanges
    // - 1 Trade

    assert.entityCount("TakeOrder", 1);
    assert.entityCount("Vault", 2);
    assert.entityCount("TradeVaultBalanceChange", 2);
    assert.entityCount("Trade", 1);
  });
});