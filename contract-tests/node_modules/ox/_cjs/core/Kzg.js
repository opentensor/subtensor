"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.versionedHashVersion = void 0;
exports.from = from;
exports.versionedHashVersion = 1;
function from(value) {
    const { blobToKzgCommitment, computeBlobKzgProof } = value;
    return {
        blobToKzgCommitment,
        computeBlobKzgProof,
    };
}
//# sourceMappingURL=Kzg.js.map