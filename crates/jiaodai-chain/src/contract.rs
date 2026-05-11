//! L2 Smart Contract definitions
//!
//! Defines the TimestampRegistry Solidity contract and ABI
//! for on-chain timestamp proofs.



/// TimestampRegistry Solidity contract source code
pub const TIMESTAMP_REGISTRY_SOL: &str = r#"
// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * @title TimestampRegistry
 * @notice Records Merkle roots with on-chain timestamps for the Jiaodai platform.
 *         Each submission is a batch of content hashes rolled up into a single root.
 */
contract TimestampRegistry {
    struct TimestampEntry {
        bytes32 merkleRoot;
        uint256 timestamp;
        address submitter;
        uint256 batchCount;
    }

    // Maps batch index to TimestampEntry
    mapping(uint256 => TimestampEntry) public entries;
    uint256 public entryCount;

    // Maps merkle root to batch index for O(1) lookups
    mapping(bytes32 => uint256) public rootToIndex;

    // Events
    event TimestampRecorded(
        uint256 indexed index,
        bytes32 indexed merkleRoot,
        uint256 timestamp,
        address submitter,
        uint256 batchCount
    );

    /**
     * @notice Submit a Merkle root with batch metadata
     * @param merkleRoot The root of the Merkle tree containing content hashes
     * @param batchCount Number of entries in this batch
     */
    function submitTimestamp(bytes32 merkleRoot, uint256 batchCount) external {
        uint256 index = entryCount;
        entries[index] = TimestampEntry({
            merkleRoot: merkleRoot,
            timestamp: block.timestamp,
            submitter: msg.sender,
            batchCount: batchCount
        });
        rootToIndex[merkleRoot] = index;
        entryCount++;

        emit TimestampRecorded(index, merkleRoot, block.timestamp, msg.sender, batchCount);
    }

    /**
     * @notice Verify a Merkle root was recorded on-chain
     * @param merkleRoot The root to verify
     * @return timestamp When the root was recorded (0 if not found)
     * @return index The batch index
     */
    function verifyRoot(bytes32 merkleRoot) external view returns (uint256 timestamp, uint256 index) {
        index = rootToIndex[merkleRoot];
        if (index == 0 && entries[0].merkleRoot != merkleRoot) {
            return (0, 0);
        }
        return (entries[index].timestamp, index);
    }

    /**
     * @notice Get entry details by index
     * @param index The batch index
     */
    function getEntry(uint256 index) external view returns (
        bytes32 merkleRoot,
        uint256 timestamp,
        address submitter,
        uint256 batchCount
    ) {
        TimestampEntry storage entry = entries[index];
        return (entry.merkleRoot, entry.timestamp, entry.submitter, entry.batchCount);
    }
}
"#;

/// Contract ABI (simplified JSON)
pub const TIMESTAMP_REGISTRY_ABI: &str = r#"[
    {"inputs":[{"internalType":"bytes32","name":"merkleRoot","type":"bytes32"},{"internalType":"uint256","name":"batchCount","type":"uint256"}],"name":"submitTimestamp","outputs":[],"stateMutability":"nonpayable","type":"function"},
    {"inputs":[{"internalType":"bytes32","name":"merkleRoot","type":"bytes32"}],"name":"verifyRoot","outputs":[{"internalType":"uint256","name":"timestamp","type":"uint256"},{"internalType":"uint256","name":"index","type":"uint256"}],"stateMutability":"view","type":"function"},
    {"inputs":[{"internalType":"uint256","name":"index","type":"uint256"}],"name":"getEntry","outputs":[{"internalType":"bytes32","name":"merkleRoot","type":"bytes32"},{"internalType":"uint256","name":"timestamp","type":"uint256"},{"internalType":"address","name":"submitter","type":"address"},{"internalType":"uint256","name":"batchCount","type":"uint256"}],"stateMutability":"view","type":"function"},
    {"anonymous":false,"inputs":[{"indexed":true,"internalType":"uint256","name":"index","type":"uint256"},{"indexed":true,"internalType":"bytes32","name":"merkleRoot","type":"bytes32"},{"indexed":false,"internalType":"uint256","name":"timestamp","type":"uint256"},{"indexed":false,"internalType":"address","name":"submitter","type":"address"},{"indexed":false,"internalType":"uint256","name":"batchCount","type":"uint256"}],"name":"TimestampRecorded","type":"event"},
    {"inputs":[],"name":"entryCount","outputs":[{"internalType":"uint256","name":"","type":"uint256"}],"stateMutability":"view","type":"function"}
]"#;

/// Contract deployment bytecode (placeholder — would be compiled output)
pub const TIMESTAMP_REGISTRY_BYTECODE: &str = "0x...placeholder...";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_source_not_empty() {
        assert!(!TIMESTAMP_REGISTRY_SOL.is_empty());
        assert!(TIMESTAMP_REGISTRY_SOL.contains("TimestampRegistry"));
        assert!(TIMESTAMP_REGISTRY_SOL.contains("submitTimestamp"));
        assert!(TIMESTAMP_REGISTRY_SOL.contains("verifyRoot"));
    }

    #[test]
    fn test_abi_not_empty() {
        assert!(!TIMESTAMP_REGISTRY_ABI.is_empty());
        assert!(TIMESTAMP_REGISTRY_ABI.contains("submitTimestamp"));
        assert!(TIMESTAMP_REGISTRY_ABI.contains("verifyRoot"));
    }
}
