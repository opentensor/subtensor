import { blake3 } from '@noble/hashes/blake3'

import * as Bytes from './Bytes.js'
import type { OneOf } from './internal/types.js'

/** Type that defines a Binary State Tree instance. */
export type BinaryStateTree = {
  root: Node
}

/** Type defining a node of the BST. */
export type Node = OneOf<EmptyNode | StemNode | InternalNode>

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
export function create(): BinaryStateTree {
  return {
    root: emptyNode(),
  }
}

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
export function insert(
  tree: BinaryStateTree,
  key: Bytes.Bytes,
  value: Bytes.Bytes,
): void {
  const stem = Bytes.slice(key, 0, 31)
  const subIndex = Bytes.slice(key, 31)[0]!

  if (tree.root.type === 'empty') {
    tree.root = stemNode(stem)
    tree.root.values[subIndex] = value
    return
  }

  function inner(
    node_: Node,
    stem: Bytes.Bytes,
    subIndex: number,
    value: Bytes.Bytes,
    depth: number,
  ): Node {
    let node = node_

    if (node.type === 'empty') {
      node = stemNode(stem)
      node.values[subIndex!] = value
      return node
    }

    const stemBits = bytesToBits(stem)
    if (node.type === 'stem') {
      if (Bytes.isEqual(node.stem, stem)) {
        node.values[subIndex!] = value
        return node
      }
      const existingStemBits = bytesToBits(node.stem)
      return splitLeaf(node, stemBits, existingStemBits, subIndex, value, depth)
    }

    if (node.type === 'internal') {
      const bit = stemBits[depth]
      if (bit === 0) {
        node.left = inner(node.left, stem, subIndex, value, depth + 1)
      } else {
        node.right = inner(node.right, stem, subIndex, value, depth + 1)
      }
      return node
    }

    return emptyNode()
  }
  tree.root = inner(tree.root, stem, subIndex, value, 0)
}

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
export function merkelize(tree: BinaryStateTree): Bytes.Bytes {
  function inner(node: Node): Bytes.Bytes {
    if (node.type === 'empty') return new Uint8Array(32).fill(0)
    if (node.type === 'internal') {
      const hash_left = inner(node.left)
      const hash_right = inner(node.right)
      return hash(Bytes.concat(hash_left, hash_right))
    }

    let level = node.values.map(hash)
    while (level.length > 1) {
      const level_ = []
      for (let i = 0; i < level.length; i += 2)
        level_.push(hash(Bytes.concat(level[i]!, level[i + 1]!)))
      level = level_
    }

    return hash(Bytes.concat(node.stem, new Uint8Array(1).fill(0), level[0]!))
  }

  return inner(tree.root)
}

//////////////////////////////////////////////////////////////////////////////
// Internal
//////////////////////////////////////////////////////////////////////////////

/** @internal */
type EmptyNode = {
  type: 'empty'
}

/** @internal */
type InternalNode = {
  left: Node
  right: Node
  type: 'internal'
}

/** @internal */
type StemNode = {
  stem: Bytes.Bytes
  values: (Bytes.Bytes | undefined)[]
  type: 'stem'
}

/** @internal */
function splitLeaf(
  leaf: Node,
  stemBits: number[],
  existingStemBits: number[],
  subIndex: number,
  value: Bytes.Bytes,
  depth: number,
): Node {
  if (stemBits[depth] === existingStemBits[depth]) {
    const internal = internalNode()
    const bit = stemBits[depth]
    if (bit === 0) {
      internal.left = splitLeaf(
        leaf,
        stemBits,
        existingStemBits,
        subIndex,
        value,
        depth + 1,
      )
    } else {
      internal.right = splitLeaf(
        leaf,
        stemBits,
        existingStemBits,
        subIndex,
        value,
        depth + 1,
      )
    }
    return internal
  }

  const internal = internalNode()
  const bit = stemBits[depth]
  const stem = bitsToBytes(stemBits)
  if (bit === 0) {
    internal.left = stemNode(stem)
    internal.left.values[subIndex] = value
    internal.right = leaf
  } else {
    internal.right = stemNode(stem)
    internal.right.values[subIndex] = value
    internal.left = leaf
  }
  return internal
}

/** @internal */
function emptyNode(): EmptyNode {
  return {
    type: 'empty',
  }
}

/** @internal */
function internalNode(): InternalNode {
  return {
    left: emptyNode(),
    right: emptyNode(),
    type: 'internal',
  }
}

/** @internal */
function stemNode(stem: Bytes.Bytes): StemNode {
  return {
    stem,
    values: Array.from({ length: 256 }, () => undefined),
    type: 'stem',
  }
}

/** @internal */
function bytesToBits(bytes: Bytes.Bytes): number[] {
  const bits = []
  for (const byte of bytes)
    for (let i = 0; i < 8; i++) bits.push((byte >> (7 - i)) & 1)
  return bits
}

/** @internal */
function bitsToBytes(bits: number[]): Bytes.Bytes {
  const byte_data = new Uint8Array(bits.length / 8)
  for (let i = 0; i < bits.length; i += 8) {
    let byte = 0
    for (let j = 0; j < 8; j++) byte |= bits[i + j]! << (7 - j)
    byte_data[i / 8] = byte
  }
  return byte_data
}

/** @internal */
function hash(bytes: Bytes.Bytes | undefined): Bytes.Bytes {
  if (!bytes) return new Uint8Array(32).fill(0)
  if (!bytes.some((byte) => byte !== 0)) return new Uint8Array(32).fill(0)
  return blake3(bytes)
}
