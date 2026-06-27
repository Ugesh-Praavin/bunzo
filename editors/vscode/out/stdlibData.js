"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.HOVER_DOCS = exports.SIGNATURES = exports.MODULE_FUNCTIONS = exports.STDLIB_MODULES = void 0;
const node_1 = require("vscode-languageserver/node");
exports.STDLIB_MODULES = [
    'vector', 'deque', 'stack', 'queue', 'priority_queue',
    'set', 'hashset', 'map', 'hashmap', 'bitset',
    'math', 'os', 'json', 'http', 'db', 'string', 'filesystem', 'path',
    'time', 'random', 'crypto', 'encoding', 'process', 'environment',
    'io', 'networking', 'thread', 'mutex', 'rwlock', 'channel', 'atomic',
    'regex', 'algorithm', 'numeric', 'test'
];
exports.MODULE_FUNCTIONS = {
    // vector
    'vector': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'vector.new() -> Vector<T>\n\nCreates a new dynamic array.' },
        { label: 'with_capacity', kind: node_1.CompletionItemKind.Function, documentation: 'vector.with_capacity(capacity: int) -> Vector<T>\n\nCreates a vector with pre-allocated capacity.' },
        { label: 'push', kind: node_1.CompletionItemKind.Function, documentation: 'vector.push(v: Vector<T>, value: T) -> void\n\nAppends an element to the back.' },
        { label: 'pop', kind: node_1.CompletionItemKind.Function, documentation: 'vector.pop(v: Vector<T>) -> T\n\nRemoves and returns the last element.' },
        { label: 'get', kind: node_1.CompletionItemKind.Function, documentation: 'vector.get(v: Vector<T>, index: int) -> T\n\nGets element at index.' },
        { label: 'set', kind: node_1.CompletionItemKind.Function, documentation: 'vector.set(v: Vector<T>, index: int, value: T) -> void\n\nSets element at index.' },
        { label: 'insert', kind: node_1.CompletionItemKind.Function, documentation: 'vector.insert(v: Vector<T>, index: int, value: T) -> void\n\nInserts element at index.' },
        { label: 'remove', kind: node_1.CompletionItemKind.Function, documentation: 'vector.remove(v: Vector<T>, index: int) -> T\n\nRemoves and returns element at index.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'vector.clear(v: Vector<T>) -> void\n\nClears all elements.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'vector.len(v: Vector<T>) -> int\n\nReturns the number of elements.' },
        { label: 'capacity', kind: node_1.CompletionItemKind.Function, documentation: 'vector.capacity(v: Vector<T>) -> int\n\nReturns current capacity.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'vector.is_empty(v: Vector<T>) -> bool\n\nReturns true if empty.' },
        { label: 'front', kind: node_1.CompletionItemKind.Function, documentation: 'vector.front(v: Vector<T>) -> T\n\nReturns first element.' },
        { label: 'back', kind: node_1.CompletionItemKind.Function, documentation: 'vector.back(v: Vector<T>) -> T\n\nReturns last element.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'vector.contains(v: Vector<T>, val: T) -> bool\n\nChecks if element is present.' },
        { label: 'index_of', kind: node_1.CompletionItemKind.Function, documentation: 'vector.index_of(v: Vector<T>, val: T) -> int\n\nReturns index of element or -1.' },
        { label: 'reverse', kind: node_1.CompletionItemKind.Function, documentation: 'vector.reverse(v: Vector<T>) -> void\n\nReverses elements in-place.' },
        { label: 'sort', kind: node_1.CompletionItemKind.Function, documentation: 'vector.sort(v: Vector<T>) -> void\n\nSorts elements in-place.' },
        { label: 'resize', kind: node_1.CompletionItemKind.Function, documentation: 'vector.resize(v: Vector<T>, size: int) -> void\n\nResizes vector.' },
        { label: 'swap', kind: node_1.CompletionItemKind.Function, documentation: 'vector.swap(v: Vector<T>, a: int, b: int) -> void\n\nSwaps two element positions.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'vector.iter(v: Vector<T>) -> Array<T>\n\nReturns iterable view.' }
    ],
    // deque
    'deque': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'deque.new() -> Deque<T>\n\nCreates a double-ended queue.' },
        { label: 'push_back', kind: node_1.CompletionItemKind.Function, documentation: 'deque.push_back(d: Deque<T>, val: T) -> void\n\nAppends to the back.' },
        { label: 'push_front', kind: node_1.CompletionItemKind.Function, documentation: 'deque.push_front(d: Deque<T>, val: T) -> void\n\nPrepends to the front.' },
        { label: 'pop_back', kind: node_1.CompletionItemKind.Function, documentation: 'deque.pop_back(d: Deque<T>) -> T\n\nRemoves from the back.' },
        { label: 'pop_front', kind: node_1.CompletionItemKind.Function, documentation: 'deque.pop_front(d: Deque<T>) -> T\n\nRemoves from the front.' },
        { label: 'front', kind: node_1.CompletionItemKind.Function, documentation: 'deque.front(d: Deque<T>) -> T\n\nReturns front element.' },
        { label: 'back', kind: node_1.CompletionItemKind.Function, documentation: 'deque.back(d: Deque<T>) -> T\n\nReturns back element.' },
        { label: 'get', kind: node_1.CompletionItemKind.Function, documentation: 'deque.get(d: Deque<T>, index: int) -> T\n\nReturns element at index.' },
        { label: 'set', kind: node_1.CompletionItemKind.Function, documentation: 'deque.set(d: Deque<T>, index: int, val: T) -> void\n\nSets element at index.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'deque.len(d: Deque<T>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'deque.is_empty(d: Deque<T>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'deque.clear(d: Deque<T>) -> void\n\nClears all.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'deque.contains(d: Deque<T>, val: T) -> bool\n\nChecks if exists.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'deque.iter(d: Deque<T>) -> Array<T>\n\nReturns iterable view.' }
    ],
    // stack
    'stack': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'stack.new() -> Stack<T>\n\nCreates a LIFO stack.' },
        { label: 'push', kind: node_1.CompletionItemKind.Function, documentation: 'stack.push(s: Stack<T>, val: T) -> void\n\nPushes element to stack.' },
        { label: 'pop', kind: node_1.CompletionItemKind.Function, documentation: 'stack.pop(s: Stack<T>) -> T\n\nPops top element.' },
        { label: 'top', kind: node_1.CompletionItemKind.Function, documentation: 'stack.top(s: Stack<T>) -> T\n\nReturns top element.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'stack.len(s: Stack<T>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'stack.is_empty(s: Stack<T>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'stack.clear(s: Stack<T>) -> void\n\nClears stack.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'stack.contains(s: Stack<T>, val: T) -> bool\n\nChecks if exists.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'stack.iter(s: Stack<T>) -> Array<T>\n\nReturns iterator.' }
    ],
    // queue
    'queue': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'queue.new() -> Queue<T>\n\nCreates a FIFO queue.' },
        { label: 'push', kind: node_1.CompletionItemKind.Function, documentation: 'queue.push(q: Queue<T>, val: T) -> void\n\nEnqueues element.' },
        { label: 'pop', kind: node_1.CompletionItemKind.Function, documentation: 'queue.pop(q: Queue<T>) -> T\n\nDequeues front element.' },
        { label: 'front', kind: node_1.CompletionItemKind.Function, documentation: 'queue.front(q: Queue<T>) -> T\n\nReturns front element.' },
        { label: 'back', kind: node_1.CompletionItemKind.Function, documentation: 'queue.back(q: Queue<T>) -> T\n\nReturns back element.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'queue.len(q: Queue<T>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'queue.is_empty(q: Queue<T>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'queue.clear(q: Queue<T>) -> void\n\nClears queue.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'queue.contains(q: Queue<T>, val: T) -> bool\n\nChecks if exists.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'queue.iter(q: Queue<T>) -> Array<T>\n\nReturns iterator.' }
    ],
    // priority_queue
    'priority_queue': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.new() -> PriorityQueue<T>\n\nCreates priority queue.' },
        { label: 'push', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.push(pq: PriorityQueue<T>, val: T) -> void\n\nPushes element.' },
        { label: 'pop', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.pop(pq: PriorityQueue<T>) -> T\n\nPops top priority element.' },
        { label: 'top', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.top(pq: PriorityQueue<T>) -> T\n\nReturns top element.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.len(pq: PriorityQueue<T>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.is_empty(pq: PriorityQueue<T>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.clear(pq: PriorityQueue<T>) -> void\n\nClears queue.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.contains(pq: PriorityQueue<T>, val: T) -> bool\n\nChecks if exists.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'priority_queue.iter(pq: PriorityQueue<T>) -> Array<T>\n\nReturns iterator.' }
    ],
    // set
    'set': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'set.new() -> Set<T>\n\nCreates an ordered Set.' },
        { label: 'insert', kind: node_1.CompletionItemKind.Function, documentation: 'set.insert(s: Set<T>, val: T) -> void\n\nInserts element.' },
        { label: 'remove', kind: node_1.CompletionItemKind.Function, documentation: 'set.remove(s: Set<T>, val: T) -> void\n\nRemoves element.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'set.contains(s: Set<T>, val: T) -> bool\n\nChecks existence.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'set.len(s: Set<T>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'set.is_empty(s: Set<T>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'set.clear(s: Set<T>) -> void\n\nClears set.' },
        { label: 'first', kind: node_1.CompletionItemKind.Function, documentation: 'set.first(s: Set<T>) -> T\n\nReturns first element.' },
        { label: 'last', kind: node_1.CompletionItemKind.Function, documentation: 'set.last(s: Set<T>) -> T\n\nReturns last element.' },
        { label: 'min', kind: node_1.CompletionItemKind.Function, documentation: 'set.min(s: Set<T>) -> T\n\nReturns minimum element.' },
        { label: 'max', kind: node_1.CompletionItemKind.Function, documentation: 'set.max(s: Set<T>) -> T\n\nReturns maximum element.' },
        { label: 'lower_bound', kind: node_1.CompletionItemKind.Function, documentation: 'set.lower_bound(s: Set<T>, val: T) -> T\n\nReturns lower bound.' },
        { label: 'upper_bound', kind: node_1.CompletionItemKind.Function, documentation: 'set.upper_bound(s: Set<T>, val: T) -> T\n\nReturns upper bound.' },
        { label: 'union', kind: node_1.CompletionItemKind.Function, documentation: 'set.union(a: Set<T>, b: Set<T>) -> Set<T>\n\nReturns union of sets.' },
        { label: 'intersection', kind: node_1.CompletionItemKind.Function, documentation: 'set.intersection(a: Set<T>, b: Set<T>) -> Set<T>\n\nReturns intersection.' },
        { label: 'difference', kind: node_1.CompletionItemKind.Function, documentation: 'set.difference(a: Set<T>, b: Set<T>) -> Set<T>\n\nReturns difference.' },
        { label: 'symmetric_difference', kind: node_1.CompletionItemKind.Function, documentation: 'set.symmetric_difference(a: Set<T>, b: Set<T>) -> Set<T>\n\nReturns symmetric diff.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'set.iter(s: Set<T>) -> Array<T>\n\nReturns elements.' }
    ],
    // hashset
    'hashset': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.new() -> HashSet<T>\n\nCreates unordered HashSet.' },
        { label: 'insert', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.insert(s: HashSet<T>, val: T) -> void\n\nInserts element.' },
        { label: 'remove', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.remove(s: HashSet<T>, val: T) -> void\n\nRemoves element.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.contains(s: HashSet<T>, val: T) -> bool\n\nChecks existence.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.len(s: HashSet<T>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.is_empty(s: HashSet<T>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.clear(s: HashSet<T>) -> void\n\nClears hashset.' },
        { label: 'union', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.union(a: HashSet<T>, b: HashSet<T>) -> HashSet<T>\n\nReturns union.' },
        { label: 'intersection', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.intersection(a: HashSet<T>, b: HashSet<T>) -> HashSet<T>\n\nReturns intersection.' },
        { label: 'difference', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.difference(a: HashSet<T>, b: HashSet<T>) -> HashSet<T>\n\nReturns difference.' },
        { label: 'symmetric_difference', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.symmetric_difference(a: HashSet<T>, b: HashSet<T>) -> HashSet<T>\n\nReturns symmetric diff.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'hashset.iter(s: HashSet<T>) -> Array<T>\n\nReturns iterator.' }
    ],
    // map
    'map': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'map.new() -> Map<K, V>\n\nCreates an ordered Map.' },
        { label: 'insert', kind: node_1.CompletionItemKind.Function, documentation: 'map.insert(m: Map<K, V>, key: K, val: V) -> void\n\nInserts key-value.' },
        { label: 'get', kind: node_1.CompletionItemKind.Function, documentation: 'map.get(m: Map<K, V>, key: K) -> V\n\nReturns value for key.' },
        { label: 'remove', kind: node_1.CompletionItemKind.Function, documentation: 'map.remove(m: Map<K, V>, key: K) -> void\n\nRemoves key-value.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'map.contains(m: Map<K, V>, key: K) -> bool\n\nChecks if key exists.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'map.len(m: Map<K, V>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'map.is_empty(m: Map<K, V>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'map.clear(m: Map<K, V>) -> void\n\nClears map.' },
        { label: 'keys', kind: node_1.CompletionItemKind.Function, documentation: 'map.keys(m: Map<K, V>) -> Array<K>\n\nReturns sorted keys.' },
        { label: 'values', kind: node_1.CompletionItemKind.Function, documentation: 'map.values(m: Map<K, V>) -> Array<V>\n\nReturns sorted values.' },
        { label: 'first_key', kind: node_1.CompletionItemKind.Function, documentation: 'map.first_key(m: Map<K, V>) -> K\n\nReturns first key.' },
        { label: 'last_key', kind: node_1.CompletionItemKind.Function, documentation: 'map.last_key(m: Map<K, V>) -> K\n\nReturns last key.' },
        { label: 'first_value', kind: node_1.CompletionItemKind.Function, documentation: 'map.first_value(m: Map<K, V>) -> V\n\nReturns first value.' },
        { label: 'last_value', kind: node_1.CompletionItemKind.Function, documentation: 'map.last_value(m: Map<K, V>) -> V\n\nReturns last value.' },
        { label: 'lower_bound', kind: node_1.CompletionItemKind.Function, documentation: 'map.lower_bound(m: Map<K, V>, key: K) -> K\n\nReturns lower bound.' },
        { label: 'upper_bound', kind: node_1.CompletionItemKind.Function, documentation: 'map.upper_bound(m: Map<K, V>, key: K) -> K\n\nReturns upper bound.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'map.iter(m: Map<K, V>) -> Array<K>\n\nReturns iterator.' }
    ],
    // hashmap
    'hashmap': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.new() -> HashMap<K, V>\n\nCreates unordered HashMap.' },
        { label: 'insert', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.insert(m: HashMap<K, V>, key: K, val: V) -> void\n\nInserts key-value.' },
        { label: 'get', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.get(m: HashMap<K, V>, key: K) -> V\n\nReturns value for key.' },
        { label: 'remove', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.remove(m: HashMap<K, V>, key: K) -> void\n\nRemoves key-value.' },
        { label: 'contains', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.contains(m: HashMap<K, V>, key: K) -> bool\n\nChecks if key exists.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.len(m: HashMap<K, V>) -> int\n\nReturns size.' },
        { label: 'is_empty', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.is_empty(m: HashMap<K, V>) -> bool\n\nChecks if empty.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.clear(m: HashMap<K, V>) -> void\n\nClears map.' },
        { label: 'keys', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.keys(m: HashMap<K, V>) -> Array<K>\n\nReturns keys.' },
        { label: 'values', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.values(m: HashMap<K, V>) -> Array<V>\n\nReturns values.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'hashmap.iter(m: HashMap<K, V>) -> Array<K>\n\nReturns iterator.' }
    ],
    // bitset
    'bitset': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.new(size: int) -> BitSet\n\nCreates a new BitSet.' },
        { label: 'set', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.set(b: BitSet, index: int) -> void\n\nSets bit to true.' },
        { label: 'reset', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.reset(b: BitSet, index: int) -> void\n\nSets bit to false.' },
        { label: 'flip', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.flip(b: BitSet, index: int) -> void\n\nFlips bit.' },
        { label: 'test', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.test(b: BitSet, index: int) -> bool\n\nTests bit value.' },
        { label: 'count', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.count(b: BitSet) -> int\n\nReturns count of set bits.' },
        { label: 'any', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.any(b: BitSet) -> bool\n\nReturns true if any bit is set.' },
        { label: 'none', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.none(b: BitSet) -> bool\n\nReturns true if no bits set.' },
        { label: 'all', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.all(b: BitSet) -> bool\n\nReturns true if all bits set.' },
        { label: 'len', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.len(b: BitSet) -> int\n\nReturns size.' },
        { label: 'clear', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.clear(b: BitSet) -> void\n\nResets all bits.' },
        { label: 'and', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.and(a: BitSet, b: BitSet) -> BitSet\n\nBitwise AND.' },
        { label: 'or', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.or(a: BitSet, b: BitSet) -> BitSet\n\nBitwise OR.' },
        { label: 'xor', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.xor(a: BitSet, b: BitSet) -> BitSet\n\nBitwise XOR.' },
        { label: 'not', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.not(b: BitSet) -> BitSet\n\nBitwise NOT.' },
        { label: 'iter', kind: node_1.CompletionItemKind.Function, documentation: 'bitset.iter(b: BitSet) -> Array<bool>\n\nReturns iterator.' }
    ],
    // process
    'process': [
        { label: 'exec', kind: node_1.CompletionItemKind.Function, documentation: 'process.exec(cmd: String) -> String\n\nExecutes command.' },
        { label: 'pid', kind: node_1.CompletionItemKind.Function, documentation: 'process.pid() -> int\n\nReturns current process ID.' }
    ],
    // environment
    'environment': [
        { label: 'get', kind: node_1.CompletionItemKind.Function, documentation: 'environment.get(key: String) -> String\n\nGets env var.' },
        { label: 'set', kind: node_1.CompletionItemKind.Function, documentation: 'environment.set(key: String, val: String) -> void\n\nSets env var.' },
        { label: 'has', kind: node_1.CompletionItemKind.Function, documentation: 'environment.has(key: String) -> bool\n\nChecks env var existence.' },
        { label: 'remove', kind: node_1.CompletionItemKind.Function, documentation: 'environment.remove(key: String) -> void\n\nRemoves env var.' },
        { label: 'all', kind: node_1.CompletionItemKind.Function, documentation: 'environment.all() -> Map<String, String>\n\nGets all env vars.' }
    ],
    // io
    'io': [
        { label: 'read_line', kind: node_1.CompletionItemKind.Function, documentation: 'io.read_line() -> String\n\nReads line from stdin.' },
        { label: 'read_char', kind: node_1.CompletionItemKind.Function, documentation: 'io.read_char() -> String\n\nReads char from stdin.' }
    ],
    // networking
    'networking': [
        { label: 'tcp_listen', kind: node_1.CompletionItemKind.Function, documentation: 'networking.tcp_listen(addr: String) -> int\n\nBinds TcpListener.' },
        { label: 'tcp_accept', kind: node_1.CompletionItemKind.Function, documentation: 'networking.tcp_accept(listener: int) -> int\n\nAccepts TCP client.' },
        { label: 'tcp_connect', kind: node_1.CompletionItemKind.Function, documentation: 'networking.tcp_connect(addr: String) -> int\n\nConnects TCP stream.' },
        { label: 'tcp_send', kind: node_1.CompletionItemKind.Function, documentation: 'networking.tcp_send(stream: int, data: String) -> void\n\nSends TCP data.' },
        { label: 'tcp_recv', kind: node_1.CompletionItemKind.Function, documentation: 'networking.tcp_recv(stream: int) -> String\n\nReceives TCP data.' },
        { label: 'udp_bind', kind: node_1.CompletionItemKind.Function, documentation: 'networking.udp_bind(addr: String) -> int\n\nBinds UdpSocket.' },
        { label: 'udp_send', kind: node_1.CompletionItemKind.Function, documentation: 'networking.udp_send(sock: int, addr: String, data: String) -> void\n\nSends UDP packet.' },
        { label: 'udp_recv', kind: node_1.CompletionItemKind.Function, documentation: 'networking.udp_recv(sock: int) -> String\n\nReceives UDP packet.' }
    ],
    // thread
    'thread': [
        { label: 'spawn', kind: node_1.CompletionItemKind.Function, documentation: 'thread.spawn(func: Function) -> int\n\nSpawns thread.' },
        { label: 'join', kind: node_1.CompletionItemKind.Function, documentation: 'thread.join(id: int) -> void\n\nJoins thread.' }
    ],
    // mutex
    'mutex': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'mutex.new(val: T) -> Mutex<T>\n\nCreates mutex.' },
        { label: 'lock', kind: node_1.CompletionItemKind.Function, documentation: 'mutex.lock(m: Mutex<T>) -> T\n\nLocks mutex.' },
        { label: 'unlock', kind: node_1.CompletionItemKind.Function, documentation: 'mutex.unlock(m: Mutex<T>) -> void\n\nUnlocks mutex.' }
    ],
    // rwlock
    'rwlock': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'rwlock.new(val: T) -> RWLock<T>\n\nCreates RWLock.' },
        { label: 'read', kind: node_1.CompletionItemKind.Function, documentation: 'rwlock.read(lock: RWLock<T>) -> T\n\nAcquires read lock.' },
        { label: 'write', kind: node_1.CompletionItemKind.Function, documentation: 'rwlock.write(lock: RWLock<T>) -> T\n\nAcquires write lock.' }
    ],
    // channel
    'channel': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'channel.new() -> Channel<T>\n\nCreates channel.' },
        { label: 'send', kind: node_1.CompletionItemKind.Function, documentation: 'channel.send(chan: Channel<T>, val: T) -> void\n\nSends channel item.' },
        { label: 'recv', kind: node_1.CompletionItemKind.Function, documentation: 'channel.recv(chan: Channel<T>) -> T\n\nReceives channel item.' }
    ],
    // atomic
    'atomic': [
        { label: 'new', kind: node_1.CompletionItemKind.Function, documentation: 'atomic.new(val: int) -> Atomic\n\nCreates atomic integer.' },
        { label: 'load', kind: node_1.CompletionItemKind.Function, documentation: 'atomic.load(a: Atomic) -> int\n\nLoads atomic value.' },
        { label: 'store', kind: node_1.CompletionItemKind.Function, documentation: 'atomic.store(a: Atomic, val: int) -> void\n\nStores atomic value.' },
        { label: 'add', kind: node_1.CompletionItemKind.Function, documentation: 'atomic.add(a: Atomic, val: int) -> int\n\nAdds to atomic value.' }
    ],
    // regex
    'regex': [
        { label: 'match', kind: node_1.CompletionItemKind.Function, documentation: 'regex.match(pattern: String, text: String) -> String\n\nFinds first matching text.' },
        { label: 'search', kind: node_1.CompletionItemKind.Function, documentation: 'regex.search(pattern: String, text: String) -> int\n\nReturns start index of first match.' },
        { label: 'find', kind: node_1.CompletionItemKind.Function, documentation: 'regex.find(pattern: String, text: String) -> String\n\nAlias for regex.match.' },
        { label: 'find_all', kind: node_1.CompletionItemKind.Function, documentation: 'regex.find_all(pattern: String, text: String) -> Array<String>\n\nFinds all matches.' },
        { label: 'replace', kind: node_1.CompletionItemKind.Function, documentation: 'regex.replace(pattern: String, text: String, rep: String) -> String\n\nReplaces match.' },
        { label: 'split', kind: node_1.CompletionItemKind.Function, documentation: 'regex.split(pattern: String, text: String) -> Array<String>\n\nSplits string by regex.' },
        { label: 'is_match', kind: node_1.CompletionItemKind.Function, documentation: 'regex.is_match(pattern: String, text: String) -> bool\n\nChecks pattern match.' }
    ],
    // algorithm
    'algorithm': [
        { label: 'sort', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.sort(col: Collection) -> void\n\nSorts collection.' },
        { label: 'stable_sort', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.stable_sort(col: Collection) -> void\n\nStable sorts collection.' },
        { label: 'reverse', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.reverse(col: Collection) -> void\n\nReverses collection.' },
        { label: 'shuffle', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.shuffle(col: Collection) -> void\n\nShuffles collection.' },
        { label: 'find', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.find(col: Collection, val: T) -> int\n\nFinds element index.' },
        { label: 'find_if', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.find_if(col: Collection, predicate: Function) -> int\n\nFinds element matching predicate.' },
        { label: 'binary_search', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.binary_search(col: Collection, val: T) -> bool\n\nPerforms binary search.' },
        { label: 'lower_bound', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.lower_bound(col: Collection, val: T) -> int\n\nReturns lower bound.' },
        { label: 'upper_bound', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.upper_bound(col: Collection, val: T) -> int\n\nReturns upper bound.' },
        { label: 'min', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.min(col: Collection) -> T\n\nReturns minimum element.' },
        { label: 'max', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.max(col: Collection) -> T\n\nReturns maximum element.' },
        { label: 'min_element', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.min_element(col: Collection) -> int\n\nReturns index of minimum.' },
        { label: 'max_element', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.max_element(col: Collection) -> int\n\nReturns index of maximum.' },
        { label: 'copy', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.copy(col: Collection) -> Collection\n\nCopies collection.' },
        { label: 'fill', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.fill(col: Collection, val: T) -> void\n\nFills collection with val.' },
        { label: 'rotate', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.rotate(col: Collection, pivot: int) -> void\n\nRotates collection.' },
        { label: 'swap', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.swap(col: Collection, a: int, b: int) -> void\n\nSwaps elements.' },
        { label: 'unique', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.unique(col: Collection) -> void\n\nRemoves consecutive duplicates.' },
        { label: 'count', kind: node_1.CompletionItemKind.Function, documentation: 'algorithm.count(col: Collection, val: T) -> int\n\nCounts elements.' }
    ],
    // numeric
    'numeric': [
        { label: 'min', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.min(a: T, b: T) -> T\n\nMinimum of two.' },
        { label: 'max', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.max(a: T, b: T) -> T\n\nMaximum of two.' },
        { label: 'clamp', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.clamp(val: T, min: T, max: T) -> T\n\nClamps value.' },
        { label: 'abs', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.abs(val: T) -> T\n\nAbsolute value.' },
        { label: 'gcd', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.gcd(a: int, b: int) -> int\n\nGreatest common divisor.' },
        { label: 'lcm', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.lcm(a: int, b: int) -> int\n\nLeast common multiple.' },
        { label: 'factorial', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.factorial(n: int) -> int\n\nFactorial.' },
        { label: 'average', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.average(col: Collection) -> float\n\nAverage of items.' },
        { label: 'sum', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.sum(col: Collection) -> T\n\nSum of items.' },
        { label: 'product', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.product(col: Collection) -> T\n\nProduct of items.' },
        { label: 'accumulate', kind: node_1.CompletionItemKind.Function, documentation: 'numeric.accumulate(col: Collection, init: T) -> T\n\nAccumulates items.' }
    ],
    // test
    'test': [
        { label: 'assert', kind: node_1.CompletionItemKind.Function, documentation: 'test.assert(cond: bool) -> void\n\nAsserts condition.' },
        { label: 'assert_eq', kind: node_1.CompletionItemKind.Function, documentation: 'test.assert_eq(a: T, b: T) -> void\n\nAsserts equality.' },
        { label: 'assert_ne', kind: node_1.CompletionItemKind.Function, documentation: 'test.assert_ne(a: T, b: T) -> void\n\nAsserts inequality.' },
        { label: 'assert_true', kind: node_1.CompletionItemKind.Function, documentation: 'test.assert_true(cond: bool) -> void\n\nAsserts true.' },
        { label: 'assert_false', kind: node_1.CompletionItemKind.Function, documentation: 'test.assert_false(cond: bool) -> void\n\nAsserts false.' },
        { label: 'fail', kind: node_1.CompletionItemKind.Function, documentation: 'test.fail(msg: String) -> void\n\nFails test immediately.' },
        { label: 'skip', kind: node_1.CompletionItemKind.Function, documentation: 'test.skip(msg: String) -> void\n\nSkips test.' }
    ]
};
exports.SIGNATURES = {
    // Vector
    'vector.new': { label: 'vector.new() -> Vector<T>', docs: 'Creates an empty Vector.', params: [] },
    'vector.push': { label: 'vector.push(v: Vector<T>, value: T) -> void', docs: 'Appends to vector.', params: [{ label: 'v: Vector<T>', docs: 'Vector.' }, { label: 'value: T', docs: 'Value.' }] },
    'vector.pop': { label: 'vector.pop(v: Vector<T>) -> T', docs: 'Pops last element.', params: [{ label: 'v: Vector<T>', docs: 'Vector.' }] },
    // Regex
    'regex.match': { label: 'regex.match(pattern: String, text: String) -> String', docs: 'Finds match.', params: [{ label: 'pattern: String', docs: 'Regex pattern.' }, { label: 'text: String', docs: 'Text.' }] },
    'regex.is_match': { label: 'regex.is_match(pattern: String, text: String) -> bool', docs: 'Checks match.', params: [{ label: 'pattern: String', docs: 'Regex pattern.' }, { label: 'text: String', docs: 'Text.' }] },
    'regex.replace': { label: 'regex.replace(pattern: String, text: String, rep: String) -> String', docs: 'Replaces match.', params: [{ label: 'pattern: String', docs: 'Regex pattern.' }, { label: 'text: String', docs: 'Text.' }, { label: 'rep: String', docs: 'Replacement.' }] },
    // Algorithm
    'algorithm.sort': { label: 'algorithm.sort(col: Collection) -> void', docs: 'Sorts collection.', params: [{ label: 'col: Collection', docs: 'Collection.' }] },
    'algorithm.find': { label: 'algorithm.find(col: Collection, val: T) -> int', docs: 'Finds item.', params: [{ label: 'col: Collection', docs: 'Collection.' }, { label: 'val: T', docs: 'Value.' }] },
    // Numeric
    'numeric.min': { label: 'numeric.min(a: T, b: T) -> T', docs: 'Minimum.', params: [{ label: 'a: T', docs: 'First val.' }, { label: 'b: T', docs: 'Second val.' }] },
    'numeric.max': { label: 'numeric.max(a: T, b: T) -> T', docs: 'Maximum.', params: [{ label: 'a: T', docs: 'First val.' }, { label: 'b: T', docs: 'Second val.' }] },
    'numeric.clamp': { label: 'numeric.clamp(val: T, min: T, max: T) -> T', docs: 'Clamps value.', params: [{ label: 'val: T', docs: 'Value.' }, { label: 'min: T', docs: 'Min bound.' }, { label: 'max: T', docs: 'Max bound.' }] }
};
exports.HOVER_DOCS = {
    'vector.new': '**vector.new()**: Returns a new empty vector.',
    'regex.is_match': '**regex.is_match(pattern, text)**: Returns true if text matches pattern.',
    'algorithm.sort': '**algorithm.sort(col)**: Sorts the collection in-place.',
    'numeric.clamp': '**numeric.clamp(val, min, max)**: Clamps value to bounds.'
};
//# sourceMappingURL=stdlibData.js.map