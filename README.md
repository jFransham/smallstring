# Small String

Create strings of any length on the stack, automatically upgrading to the heap
when they become larger than the buffer. Also allows converting from a `String`
for free (i.e. without copying to the stack even if they're small enough).

Backed by [`smallvec`](https://crates.io/crates/smallvec).

```rust
// Default maximum size to store on the stack: 8 bytes
let stack: SmallString = "Hello!".into();

// Reuses allocation
let heap: String = "Hello!".into();
let still_heap: SmallString = heap.into();
```
