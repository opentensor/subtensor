"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.create = create;
exports.insert = insert;
exports.merkelize = merkelize;
const blake3_1 = require("@noble/hashes/blake3");
const Bytes = require("./Bytes.js");
function create() {
    return {
        root: emptyNode(),
    };
}
function insert(tree, key, value) {
    const stem = Bytes.slice(key, 0, 31);
    const subIndex = Bytes.slice(key, 31)[0];
    if (tree.root.type === 'empty') {
        tree.root = stemNode(stem);
        tree.root.values[subIndex] = value;
        return;
    }
    function inner(node_, stem, subIndex, value, depth) {
        let node = node_;
        if (node.type === 'empty') {
            node = stemNode(stem);
            node.values[subIndex] = value;
            return node;
        }
        const stemBits = bytesToBits(stem);
        if (node.type === 'stem') {
            if (Bytes.isEqual(node.stem, stem)) {
                node.values[subIndex] = value;
                return node;
            }
            const existingStemBits = bytesToBits(node.stem);
            return splitLeaf(node, stemBits, existingStemBits, subIndex, value, depth);
        }
        if (node.type === 'internal') {
            const bit = stemBits[depth];
            if (bit === 0) {
                node.left = inner(node.left, stem, subIndex, value, depth + 1);
            }
            else {
                node.right = inner(node.right, stem, subIndex, value, depth + 1);
            }
            return node;
        }
        return emptyNode();
    }
    tree.root = inner(tree.root, stem, subIndex, value, 0);
}
function merkelize(tree) {
    function inner(node) {
        if (node.type === 'empty')
            return new Uint8Array(32).fill(0);
        if (node.type === 'internal') {
            const hash_left = inner(node.left);
            const hash_right = inner(node.right);
            return hash(Bytes.concat(hash_left, hash_right));
        }
        let level = node.values.map(hash);
        while (level.length > 1) {
            const level_ = [];
            for (let i = 0; i < level.length; i += 2)
                level_.push(hash(Bytes.concat(level[i], level[i + 1])));
            level = level_;
        }
        return hash(Bytes.concat(node.stem, new Uint8Array(1).fill(0), level[0]));
    }
    return inner(tree.root);
}
function splitLeaf(leaf, stemBits, existingStemBits, subIndex, value, depth) {
    if (stemBits[depth] === existingStemBits[depth]) {
        const internal = internalNode();
        const bit = stemBits[depth];
        if (bit === 0) {
            internal.left = splitLeaf(leaf, stemBits, existingStemBits, subIndex, value, depth + 1);
        }
        else {
            internal.right = splitLeaf(leaf, stemBits, existingStemBits, subIndex, value, depth + 1);
        }
        return internal;
    }
    const internal = internalNode();
    const bit = stemBits[depth];
    const stem = bitsToBytes(stemBits);
    if (bit === 0) {
        internal.left = stemNode(stem);
        internal.left.values[subIndex] = value;
        internal.right = leaf;
    }
    else {
        internal.right = stemNode(stem);
        internal.right.values[subIndex] = value;
        internal.left = leaf;
    }
    return internal;
}
function emptyNode() {
    return {
        type: 'empty',
    };
}
function internalNode() {
    return {
        left: emptyNode(),
        right: emptyNode(),
        type: 'internal',
    };
}
function stemNode(stem) {
    return {
        stem,
        values: Array.from({ length: 256 }, () => undefined),
        type: 'stem',
    };
}
function bytesToBits(bytes) {
    const bits = [];
    for (const byte of bytes)
        for (let i = 0; i < 8; i++)
            bits.push((byte >> (7 - i)) & 1);
    return bits;
}
function bitsToBytes(bits) {
    const byte_data = new Uint8Array(bits.length / 8);
    for (let i = 0; i < bits.length; i += 8) {
        let byte = 0;
        for (let j = 0; j < 8; j++)
            byte |= bits[i + j] << (7 - j);
        byte_data[i / 8] = byte;
    }
    return byte_data;
}
function hash(bytes) {
    if (!bytes)
        return new Uint8Array(32).fill(0);
    if (!bytes.some((byte) => byte !== 0))
        return new Uint8Array(32).fill(0);
    return (0, blake3_1.blake3)(bytes);
}
//# sourceMappingURL=BinaryStateTree.js.map