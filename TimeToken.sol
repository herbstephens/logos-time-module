// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title TimeToken
 * @notice ERC-20 token representing units of human work.
 *
 * 1 TIME = 1 hour of verified, compensated human labour.
 *
 * TIME is only created through two pathways:
 *   1. EARNED:    Payment for work (the mint event). Triggered by TimeProtocol.sol.
 *   2. BIRTHRIGHT: 1 TIME/day issued to every verified human. Triggered by BirthrightClock.sol.
 *
 * There is no pre-mine, no founder allocation, no investor round.
 * TIME cannot be minted by any address that does not hold MINTER_ROLE.
 * MINTER_ROLE is granted exclusively to TimeProtocol.sol and BirthrightClock.sol.
 */
contract TimeToken is ERC20, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    /// @notice Emitted when TIME is minted as payment for work
    event WorkMint(
        address indexed worker,
        address indexed payer,
        uint256 amount,
        bytes32 indexed workNftId
    );

    /// @notice Emitted when TIME is minted as a birthright allocation
    event BirthrightMint(address indexed recipient, uint256 amount, uint256 day);

    constructor(address admin) ERC20("TIME", "TIME") {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /**
     * @notice Mint TIME as compensation for verified work.
     * @dev Called exclusively by TimeProtocol.sol after payment confirmation.
     * @param worker  The address receiving TIME (the worker)
     * @param amount  Amount of TIME to mint (in wei, 18 decimals; 1e18 = 1 TIME = 1 hour)
     * @param workNftId The ID of the soulbound WorkNFT being minted alongside this
     * @param payer   The address that made the payment
     */
    function mintForWork(
        address worker,
        uint256 amount,
        bytes32 workNftId,
        address payer
    ) external onlyRole(MINTER_ROLE) {
        require(worker != address(0), "TIME: mint to zero address");
        require(amount > 0, "TIME: zero amount");
        _mint(worker, amount);
        emit WorkMint(worker, payer, amount, workNftId);
    }

    /**
     * @notice Mint the daily birthright allocation to a verified human.
     * @dev Called exclusively by BirthrightClock.sol. Sybil resistance enforced upstream.
     * @param recipient The verified human address
     * @param day       The day number (unix timestamp / 86400) for dedup
     */
    function mintBirthright(
        address recipient,
        uint256 day
    ) external onlyRole(MINTER_ROLE) {
        require(recipient != address(0), "TIME: mint to zero address");
        // 1 TIME per day = 1e18 tokens
        uint256 amount = 1 ether;
        _mint(recipient, amount);
        emit BirthrightMint(recipient, amount, day);
    }

    /**
     * @notice Returns the decimals (18, standard ERC-20).
     * @dev 1 TIME = 1e18 units. Subdivisions represent fractional hours.
     */
    function decimals() public pure override returns (uint8) {
        return 18;
    }
}
