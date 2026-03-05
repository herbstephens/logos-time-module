import { ethers } from "ethers";

const TIME_PROTOCOL_ABI = [
  "event MintEvent(address indexed worker, address indexed payer, uint256 timeAmount, uint256 workNftId, uint256 hoursWorkedScaled, bytes32 agreementHash, address paymentToken, uint256 paymentAmount)",
  "function payForWork(address worker, uint256 hoursWorkedScaled, address paymentToken, uint256 paymentAmount, string contentUri, bytes32 agreementHash) payable returns (uint256 workNftId)",
  "function remainingDailyCapFor(address worker) view returns (uint256)",
];

export interface MintEventData {
  worker:            string;
  payer:             string;
  timeAmount:        bigint;
  workNftId:         bigint;
  hoursWorkedScaled: bigint;
  agreementHash:     string;
  paymentToken:      string;
  paymentAmount:     bigint;
  txHash:            string;
  blockNumber:       number;
}

/**
 * Watches the Logos blockchain for MintEvent emissions from TimeProtocol.sol.
 * Call `onMint` to register callbacks for confirmed mint events.
 */
export class PaymentWatcher {
  private contract: ethers.Contract;
  private callbacks: Array<(event: MintEventData) => void> = [];

  constructor(
    private readonly provider:        ethers.WebSocketProvider,
    private readonly contractAddress: string,
  ) {
    this.contract = new ethers.Contract(contractAddress, TIME_PROTOCOL_ABI, provider);
  }

  static async connect(
    rpcUrl:          string,
    contractAddress: string
  ): Promise<PaymentWatcher> {
    const provider = new ethers.WebSocketProvider(rpcUrl);
    return new PaymentWatcher(provider, contractAddress);
  }

  /** Register a callback to fire on every confirmed mint event. */
  onMint(callback: (event: MintEventData) => void): void {
    this.callbacks.push(callback);
  }

  /** Begin listening for MintEvent logs. */
  async start(): Promise<void> {
    console.log(`[PaymentWatcher] Listening for MintEvents on ${this.contractAddress}`);

    this.contract.on("MintEvent", (
      worker, payer, timeAmount, workNftId,
      hoursWorkedScaled, agreementHash, paymentToken, paymentAmount,
      event
    ) => {
      const data: MintEventData = {
        worker,
        payer,
        timeAmount,
        workNftId,
        hoursWorkedScaled,
        agreementHash,
        paymentToken,
        paymentAmount,
        txHash:      event.log.transactionHash,
        blockNumber: event.log.blockNumber,
      };

      const timeFormatted = Number(timeAmount) / 1e18;
      console.log(
        `[PaymentWatcher] ✓ MINT EVENT — ` +
        `worker: ${worker.slice(0,8)}..., ` +
        `TIME: ${timeFormatted.toFixed(3)}, ` +
        `WorkNFT #${workNftId}, ` +
        `block: ${data.blockNumber}`
      );

      this.callbacks.forEach(cb => cb(data));
    });
  }

  /** Stop listening. */
  async stop(): Promise<void> {
    this.contract.removeAllListeners("MintEvent");
    await this.provider.destroy();
    console.log("[PaymentWatcher] Stopped");
  }

  /**
   * Execute a payForWork transaction as the payer.
   * This is the convenience method for client applications.
   */
  async payForWork(params: {
    signer:            ethers.Signer;
    worker:            string;
    hoursWorkedScaled: number;
    paymentToken:      string;
    paymentAmount:     bigint;
    contentUri:        string;
    agreementHash:     string;
  }): Promise<ethers.TransactionReceipt> {
    const connected = this.contract.connect(params.signer);

    const isNative = params.paymentToken === ethers.ZeroAddress;

    const tx = await (connected as any).payForWork(
      params.worker,
      params.hoursWorkedScaled,
      params.paymentToken,
      params.paymentAmount,
      params.contentUri,
      params.agreementHash,
      { value: isNative ? params.paymentAmount : 0n }
    );

    console.log(`[PaymentWatcher] payForWork tx submitted: ${tx.hash}`);
    const receipt = await tx.wait();
    console.log(`[PaymentWatcher] payForWork confirmed in block ${receipt.blockNumber}`);
    return receipt;
  }

  /**
   * Check how much TIME a worker can still earn today.
   */
  async remainingDailyCapFor(worker: string): Promise<bigint> {
    return this.contract.remainingDailyCapFor(worker);
  }
}
