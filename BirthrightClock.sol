// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "./TimeToken.sol";

/**
 * @title BirthrightClock
 * @notice Issues 1 TIME/day to every verified unique human.
 *
 * The birthright is the economic floor of the parallel society.
 * Every verified human starts each day with 1 TIME — regardless of
 * whether they performed paid work. This is not welfare; it is the
 * acknowledgement that human existence itself has value.
 *
 * Sybil resistance is provided by World ID nullifiers. Each World ID
 * nullifier may claim exactly one birthright per day. The nullifier
 * is a ZK proof that the claimant is a unique human — no identity
 * is revealed beyond that fact.
 *
 * Integration note: In the Logos LSSA environment, the World ID
 * verifier address is the canonical IWorldID contract deployed to
 * the LSSA execution environment.
 */
contract BirthrightClock {
    TimeToken public immutable timeToken;

    /// @notice World ID verifier interface (nullifier-based)
    IWorldID public immutable worldId;

    /// @notice App ID for TIME Protocol World ID scope
    uint256 public immutable appId;

    /// @notice Action ID for birthright claim
    uint256 public constant ACTION_ID = uint256(keccak256("time-protocol/birthright/v1"));

    /// @notice day number => nullifier hash => claimed
    mapping(uint256 => mapping(uint256 => bool)) public claimed;

    event BirthrightClaimed(
        address indexed recipient,
        uint256 indexed dayNumber,
        uint256 nullifierHash
    );

    constructor(
        address _timeToken,
        address _worldId,
        uint256 _appId
    ) {
        timeToken = TimeToken(_timeToken);
        worldId = IWorldID(_worldId);
        appId = _appId;
    }

    /**
     * @notice Claim the daily birthright allocation.
     * @dev Caller must supply a valid World ID ZK proof. One claim per nullifier per day.
     * @param recipient     The address to receive 1 TIME
     * @param root          World ID Merkle tree root
     * @param nullifierHash Unique per-person per-day identifier (privacy-preserving)
     * @param proof         ZK proof of World ID membership
     */
    function claim(
        address recipient,
        uint256 root,
        uint256 nullifierHash,
        uint256[8] calldata proof
    ) external {
        uint256 dayNumber = currentDay();

        require(
            !claimed[dayNumber][nullifierHash],
            "BirthrightClock: already claimed today"
        );

        // Verify the World ID ZK proof
        worldId.verifyProof(
            root,
            appId,
            abi.encodePacked(recipient).toUint256(),
            nullifierHash,
            ACTION_ID,
            proof
        );

        claimed[dayNumber][nullifierHash] = true;

        timeToken.mintBirthright(recipient, dayNumber);

        emit BirthrightClaimed(recipient, dayNumber, nullifierHash);
    }

    /**
     * @notice Current day number (unix timestamp / 86400).
     * @dev Used as the deduplication key for daily claims.
     */
    function currentDay() public view returns (uint256) {
        return block.timestamp / 86400;
    }

    /**
     * @notice Check if a nullifier has claimed today's birthright.
     */
    function hasClaimedToday(uint256 nullifierHash) external view returns (bool) {
        return claimed[currentDay()][nullifierHash];
    }
}

// ─── Minimal World ID interface ────────────────────────────────────────────────

interface IWorldID {
    function verifyProof(
        uint256 root,
        uint256 groupId,
        uint256 signalHash,
        uint256 nullifierHash,
        uint256 externalNullifierHash,
        uint256[8] calldata proof
    ) external view;
}

// ─── Utility ───────────────────────────────────────────────────────────────────

library BytesUtils {
    function toUint256(bytes memory b) internal pure returns (uint256) {
        return uint256(keccak256(b));
    }
}

using BytesUtils for bytes;
