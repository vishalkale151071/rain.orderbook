import { Bytes, ethereum } from "@graphprotocol/graph-ts";
import { AddOrderV2, RemoveOrderV2 } from "../generated/OrderBook/OrderBook";
import { AddOrder, Order, RemoveOrder } from "../generated/schema";
import { vaultEntityId } from "./vault";
import { eventId } from "./interfaces/event";
import { createTransactionEntity } from "./transaction";

export function handleAddOrder(event: AddOrderV2): void {
  createOrderEntity(event);
  createAddOrderEntity(event);
}

export function handleRemoveOrder(event: RemoveOrderV2): void {
  let order = Order.load(event.params.orderHash);
  if (order != null) {
    order.active = false;
    order.save();
  }
  createRemoveOrderEntity(event);
}

export function createOrderEntity(event: AddOrderV2): void {
  let order = new Order(event.params.orderHash);
  order.active = true;
  order.orderHash = event.params.orderHash;
  order.owner = event.params.sender;
  let sender = event.params.sender;

  order.inputs = [];
  order.outputs = [];

  for (let i = 0; i < event.params.order.validInputs.length; i++) {
    let input = event.params.order.validInputs[i];
    let vaultId = input.vaultId;
    let token = input.token;
    let vault = vaultEntityId(sender, vaultId, token);
    order.inputs.push(vault);
  }

  for (let i = 0; i < event.params.order.validOutputs.length; i++) {
    let output = event.params.order.validOutputs[i];
    let vaultId = output.vaultId;
    let token = output.token;
    let vault = vaultEntityId(sender, vaultId, token);
    order.outputs.push(vault);
  }

  order.nonce = event.params.order.nonce;
  order.orderBytes = ethereum.encode(event.parameters[2].value)!;
  order.save();
}

export function createAddOrderEntity(event: AddOrderV2): void {
  let addOrder = new AddOrder(event.transaction.hash);
  addOrder.id = eventId(event);
  addOrder.order = event.params.orderHash;
  addOrder.sender = event.params.sender;
  addOrder.transaction = createTransactionEntity(event);
  addOrder.save();
}

export function createRemoveOrderEntity(event: RemoveOrderV2): void {
  let removeOrder = new RemoveOrder(event.transaction.hash);
  removeOrder.id = eventId(event);
  removeOrder.order = event.params.orderHash;
  removeOrder.sender = event.params.sender;
  removeOrder.transaction = createTransactionEntity(event);
  removeOrder.save();
}