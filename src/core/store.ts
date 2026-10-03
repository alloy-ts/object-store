import { ObjectStore } from "../../index.js";

export function createStore(): ObjectStore {
  return ObjectStore.createInMemory();
}
