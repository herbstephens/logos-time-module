import { createLightNode, waitForRemotePeer, LightNode } from "@waku/sdk";
import { ethers } from "ethers";
import { WorkAgreement, SignedWorkAgreement } from "./types";

const CONTENT_TOPIC = "/time/1/work-agreements/proto";

/**
 * Publishes a countersigned WorkAgreement to the Waku content topic,
 * signalling to all listening Logos nodes that payment is about to occur.
 *
 * The typical flow:
 *   1. Worker creates agreement and signs it
 *   2. Worker sends signed agreement to payer (off-band or via Waku DM)
 *   3. Payer countersigns → agreement is now valid
 *   4. Payer calls this function to broadcast the countersigned agreement
 *   5. Payer calls TimeProtocol.payForWork() on-chain
 *   6. Logos TIME module detects the Waku message and the on-chain event
 */
export async function publishWorkAgreement(
  agreement: WorkAgreement,
  payerWallet: ethers.Wallet,
  wakuNodeUrl: string = "ws://localhost:8546"
): Promise<void> {
  console.log(`[WorkAgreement] Publishing agreement ${agreement.id} to Waku`);

  // Sign the agreement as payer
  const agreementHash = computeAgreementHash(agreement);
  const payerSignature = await payerWallet.signMessage(agreementHash);

  const signed: SignedWorkAgreement = {
    ...agreement,
    payerSignature,
    agreementHash,
    publishedAt: Math.floor(Date.now() / 1000),
  };

  // Connect to Waku
  const node = await createLightNode({ defaultBootstrap: true });
  await node.start();
  await waitForRemotePeer(node);

  // Encode and publish
  const payload = new TextEncoder().encode(JSON.stringify(signed));
  await node.lightPush.send(
    { contentTopic: CONTENT_TOPIC, version: 1 },
    { payload }
  );

  console.log(
    `[WorkAgreement] Published — worker: ${agreement.worker}, ` +
    `hours: ${agreement.hoursWorkedScaled / 1000}, ` +
    `hash: ${agreementHash.slice(0, 10)}...`
  );

  await node.stop();
}

/**
 * Create a new WorkAgreement.
 * The worker calls this, signs it, and sends it to the payer for countersigning.
 */
export function createWorkAgreement(params: {
  worker:            string;
  payer:             string;
  hoursWorkedScaled: number;   // hours * 1000 (e.g. 1500 = 1.5h)
  paymentToken:      string;   // ERC-20 address or ethers.ZeroAddress for native
  paymentAmount:     bigint;
  description:       string;
  expiresInHours?:   number;
}): WorkAgreement {
  const now = Math.floor(Date.now() / 1000);
  return {
    id: ethers.hexlify(ethers.randomBytes(32)),
    worker:            params.worker,
    payer:             params.payer,
    hoursWorkedScaled: params.hoursWorkedScaled,
    paymentToken:      params.paymentToken,
    paymentAmount:     params.paymentAmount.toString(),
    description:       params.description,
    createdAt:         now,
    expiresAt:         params.expiresInHours
      ? now + params.expiresInHours * 3600
      : 0,
  };
}

/**
 * Sign a WorkAgreement as the worker.
 */
export async function signAsWorker(
  agreement: WorkAgreement,
  workerWallet: ethers.Wallet
): Promise<WorkAgreement & { workerSignature: string }> {
  const hash = computeAgreementHash(agreement);
  const workerSignature = await workerWallet.signMessage(hash);
  return { ...agreement, workerSignature };
}

/**
 * Verify both signatures on a countersigned WorkAgreement.
 */
export function verifySignatures(signed: SignedWorkAgreement): boolean {
  try {
    const hash = computeAgreementHash(signed);

    const workerRecovered = ethers.verifyMessage(hash, signed.workerSignature);
    const payerRecovered  = ethers.verifyMessage(hash, signed.payerSignature);

    const workerOk = workerRecovered.toLowerCase() === signed.worker.toLowerCase();
    const payerOk  = payerRecovered.toLowerCase()  === signed.payer.toLowerCase();

    if (!workerOk) console.error("[WorkAgreement] Worker signature invalid");
    if (!payerOk)  console.error("[WorkAgreement] Payer signature invalid");

    return workerOk && payerOk;
  } catch {
    return false;
  }
}

function computeAgreementHash(agreement: WorkAgreement): string {
  const encoded = ethers.AbiCoder.defaultAbiCoder().encode(
    ["bytes32", "address", "address", "uint256", "address", "uint256", "uint256", "uint256"],
    [
      agreement.id,
      agreement.worker,
      agreement.payer,
      agreement.hoursWorkedScaled,
      agreement.paymentToken,
      BigInt(agreement.paymentAmount),
      agreement.createdAt,
      agreement.expiresAt,
    ]
  );
  return ethers.keccak256(encoded);
}
