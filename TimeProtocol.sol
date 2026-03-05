// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./TimeToken.sol";
import "./WorkNFT.sol";

/**
 * @title TimeProtocol
 * @notice The orchestrator for the TIME Protocol mint event.
 *
 * This contract is the single entry point for all work payments.
 * When a payer calls `payForWork()`, the protocol simultaneously:
 *   1. Transfers payment from payer to worker (in any ERC-20 or native token)
 *   2. Mints TIME tokens to the worker
 *   3. Mints a soulbound WorkNFT to the worker
 *   4. Emits a MintEvent for the Logos Core module to index
 *
 * There is no other way to create TIME. Payment for work IS the mint event.
 *
 * The TIME amount minted is calculated from the agreed hourly rate and duration,
 * with a maximum of 23 TIME/day from earned work (24 max including birthright).
 *
 * ┌─────────────────────────────────────────────────────────────┐
 * │  PAYMENT FOR WORK  ────────────────────────►  MINT EVENT    │
 * │                                                             │
 * │  payer.payForWork(worker, hours, rate, currency)            │
 * │       │                                                     │
 * │       ├──► Transfer payment to worker                      │
 * │       ├──► Mint TIME tokens to worker                      │
 * │       └──► Mint WorkNFT (soulbound) to worker              │
 * └─────────────────────────────────────────────────────────────┘
 */
contract TimeProtocol {
    TimeToken public immutable timeToken;
    WorkNFT   public immutable workNFT;

    /// @notice Maximum TIME mintable per day from earned work (23 hours)
    uint256 public constant MAX_EARNED_PER_DAY = 23 ether; // 23 TIME

    /// @notice Scale factor for hours (1000 = 1.0 hours, 500 = 0.5 hours)
    uint256 public constant HOURS_SCALE = 1000;

    /// @notice Tracks TIME minted per worker per day to enforce daily cap
    /// worker => day => TIME minted
    mapping(address => mapping(uint256 => uint256)) public dailyMinted;

    // ─── Events ──────────────────────────────────────────────────────────────

    /**
     * @notice Emitted on every successful mint event.
     * @dev The Logos Core TIME module indexes this event to build the work registry.
     */
    event MintEvent(
        address indexed worker,
        address indexed payer,
        uint256 timeAmount,
        uint256 workNftId,
        uint256 hoursWorkedScaled,  // hours * 1000
        bytes32 agreementHash,
        address paymentToken,
        uint256 paymentAmount
    );

    event PaymentRouted(
        address indexed from,
        address indexed to,
        address token,
        uint256 amount
    );

    // ─── Errors ───────────────────────────────────────────────────────────────

    error DailyCapExceeded(address worker, uint256 attempted, uint256 remaining);
    error InvalidHours(uint256 hoursScaled);
    error AgreementHashMismatch();
    error PaymentFailed();

    // ─── Constructor ──────────────────────────────────────────────────────────

    constructor(address _timeToken, address _workNFT) {
        timeToken = TimeToken(_timeToken);
        workNFT   = WorkNFT(_workNFT);
    }

    // ─── Core Function: The Mint Event ────────────────────────────────────────

    /**
     * @notice Pay for work. This IS the mint event.
     *
     * @param worker            Address of the worker receiving payment + TIME + WorkNFT
     * @param hoursWorkedScaled Hours worked, scaled by 1000 (e.g., 1500 = 1.5 hours)
     * @param paymentToken      ERC-20 token used for payment (address(0) for native)
     * @param paymentAmount     Amount of payment token to transfer to worker
     * @param contentUri        Logos Storage URI for encrypted work metadata
     * @param agreementHash     Keccak256 of the signed off-chain WorkAgreement
     */
    function payForWork(
        address worker,
        uint256 hoursWorkedScaled,
        address paymentToken,
        uint256 paymentAmount,
        string calldata contentUri,
        bytes32 agreementHash
    ) external payable returns (uint256 workNftId) {
        require(worker != address(0), "TimeProtocol: worker is zero address");
        require(hoursWorkedScaled > 0, "TimeProtocol: zero hours");
        require(hoursWorkedScaled <= 23 * HOURS_SCALE, "TimeProtocol: exceeds 23 hour max");

        // ── 1. Calculate TIME amount to mint ──────────────────────────────────
        // TIME is minted at 1:1 with hours worked (1 TIME = 1 hour)
        // hoursWorkedScaled / HOURS_SCALE * 1e18
        uint256 timeAmount = (hoursWorkedScaled * 1 ether) / HOURS_SCALE;

        // ── 2. Check daily cap ────────────────────────────────────────────────
        uint256 day = block.timestamp / 86400;
        uint256 alreadyMinted = dailyMinted[worker][day];
        uint256 remaining = MAX_EARNED_PER_DAY > alreadyMinted
            ? MAX_EARNED_PER_DAY - alreadyMinted
            : 0;

        if (timeAmount > remaining) {
            revert DailyCapExceeded(worker, timeAmount, remaining);
        }

        // ── 3. Route payment from payer → worker ──────────────────────────────
        if (paymentToken == address(0)) {
            // Native token payment
            require(msg.value == paymentAmount, "TimeProtocol: incorrect native payment");
            (bool success,) = worker.call{value: paymentAmount}("");
            if (!success) revert PaymentFailed();
        } else {
            // ERC-20 payment
            IERC20(paymentToken).transferFrom(msg.sender, worker, paymentAmount);
        }
        emit PaymentRouted(msg.sender, worker, paymentToken, paymentAmount);

        // ── 4. Mint WorkNFT (soulbound) ───────────────────────────────────────
        workNftId = workNFT.mint(
            worker,
            msg.sender,
            timeAmount,
            hoursWorkedScaled,
            contentUri,
            agreementHash
        );

        // ── 5. Mint TIME tokens ───────────────────────────────────────────────
        dailyMinted[worker][day] += timeAmount;
        timeToken.mintForWork(worker, timeAmount, bytes32(workNftId), msg.sender);

        // ── 6. Emit the canonical MintEvent ───────────────────────────────────
        emit MintEvent(
            worker,
            msg.sender,
            timeAmount,
            workNftId,
            hoursWorkedScaled,
            agreementHash,
            paymentToken,
            paymentAmount
        );
    }

    /**
     * @notice How much TIME can a worker still earn today?
     */
    function remainingDailyCapFor(address worker) external view returns (uint256) {
        uint256 day = block.timestamp / 86400;
        uint256 minted = dailyMinted[worker][day];
        return minted >= MAX_EARNED_PER_DAY ? 0 : MAX_EARNED_PER_DAY - minted;
    }
}

// ─── Minimal ERC-20 interface ─────────────────────────────────────────────────

interface IERC20 {
    function transferFrom(address from, address to, uint256 amount) external returns (bool);
}
