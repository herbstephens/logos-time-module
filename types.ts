// types.ts — shared TypeScript types for the TIME Protocol Waku integration

export interface WorkAgreement {
  id:                string;   // bytes32 hex
  worker:            string;   // address
  payer:             string;   // address
  hoursWorkedScaled: number;   // hours * 1000
  paymentToken:      string;   // ERC-20 address or ZeroAddress
  paymentAmount:     string;   // uint256 as decimal string
  description:       string;
  createdAt:         number;   // unix timestamp
  expiresAt:         number;   // unix timestamp (0 = no expiry)
  workerSignature?:  string;
}

export interface SignedWorkAgreement extends WorkAgreement {
  workerSignature: string;
  payerSignature:  string;
  agreementHash:   string;
  publishedAt:     number;
}
