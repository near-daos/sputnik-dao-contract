"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
/**
 * Unimplemented types

    /// Postponed Action Receipt.
    PostponedReceipt(Box<Receipt>),
    /// Received data from DataReceipt encoded in base64 for the given account_id and data_id.
    ReceivedData {
        account_id: AccountId,
        data_id: CryptoHash,
        #[serde(with = "option_base64_format")]
        data: Option<Vec<u8>>,
    },
    /// Delayed Receipt.
    /// The receipt was delayed because the shard was overwhelmed.
    DelayedReceipt(Box<Receipt>),
 */
//# sourceMappingURL=types.js.map