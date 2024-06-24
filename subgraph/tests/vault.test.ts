import {
  test,
  assert,
  clearStore,
  describe,
  afterEach,
  clearInBlockStore,
} from "matchstick-as";
import { handleVaultBalanceChange, vaultEntityId } from "../src/vault";
import { Bytes, BigInt, Address } from "@graphprotocol/graph-ts";
import { createDepositEvent, createWithdrawEvent } from "./event-mocks.test";

describe("Vault balance changes", () => {
  afterEach(() => {
    clearStore();
    clearInBlockStore();
  });

  test("handleVaultBalanceChange()", () => {
    handleVaultBalanceChange(
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      0
    );

    let vaultId = vaultEntityId(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("handleVaultDeposit()", () => {
    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender,
      0
    );

    let vaultId = vaultEntityId(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("handleVaultWithdraw()", () => {
    // first we need to deposit
    let depositEvent = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200)
    );

    handleVaultBalanceChange(
      depositEvent.params.vaultId,
      depositEvent.params.token,
      depositEvent.params.amount,
      depositEvent.params.sender,
      0
    );

    // then we withdraw
    let event = createWithdrawEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(200),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender,
      1
    );

    let vaultId = vaultEntityId(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });

  test("If vault does not exist, create it", () => {
    assert.entityCount("Vault", 0);

    let event = createDepositEvent(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      Address.fromString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(1),
      BigInt.fromI32(100)
    );
    handleVaultBalanceChange(
      event.params.vaultId,
      event.params.token,
      event.params.amount,
      event.params.sender,
      0
    );

    let vaultId = vaultEntityId(
      Address.fromString("0x0987654321098765432109876543210987654321"),
      BigInt.fromI32(1),
      Address.fromString("0x1234567890123456789012345678901234567890")
    );

    assert.entityCount("Vault", 1);
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "balance",
      BigInt.fromI32(100).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "token",
      "0x1234567890123456789012345678901234567890"
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "vaultId",
      BigInt.fromI32(1).toString()
    );
    assert.fieldEquals(
      "Vault",
      vaultId.toHexString(),
      "owner",
      "0x0987654321098765432109876543210987654321"
    );
  });
  test("handleVaultBalanceChange returns 0 if vault doesn't exist yet", () => {
    let oldBalance = handleVaultBalanceChange(
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      0
    );

    assert.bigIntEquals(oldBalance, BigInt.fromI32(0));
  });

  test("handleVaultBalanceChange returns old balance if vault exists", () => {
    handleVaultBalanceChange(
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      0
    );

    let oldBalance = handleVaultBalanceChange(
      BigInt.fromI32(1),
      Bytes.fromHexString("0x1234567890123456789012345678901234567890"),
      BigInt.fromI32(100),
      Bytes.fromHexString("0x0987654321098765432109876543210987654321"),
      0
    );

    assert.bigIntEquals(oldBalance, BigInt.fromI32(100));
  });
});