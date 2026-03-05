// SPDX-License-Identifier: MIT
pragma solidity ^0.8.24;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";

/**
 * @title WorkNFT
 * @notice Soulbound (ERC-5192) NFT representing a verified unit of completed work.
 *
 * Every mint event in TIME Protocol produces two outputs simultaneously:
 *   - TIME tokens (transferable, the currency)
 *   - A WorkNFT (non-transferable, the permanent record)
 *
 * WorkNFTs are the identity layer of TIME Protocol. They are:
 *   - Permanently bound to the worker's address
 *   - Non-transferable (any transfer attempt reverts)
 *   - Selectively disclosable (the owner chooses what metadata to reveal)
 *   - Stored in Logos decentralised storage (content-addressed URI)
 *
 * The collection of WorkNFTs in a wallet is the worker's verifiable contribution history —
 * a reputation layer that no capital-holder can buy.
 */
contract WorkNFT is ERC721, AccessControl {
    bytes32 public constant MINTER_ROLE = keccak256("MINTER_ROLE");

    /// @notice ERC-5192: Minimal Soulbound NFTs interface
    /// @dev https://eips.ethereum.org/EIPS/eip-5192
    event Locked(uint256 tokenId);

    uint256 private _tokenIdCounter;

    struct WorkRecord {
        address worker;         // Who did the work
        address payer;          // Who paid for the work
        uint256 timeAmount;     // How much TIME was minted (in wei)
        uint256 hoursWorked;    // Duration in hours (scaled x1000 for precision)
        uint256 timestamp;      // Block timestamp of payment
        string  contentUri;     // Logos Storage URI for encrypted work metadata
        bytes32 agreementHash;  // Keccak256 of the signed WorkAgreement message
    }

    mapping(uint256 => WorkRecord) public workRecords;

    /// @notice ERC-5192: All tokens are locked (soulbound) by default
    mapping(uint256 => bool) private _locked;

    constructor(address admin) ERC721("TIME Work Record", "WORK") {
        _grantRole(DEFAULT_ADMIN_ROLE, admin);
    }

    /**
     * @notice Mint a soulbound WorkNFT to a worker upon verified payment.
     * @dev Called exclusively by TimeProtocol.sol. Emits Locked per ERC-5192.
     * @param worker        The worker's address (soul the NFT binds to)
     * @param payer         The payer's address
     * @param timeAmount    TIME tokens minted alongside this NFT
     * @param hoursWorked   Hours * 1000 (e.g., 1500 = 1.5 hours)
     * @param contentUri    Logos Storage content address for encrypted work metadata
     * @param agreementHash Hash of the signed off-chain WorkAgreement
     * @return tokenId      The ID of the newly minted WorkNFT
     */
    function mint(
        address worker,
        address payer,
        uint256 timeAmount,
        uint256 hoursWorked,
        string calldata contentUri,
        bytes32 agreementHash
    ) external onlyRole(MINTER_ROLE) returns (uint256) {
        require(worker != address(0), "WorkNFT: mint to zero address");

        uint256 tokenId = ++_tokenIdCounter;
        _safeMint(worker, tokenId);

        workRecords[tokenId] = WorkRecord({
            worker:        worker,
            payer:         payer,
            timeAmount:    timeAmount,
            hoursWorked:   hoursWorked,
            timestamp:     block.timestamp,
            contentUri:    contentUri,
            agreementHash: agreementHash
        });

        // ERC-5192: Lock immediately upon mint
        _locked[tokenId] = true;
        emit Locked(tokenId);

        return tokenId;
    }

    // ─── ERC-5192: Soulbound enforcement ───────────────────────────────────────

    /**
     * @notice Returns true — all WorkNFTs are permanently locked.
     * @dev ERC-5192 compliance.
     */
    function locked(uint256 tokenId) external view returns (bool) {
        require(_ownerOf(tokenId) != address(0), "WorkNFT: token does not exist");
        return _locked[tokenId];
    }

    /**
     * @dev Block all transfers. WorkNFTs are soulbound — they cannot move.
     */
    function _update(
        address to,
        uint256 tokenId,
        address auth
    ) internal override returns (address) {
        address from = _ownerOf(tokenId);
        // Allow minting (from == address(0)) but block all transfers
        if (from != address(0)) {
            revert("WorkNFT: soulbound — transfers are disabled");
        }
        return super._update(to, tokenId, auth);
    }

    /**
     * @notice Returns the Logos Storage URI for a given token.
     */
    function tokenURI(uint256 tokenId) public view override returns (string memory) {
        require(_ownerOf(tokenId) != address(0), "WorkNFT: token does not exist");
        return workRecords[tokenId].contentUri;
    }

    /**
     * @notice How many WorkNFTs does an address hold? This is their contribution count.
     */
    function contributionCount(address worker) external view returns (uint256) {
        return balanceOf(worker);
    }

    // Required override for AccessControl + ERC721
    function supportsInterface(
        bytes4 interfaceId
    ) public view override(ERC721, AccessControl) returns (bool) {
        // ERC-5192 interface ID
        return interfaceId == 0xb45a3c0e || super.supportsInterface(interfaceId);
    }
}
