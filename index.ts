/**
 * TIME Protocol — Waku Integration
 *
 * Off-chain coordination layer for TIME Protocol work agreements.
 * Uses Logos messaging (Waku) for private, censorship-resistant
 * coordination between workers and payers before the on-chain mint event.
 *
 * Usage:
 *   import { createWorkAgreement, signAsWorker, publishWorkAgreement } from "logos-time-waku";
 *   import { PaymentWatcher } from "logos-time-waku";
 */

export { createWorkAgreement, signAsWorker, publishWorkAgreement, verifySignatures } from "./workAgreement";
export { PaymentWatcher } from "./paymentWatcher";
export type { WorkAgreement, SignedWorkAgreement } from "./types";
export type { MintEventData } from "./paymentWatcher";

// ─── Example: Full mint event flow ─────────────────────────────────────────────
//
// import { ethers } from "ethers";
// import { createWorkAgreement, signAsWorker, publishWorkAgreement } from "./workAgreement";
// import { PaymentWatcher } from "./paymentWatcher";
//
// const workerWallet = new ethers.Wallet(process.env.WORKER_KEY!);
// const payerWallet  = new ethers.Wallet(process.env.PAYER_KEY!);
//
// // 1. Worker creates and signs agreement
// const agreement = createWorkAgreement({
//   worker:            workerWallet.address,
//   payer:             payerWallet.address,
//   hoursWorkedScaled: 2000,          // 2.0 hours
//   paymentToken:      ethers.ZeroAddress,  // native token
//   paymentAmount:     ethers.parseEther("0.01"),
//   description:       "Frontend development — TIME Protocol landing page",
//   expiresInHours:    24,
// });
// const workerSigned = await signAsWorker(agreement, workerWallet);
//
// // 2. Worker sends to payer; payer publishes countersigned agreement to Waku
// await publishWorkAgreement(workerSigned, payerWallet);
//
// // 3. Payer submits on-chain payment — this IS the mint event
// const watcher = await PaymentWatcher.connect(process.env.LOGOS_RPC!, CONTRACT_ADDRESS);
// watcher.onMint(event => console.log("TIME minted:", event));
// await watcher.start();
//
// await watcher.payForWork({
//   signer:            payerWallet.connect(provider),
//   worker:            workerWallet.address,
//   hoursWorkedScaled: 2000,
//   paymentToken:      ethers.ZeroAddress,
//   paymentAmount:     ethers.parseEther("0.01"),
//   contentUri:        "logos://pending/tbd",
//   agreementHash:     workerSigned.agreementHash!,
// });
//
// // Result: 2 TIME tokens + 1 soulbound WorkNFT minted to worker
