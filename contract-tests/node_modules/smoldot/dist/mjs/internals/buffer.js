// Smoldot
// Copyright (C) 2019-2022  Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
export function utf8BytesToString(buffer, offset, length) {
    checkRange(buffer, offset, length);
    // The `TextDecoder` API is supported by all major browsers and by NodeJS.
    // <https://developer.mozilla.org/en-US/docs/Web/API/TextDecoder>
    return new TextDecoder().decode(buffer.slice(offset, offset + length));
}
export function readUInt8(buffer, offset) {
    checkRange(buffer, offset, 1);
    return buffer[offset];
}
export function readUInt16BE(buffer, offset) {
    checkRange(buffer, offset, 2);
    return ((buffer[offset] << 8) | buffer[offset + 1]);
}
export function readUInt32LE(buffer, offset) {
    checkRange(buffer, offset, 4);
    return (buffer[offset] | (buffer[offset + 1] << 8) | (buffer[offset + 2] << 16)) + (buffer[offset + 3] * 0x1000000);
}
/**
 * Sets the value of a given byte in the buffer.
 *
 * This function is equivalent to `buffer[offset] = value`, except that an exception is thrown
 * if `offset` is out of range.
 */
export function writeUInt8(buffer, offset, value) {
    checkRange(buffer, offset, 1);
    buffer[offset] = value & 0xff;
}
export function writeUInt32LE(buffer, offset, value) {
    checkRange(buffer, offset, 4);
    buffer[offset + 3] = (value >>> 24) & 0xff;
    buffer[offset + 2] = (value >>> 16) & 0xff;
    buffer[offset + 1] = (value >>> 8) & 0xff;
    buffer[offset] = value & 0xff;
}
export function writeUInt64LE(buffer, offset, value) {
    checkRange(buffer, offset, 8);
    buffer[offset + 7] = Number((value >> BigInt(56)) & BigInt(0xff));
    buffer[offset + 6] = Number((value >> BigInt(48)) & BigInt(0xff));
    buffer[offset + 5] = Number((value >> BigInt(40)) & BigInt(0xff));
    buffer[offset + 4] = Number((value >> BigInt(32)) & BigInt(0xff));
    buffer[offset + 3] = Number((value >> BigInt(24)) & BigInt(0xff));
    buffer[offset + 2] = Number((value >> BigInt(16)) & BigInt(0xff));
    buffer[offset + 1] = Number((value >> BigInt(8)) & BigInt(0xff));
    buffer[offset] = Number(value & BigInt(0xff));
}
function checkRange(buffer, offset, length) {
    if (!Number.isInteger(offset) || offset < 0)
        throw new RangeError();
    if (offset + length > buffer.length)
        throw new RangeError();
}
