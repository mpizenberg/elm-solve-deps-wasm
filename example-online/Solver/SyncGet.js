// @ts-check
import path from "node:path";
import {
  Worker,
  MessageChannel,
  receiveMessageOnPort,
} from "node:worker_threads";

/**
 * Start a worker thread.
 *
 *
 * @returns {{ get: (string: string) => string, shutDown: () => void }}
 *  a `syncGetWorker` capable of making sync requests until shut down.
 */
function startWorker() {
  const { port1: localPort, port2: workerPort } = new MessageChannel();
  const sharedLock = new SharedArrayBuffer(4);
  const sharedLockArray = new Int32Array(sharedLock);
  const workerPath = new URL(import.meta.resolve("./SyncGetWorker.js"));
  const worker = new Worker(workerPath, {
    workerData: { sharedLock, requestPort: workerPort },
    transferList: [workerPort],
  });
  function get(url) {
    worker.postMessage(url);
    Atomics.wait(sharedLockArray, 0, 0); // blocks until notified at index 0.
    const response = receiveMessageOnPort(localPort);
    if (response?.message.error) {
      throw response?.message.error;
    } else {
      return response?.message;
    }
  }
  function shutDown() {
    localPort.close();
    worker.terminate();
  }
  return { get, shutDown };
}

export { startWorker };
