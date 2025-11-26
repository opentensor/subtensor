export const DEFAULT_CAPACITY = 1024;
export const DEFAULT_TTL = 30000; // 30 seconds
const MAX_TTL = 1800_000; // 30 minutes
const DISABLED_TTL = 31_536_000_000;
class LRUNode {
    key;
    #expires;
    #ttl;
    createdAt;
    next;
    prev;
    constructor(key, ttl) {
        this.key = key;
        this.#ttl = ttl;
        this.#expires = Date.now() + ttl;
        this.createdAt = Date.now();
        this.next = this.prev = this;
    }
    refresh() {
        this.#expires = Date.now() + this.#ttl;
    }
    get expiry() {
        return this.#expires;
    }
}
export class LRUCache {
    capacity;
    #data = new Map();
    #refs = new Map();
    #length = 0;
    #head;
    #tail;
    #ttl;
    constructor(capacity = DEFAULT_CAPACITY, ttl = DEFAULT_TTL) {
        // Validate capacity
        if (!Number.isInteger(capacity) || capacity < 0) {
            throw new Error(`LRUCache initialization error: 'capacity' must be a non-negative integer. Received: ${capacity}`);
        }
        // Validate ttl
        if (ttl !== null && (!Number.isFinite(ttl) || ttl < 0 || ttl > MAX_TTL)) {
            throw new Error(`LRUCache initialization error: 'ttl' must be between 0 and ${MAX_TTL} ms or null to disable. Received: ${ttl}`);
        }
        this.capacity = capacity;
        ttl ? this.#ttl = ttl : this.#ttl = DISABLED_TTL;
        this.#head = this.#tail = new LRUNode('<empty>', this.#ttl);
    }
    get ttl() {
        return this.#ttl;
    }
    get length() {
        return this.#length;
    }
    get lengthData() {
        return this.#data.size;
    }
    get lengthRefs() {
        return this.#refs.size;
    }
    entries() {
        const keys = this.keys();
        const count = keys.length;
        const entries = new Array(count);
        for (let i = 0; i < count; i++) {
            const key = keys[i];
            entries[i] = [key, this.#data.get(key)];
        }
        return entries;
    }
    keys() {
        const keys = [];
        if (this.#length) {
            let curr = this.#head;
            while (curr !== this.#tail) {
                keys.push(curr.key);
                curr = curr.next;
            }
            keys.push(curr.key);
        }
        return keys;
    }
    get(key) {
        const data = this.#data.get(key);
        if (data) {
            this.#toHead(key);
            // Evict TTL once data is refreshed
            this.#evictTTL();
            return data;
        }
        this.#evictTTL();
        return null;
    }
    set(key, value) {
        if (this.#data.has(key)) {
            this.#toHead(key);
        }
        else {
            const node = new LRUNode(key, this.#ttl);
            this.#refs.set(node.key, node);
            if (this.length === 0) {
                this.#head = this.#tail = node;
            }
            else {
                this.#head.prev = node;
                node.next = this.#head;
                this.#head = node;
            }
            if (this.#length === this.capacity) {
                this.#data.delete(this.#tail.key);
                this.#refs.delete(this.#tail.key);
                this.#tail = this.#tail.prev;
                this.#tail.next = this.#head;
            }
            else {
                this.#length += 1;
            }
        }
        // Evict TTL once data is refreshed or added
        this.#evictTTL();
        this.#data.set(key, value);
    }
    #evictTTL() {
        // Find last node to keep
        // traverse map to find the expired nodes
        while (this.#tail.expiry && this.#tail.expiry < Date.now() && this.#length > 0) {
            this.#refs.delete(this.#tail.key);
            this.#data.delete(this.#tail.key);
            this.#length -= 1;
            this.#tail = this.#tail.prev;
            this.#tail.next = this.#head;
        }
        if (this.#length === 0) {
            this.#head = this.#tail = new LRUNode('<empty>', this.#ttl);
        }
    }
    #toHead(key) {
        const ref = this.#refs.get(key);
        if (ref && ref !== this.#head) {
            ref.refresh();
            ref.prev.next = ref.next;
            ref.next.prev = ref.prev;
            ref.next = this.#head;
            this.#head.prev = ref;
            this.#head = ref;
        }
    }
}
