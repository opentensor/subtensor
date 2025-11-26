import * as Bytes from './Bytes.js';
import type { OneOf } from './internal/types.js';
/** Type that defines a Binary State Tree instance. */
export type BinaryStateTree = {
    root: Node;
};
/** Type defining a node of the BST. */
export type Node = OneOf<EmptyNode | StemNode | InternalNode>;
/**
 * Creates a new Binary State Tree instance.
 *
 * @example
 * ```ts twoslash
 * import { BinaryStateTree } from 'ox'
 *
 * const tree = BinaryStateTree.create()
 * ```
 *
 * @returns A Binary State Tree.
 */
export declare function create(): BinaryStateTree;
/**
 * Inserts a key-value pair into the Binary State Tree.
 *
 * @example
 * ```ts twoslash
 * import { BinaryStateTree, Bytes } from 'ox'
 *
 * const tree = BinaryStateTree.create()
 *
 * BinaryStateTree.insert( // [!code focus]
 *   tree, // [!code focus]
 *   Bytes.fromHex('0xe34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54'), // [!code focus]
 *   Bytes.fromHex('0xd4fd4e189132273036449fc9e11198c739161b4c0116a9a2dccdfa1c492006f1') // [!code focus]
 * ) // [!code focus]
 * ```
 *
 * @param tree - Binary State Tree instance.
 * @param key - Key to insert.
 * @param value - Value to insert.
 */
export declare function insert(tree: BinaryStateTree, key: Bytes.Bytes, value: Bytes.Bytes): void;
/**
 * Merkelizes a Binary State Tree.
 *
 * @example
 * ```ts twoslash
 * import { BinaryStateTree, Bytes } from 'ox'
 *
 * const tree = BinaryStateTree.create()
 *
 * BinaryStateTree.insert(
 *   tree,
 *   Bytes.fromHex('0xe34f199b19b2b4f47f68442619d555527d244f78a3297ea89325f843f87b8b54'),
 *   Bytes.fromHex('0xd4fd4e189132273036449fc9e11198c739161b4c0116a9a2dccdfa1c492006f1')
 * )
 *
 * const hash = BinaryStateTree.merkelize(tree) // [!code focus]
 * ```
 *
 * @param tree - Binary State Tree instance.
 * @returns Merkle hash.
 */
export declare function merkelize(tree: BinaryStateTree): Bytes.Bytes;
/** @internal */
type EmptyNode = {
    type: 'empty';
};
/** @internal */
type InternalNode = {
    left: Node;
    right: Node;
    type: 'internal';
};
/** @internal */
type StemNode = {
    stem: Bytes.Bytes;
    values: (Bytes.Bytes | undefined)[];
    type: 'stem';
};
export {};
//# sourceMappingURL=BinaryStateTree.d.ts.map