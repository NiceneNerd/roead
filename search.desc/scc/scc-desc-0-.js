searchState.loadedDescShard("scc", 0, "Scalable Concurrent Containers\n<code>Entry</code> stores an instance of <code>T</code> and a link to the next entry.\n<code>LinkedList</code> is a type trait implementing a lock-free singly …\n<code>Bag</code> is a lock-free concurrent unordered instance container.\nDeletes <code>self</code>.\nDeletes <code>self</code>.\nRe-exports the <code>sdd</code> crate for backward compatibility.\nReturns the argument unchanged.\n<code>HashCache</code> is a concurrent and asynchronous 32-way …\n<code>HashIndex</code> is a read-optimized concurrent and asynchronous …\n<code>HashMap</code> is a concurrent and asynchronous hash map.\n<code>HashSet</code> is a concurrent and asynchronous hash set.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if <code>self</code> is reachable and not marked.\nReturns <code>true</code> if <code>self</code> is reachable and not marked.\nReturns <code>true</code> if <code>self</code> has been deleted.\nReturns <code>true</code> if <code>self</code> has been deleted.\nReturns <code>true</code> if <code>self</code> has a mark on it.\nReturns <code>true</code> if <code>self</code> has a mark on it.\nReturns a reference to the forward link.\nMarks <code>self</code> with an internal flag to denote that <code>self</code> is in …\nMarks <code>self</code> with an internal flag to denote that <code>self</code> is in …\nReturns the closest next valid entry.\nReturns the closest next valid entry.\nReturns a <code>Shared</code> handle to the closest next valid entry.\nReturns a <code>Shared</code> handle to the closest next valid entry.\nAppends the given entry to <code>self</code> and returns a pointer to …\nAppends the given entry to <code>self</code> and returns a pointer to …\n<code>Queue</code> is a lock-free concurrent first-in-first-out …\n<code>Stack</code> is a lock-free concurrent last-in-first-out …\nExtracts the inner instance of <code>T</code>.\n<code>TreeIndex</code> is a read-optimized concurrent and asynchronous …\nRemoves any mark from <code>self</code>.\nRemoves any mark from <code>self</code>.\n<code>Bag</code> is a lock-free concurrent unordered instance container.\nAn iterator that moves out of a <code>Bag</code>.\nA mutable iterator over the entries of a <code>Bag</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the <code>Bag</code> is empty.\nIterates over contained instances for modifying them.\nReturns the number of entries in the <code>Bag</code>.\nCreates a new <code>Bag</code>.\nPops an instance in the <code>Bag</code> if not empty.\nPops all the entries at once, and folds them into an …\nPushes an instance of <code>T</code>.\nThe default maximum capacity of a <code>HashCache</code> is <code>256</code>.\n<code>Entry</code> represents a single cache entry in a <code>HashCache</code>.\n<code>EvictedEntry</code> is a type alias for <code>Option&lt;(K, V)&gt;</code>.\nScalable concurrent 32-way associative cache backed by …\nNo value.\nAn occupied entry.\n<code>OccupiedEntry</code> is a view into an occupied cache entry in a …\nSome value of type <code>T</code>.\nA vacant entry.\n<code>VacantEntry</code> is a view into a vacant cache entry in a …\nProvides in-place mutable access to an occupied entry.\nSearches for any entry that satisfies the given predicate.\nSearches for any entry that satisfies the given predicate.\nReturns the capacity of the <code>HashCache</code>.\nReturns the current capacity range of the <code>HashCache</code>.\nClears the <code>HashCache</code> by removing all key-value pairs.\nClears the <code>HashCache</code> by removing all key-value pairs.\nReturns <code>true</code> if the <code>HashCache</code> contains a value for the …\nReturns <code>true</code> if the <code>HashCache</code> contains a value for the …\nCreates an empty default <code>HashCache</code>.\nGets the entry associated with the given key in the map …\nGets the entry associated with the given key in the map …\nCompares two <code>HashCache</code> instances.\nIterates over all the entries in the <code>HashCache</code> to print …\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets an <code>OccupiedEntry</code> corresponding to the key for …\nGets a reference to the value in the entry.\nGets an <code>OccupiedEntry</code> corresponding to the key for …\nGets a mutable reference to the value in the entry.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nTakes ownership of the key.\nReturns <code>true</code> if the <code>HashCache</code> is empty.\nReturns a reference to the key of this entry.\nGets a reference to the key in the entry.\nGets a reference to the key.\nReturns the number of entries in the <code>HashCache</code>.\nCreates an empty default <code>HashCache</code>.\nEnsures a value is in the entry by putting the default …\nEnsures a value is in the entry by putting the supplied …\nEnsures a value is in the entry by putting the result of …\nEnsures a value is in the entry by putting the result of …\nPuts a key-value pair into the <code>HashCache</code>.\nSets the value of the entry, and returns the old value.\nPuts a key-value pair into the <code>HashCache</code>.\nSets the value of the entry.\nSets the value of the entry with its key, and returns an …\nReads a key-value pair.\nReads a key-value pair.\nRemoves a key-value pair if the key exists.\nTakes the value out of the entry, and returns it.\nRemoves a key-value pair if the key exists.\nTakes ownership of the key and value from the <code>HashCache</code>.\nRemoves a key-value pair if the key exists and the given …\nRemoves a key-value pair if the key exists and the given …\nRetains the entries specified by the predicate.\nRetains the entries specified by the predicate.\nScans all the entries.\nScans all the entries.\nCreates an empty <code>HashCache</code> with the specified capacity.\nCreates an empty <code>HashCache</code> with the specified capacity and …\nCreates an empty <code>HashCache</code> with the given <code>BuildHasher</code>.\n<code>Entry</code> represents a single entry in a <code>HashIndex</code>.\nScalable concurrent hash index.\nAn iterator over the entries of a <code>HashIndex</code>.\nAn occupied entry.\n<code>OccupiedEntry</code> is a view into an occupied entry in a …\n<code>Reserve</code> keeps the capacity of the associated <code>HashIndex</code> …\nA vacant entry.\n<code>VacantEntry</code> is a view into a vacant entry in a <code>HashIndex</code>.\nReturns the number of reserved slots.\nProvides in-place mutable access to an occupied entry.\nFinds any entry satisfying the supplied predicate for …\nFinds any entry satisfying the supplied predicate for …\nReturns the index of the bucket that may contain the key.\nReturns the capacity of the <code>HashIndex</code>.\nReturns the current capacity range of the <code>HashIndex</code>.\nClears the <code>HashIndex</code> by removing all key-value pairs.\nClears the <code>HashIndex</code> by removing all key-value pairs.\nReturns <code>true</code> if the <code>HashIndex</code> contains a value for the …\nCreates an empty default <code>HashIndex</code>.\nGets the entry associated with the given key in the map …\nGets the entry associated with the given key in the map …\nGets the first occupied entry for in-place manipulation.\nGets the first occupied entry for in-place manipulation.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets an <code>OccupiedEntry</code> corresponding to the key for …\nGets a reference to the value in the entry.\nGets an <code>OccupiedEntry</code> corresponding to the key for …\nGets a mutable reference to the value in the entry.\nInserts a key-value pair into the <code>HashIndex</code>.\nInserts a key-value pair into the <code>HashIndex</code>.\nSets the value of the entry with its key, and returns an …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nTakes ownership of the key.\nReturns <code>true</code> if the <code>HashIndex</code> is empty.\nReturns an <code>Iter</code>.\nReturns a reference to the key of this entry.\nGets a reference to the key in the entry.\nGets a reference to the key.\nReturns the number of entries in the <code>HashIndex</code>.\nCreates an empty default <code>HashIndex</code>.\nGets the next closest occupied entry.\nGets the next closest occupied entry.\nEnsures a value is in the entry by inserting the default …\nEnsures a value is in the entry by inserting the supplied …\nEnsures a value is in the entry by inserting the result of …\nEnsures a value is in the entry by inserting the result of …\nReturns a guarded reference to the value for the specified …\nPeeks a key-value pair without acquiring locks.\nRemoves a key-value pair if the key exists.\nRemoves a key-value pair if the key exists.\nMarks that the entry is removed from the <code>HashIndex</code>.\nRemoves a key-value pair if the key exists and the given …\nRemoves a key-value pair if the key exists and the given …\nTemporarily increases the minimum capacity of the <code>HashIndex</code>…\nRetains the entries specified by the predicate.\nRetains the entries specified by the predicate.\nUpdates the entry by inserting a new entry and marking the …\nCreates an empty <code>HashIndex</code> with the specified capacity.\nCreates an empty <code>HashIndex</code> with the specified capacity and …\nCreates an empty <code>HashIndex</code> with the given <code>BuildHasher</code>.\n<code>Entry</code> represents a single entry in a <code>HashMap</code>.\nScalable concurrent hash map.\nAn occupied entry.\n<code>OccupiedEntry</code> is a view into an occupied entry in a <code>HashMap</code>…\n<code>Reserve</code> keeps the capacity of the associated <code>HashMap</code> …\nA vacant entry.\n<code>VacantEntry</code> is a view into a vacant entry in a <code>HashMap</code>.\nReturns the number of reserved slots.\nProvides in-place mutable access to an occupied entry.\nSearches for any entry that satisfies the given predicate.\nSearches for any entry that satisfies the given predicate.\nFinds any entry satisfying the supplied predicate for …\nFinds any entry satisfying the supplied predicate for …\nReturns the index of the bucket that may contain the key.\nReturns the capacity of the <code>HashMap</code>.\nReturns the current capacity range of the <code>HashMap</code>.\nClears the <code>HashMap</code> by removing all key-value pairs.\nClears the <code>HashMap</code> by removing all key-value pairs.\nReturns <code>true</code> if the <code>HashMap</code> contains a value for the …\nReturns <code>true</code> if the <code>HashMap</code> contains a value for the …\nCreates an empty default <code>HashMap</code>.\nGets the entry associated with the given key in the map …\nGets the entry associated with the given key in the map …\nCompares two <code>HashMap</code> instances.\nGets the first occupied entry for in-place manipulation.\nGets the first occupied entry for in-place manipulation.\nIterates over all the entries in the <code>HashMap</code> to print them.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nGets an <code>OccupiedEntry</code> corresponding to the key for …\nGets a reference to the value in the entry.\nGets an <code>OccupiedEntry</code> corresponding to the key for …\nGets a mutable reference to the value in the entry.\nInserts a key-value pair into the <code>HashMap</code>.\nSets the value of the entry, and returns the old value.\nInserts a key-value pair into the <code>HashMap</code>.\nSets the value of the entry.\nSets the value of the entry with its key, and returns an …\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nTakes ownership of the key.\nReturns <code>true</code> if the <code>HashMap</code> is empty.\nReturns a reference to the key of this entry.\nGets a reference to the key in the entry.\nGets a reference to the key.\nReturns the number of entries in the <code>HashMap</code>.\nCreates an empty default <code>HashMap</code>.\nGets the next closest occupied entry.\nGets the next closest occupied entry.\nEnsures a value is in the entry by inserting the default …\nEnsures a value is in the entry by inserting the supplied …\nEnsures a value is in the entry by inserting the result of …\nEnsures a value is in the entry by inserting the result of …\nPrunes the entries specified by the predicate.\nPrunes the entries specified by the predicate.\nReads a key-value pair.\nReads a key-value pair.\nRemoves a key-value pair if the key exists.\nTakes the value out of the entry, and returns it.\nRemoves a key-value pair if the key exists.\nTakes ownership of the key and value from the <code>HashMap</code>.\nRemoves a key-value pair if the key exists and the given …\nRemoves a key-value pair if the key exists and the given …\nTemporarily increases the minimum capacity of the <code>HashMap</code>.\nRetains the entries specified by the predicate.\nRetains the entries specified by the predicate.\nScans all the entries.\nScans all the entries.\nUpdates an existing key-value pair in-place.\nUpdates an existing key-value pair in-place.\nUpserts a key-value pair into the <code>HashMap</code>.\nUpserts a key-value pair into the <code>HashMap</code>.\nCreates an empty <code>HashMap</code> with the specified capacity.\nCreates an empty <code>HashMap</code> with the specified capacity and …\nCreates an empty <code>HashMap</code> with the given <code>BuildHasher</code>.\nScalable concurrent hash set.\n<code>Reserve</code> keeps the capacity of the associated <code>HashSet</code> …\nSearches for any key that satisfies the given predicate.\nSearches for any key that satisfies the given predicate.\nReturns the index of the bucket that may contain the key.\nReturns the capacity of the <code>HashSet</code>.\nReturns the current capacity range of the <code>HashSet</code>.\nClears the <code>HashSet</code> by removing all keys.\nClears the <code>HashSet</code> by removing all keys.\nReturns <code>true</code> if the <code>HashSet</code> contains the specified key.\nReturns <code>true</code> if the <code>HashSet</code> contains the specified key.\nCreates an empty default <code>HashSet</code>.\nCompares two <code>HashSet</code> instances.\nReturns the argument unchanged.\nInserts a key into the <code>HashSet</code>.\nInserts a key into the <code>HashSet</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the <code>HashSet</code> is empty.\nReturns the number of entries in the <code>HashSet</code>.\nCreates an empty default <code>HashSet</code>.\nReads a key.\nReads a key.\nRemoves a key if the key exists.\nRemoves a key if the key exists.\nRemoves a key if the key exists and the given condition is …\nRemoves a key if the key exists and the given condition is …\nTemporarily increases the minimum capacity of the <code>HashSet</code>.\nRetains keys that satisfy the given predicate.\nRetains keys that satisfy the given predicate.\nScans all the keys.\nScans all the keys.\nCreates an empty <code>HashSet</code> with the specified capacity.\nCreates an empty <code>HashSet</code> with the specified capacity and …\nCreates an empty <code>HashSet</code> with the given <code>BuildHasher</code>.\nAn iterator over the entries of a <code>Queue</code>.\n<code>Queue</code> is a lock-free concurrent first-in-first-out …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the <code>Queue</code> is empty.\nReturns an <code>Iter</code>.\nReturns the number of entries in the <code>Queue</code>.\nReturns a guarded reference to the oldest entry.\nPeeks the oldest entry.\nPops the oldest entry.\nPops the oldest entry if the entry satisfies the given …\nPushes an instance of <code>T</code>.\nPushes an instance of <code>T</code> if the newest entry satisfies the …\nPushes an instance of <code>T</code> if the newest entry satisfies the …\nPushes an instance of <code>T</code> without checking the lifetime of <code>T</code>.\nAn iterator over the entries of a <code>Stack</code>.\n<code>Stack</code> is a lock-free concurrent last-in-first-out …\nReturns the argument unchanged.\nReturns the argument unchanged.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the <code>Stack</code> is empty.\nReturns an <code>Iter</code>.\nReturns the number of entries in the <code>Stack</code>.\nReturns a guarded reference to the newest entry.\nPeeks the newest entry.\nPops the newest entry.\nPops all the entries at once, and passes each one of the …\nPops the newest entry if the entry satisfies the given …\nPushes an instance of <code>T</code>.\nPushes an instance of <code>T</code> if the newest entry satisfies the …\nPushes an instance of <code>T</code> if the newest entry satisfies the …\nPushes an instance of <code>T</code> without checking the lifetime of <code>T</code>.\nAn iterator over the entries of a <code>TreeIndex</code>.\nAn iterator over a sub-range of entries in a <code>TreeIndex</code>.\nScalable concurrent B-plus tree.\nClears the <code>TreeIndex</code>.\nReturns <code>true</code> if the <code>TreeIndex</code> contains the key.\nCreates a <code>TreeIndex</code> with the default parameters.\nReturns the depth of the <code>TreeIndex</code>.\nReturns the argument unchanged.\nReturns the argument unchanged.\nReturns the argument unchanged.\nInserts a key-value pair.\nInserts a key-value pair.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nCalls <code>U::from(self)</code>.\nReturns <code>true</code> if the <code>TreeIndex</code> is empty.\nReturns an <code>Iter</code>.\nReturns the size of the <code>TreeIndex</code>.\nCreates an empty <code>TreeIndex</code>.\nReturns a guarded reference to the value for the specified …\nPeeks a key-value pair without acquiring locks.\nReturns a <code>Range</code> that scans keys in the given range.\nRemoves a key-value pair.\nRemoves a key-value pair.\nRemoves a key-value pair if the given condition is met.\nRemoves a key-value pair if the given condition is met.\nRemoves keys in the specified range.\nRemoves keys in the specified range.")