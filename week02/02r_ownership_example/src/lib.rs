// ╔══════════════════════════════════════════════════════════════════════╗
// ║  02r — Ownership and Moves — IN-CLASS EXAMPLE                        ║
// ║  Same shape as the exercise, different Vec: a pool of free process   ║
// ║  IDs instead of a free list of physical pages.                       ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/02r_ownership/solution/lib.rs`. Every function
// here has a twin there. If you can explain why the twins are the same shape,
// you have understood ownership; the domain (pages vs. PIDs) is decoration.

/// Returned by `claim_pid` when every process ID is in use.
///
/// UNDERSTAND: xv6 numbers processes from 1, and `fork` returns -1 when the
///   process table is full. We are working in `usize`, which has no -1, so we
///   reserve `usize::MAX` to mean "nothing here" instead. A value set aside to
///   mean "nothing" is a sentinel — the same trick as `NO_PAGE` in the
///   exercise, and as the null pointer `kalloc` returns in exercise 32k.
pub const NO_PID: usize = usize::MAX;

/// Build a pool holding every process ID from 1 up to and including `count`.
///
/// UNDERSTAND: a `Vec<usize>` is a growable list of `usize` values whose
///   elements live in a buffer on the heap. The `Vec` value OWNS that buffer:
///   exactly one binding is responsible for it at a time, and when that
///   binding goes away the buffer is released. Returning the Vec hands
///   ownership to the caller.
///
/// NOTE the off-by-one difference from `new_free_list`: page numbers start at
///   0, but PID 0 is reserved for the kernel's own scheduler process, so this
///   pool starts at 1. Ownership does not care either way — but you do, and
///   the test at the bottom of this file is what pins it down.
pub fn new_pid_pool(count: usize) -> Vec<usize> {
    let mut pool = Vec::new();
    let mut pid = 1;
    while pid <= count {
        pool.push(pid);
        pid += 1;
    }
    // `pool` is moved out of this function and into the caller's binding.
    pool
}

/// Spawn a process: take the last PID out of the pool and return it along
/// with the (now smaller) pool.
///
/// UNDERSTAND: the parameter is `pool: Vec<usize>` with no `&`, so calling
///   this function MOVES the caller's pool into it. The caller's old binding
///   is dead from that point on — the compiler refuses to let anyone use it.
///   That is the whole point: two processes holding the same PID is exactly
///   the bug this example is about. Since the caller does still need the pool
///   back, we return it as half of a tuple.
pub fn claim_pid(mut pool: Vec<usize>) -> (Vec<usize>, usize) {
    if pool.is_empty() {
        // The process table is full. Give the (empty) pool back anyway — we
        // own it, and dropping it here would destroy the caller's pool.
        return (pool, NO_PID);
    }
    let last = pool.len() - 1;
    let pid = pool.remove(last);
    (pool, pid)
}

/// Reap a process: put `pid` back in the pool and return the pool.
///
/// UNDERSTAND: this is the mirror image of `claim_pid`. The pool is moved in
///   and moved back out, so at every instant exactly one place owns it.
pub fn release_pid(mut pool: Vec<usize>, pid: usize) -> Vec<usize> {
    // `pid` is a usize, which is Copy: this stores a copy of the number and
    // leaves the caller's binding alone.
    pool.push(pid);
    pool
}

/// Measure a process's command name: report its length in bytes, and hand the
/// name back to the caller.
///
/// UNDERSTAND: a `String` owns a heap buffer just as a `Vec` does, so a
///   `String` argument is moved, not copied. This "take it, use it, give it
///   back" shape is what you are stuck with until you learn borrowing in 03r.
pub fn measure_command(name: String) -> (String, usize) {
    // The length has to be read BEFORE the tuple is built. `(name,
    // name.len())` moves `name` into the tuple first and then fails to
    // compile, because you cannot use a value you have already given away.
    let bytes = name.len();
    (name, bytes)
}

// ---------------------------------------------------------------------------
// The same test contract as the exercise, rewritten for PIDs. Read these
// first: they say what the four functions above are for.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_pool_holds_every_pid() {
        // PID 0 belongs to the kernel, so the pool starts at 1.
        assert_eq!(new_pid_pool(4), vec![1, 2, 3, 4]);
        assert_eq!(new_pid_pool(1), vec![1]);
        assert!(new_pid_pool(0).is_empty());
    }

    #[test]
    fn claiming_a_pid_removes_it_from_the_pool() {
        let pool = new_pid_pool(4);
        let (pool, pid) = claim_pid(pool);
        assert_eq!(pid, 4);
        assert_eq!(pool, vec![1, 2, 3]);
    }

    #[test]
    fn an_exhausted_pool_hands_out_no_pid() {
        let (pool, pid) = claim_pid(new_pid_pool(0));
        assert_eq!(pid, NO_PID);
        assert!(pool.is_empty());

        // Reap one process into that empty pool and its PID can be handed out
        // again — and then the pool is exhausted once more.
        let pool = release_pid(pool, 7);
        let (pool, pid) = claim_pid(pool);
        assert_eq!(pid, 7);
        let (pool, pid) = claim_pid(pool);
        assert_eq!(pid, NO_PID);
        assert!(pool.is_empty());
    }

    #[test]
    fn releasing_a_pid_makes_it_available_again() {
        let pool = new_pid_pool(3);
        let (pool, pid) = claim_pid(pool);
        assert_eq!(pool.len(), 2);

        let pool = release_pid(pool, pid);
        assert_eq!(pool.len(), 3);

        // Last one reaped is the first one spawned again. The kernel's free
        // lists are LIFO for the same reason: the end of a Vec is the only
        // place you can add or remove without shuffling everything else.
        let (_pool, again) = claim_pid(pool);
        assert_eq!(again, pid);
    }

    #[test]
    fn the_same_pid_is_never_claimed_twice() {
        // The property this whole example exists for. Once a PID has been
        // claimed it is gone from the pool, so it cannot be claimed by a
        // second process. Two processes sharing a PID means `kill` kills the
        // wrong one and `wait` waits on the wrong one.
        let pool = new_pid_pool(3);
        let (pool, first) = claim_pid(pool);
        let (pool, second) = claim_pid(pool);
        let (pool, third) = claim_pid(pool);
        assert_ne!(first, second);
        assert_ne!(second, third);
        assert_ne!(first, third);
        assert!(pool.is_empty());

        let (_pool, nothing_left) = claim_pid(pool);
        assert_eq!(nothing_left, NO_PID);
    }

    #[test]
    fn a_pid_is_copied_not_moved() {
        let pool = new_pid_pool(4);
        let (pool, pid) = claim_pid(pool);

        // `pid` is a usize, and usize is Copy: handing it to `release_pid`
        // copies the number instead of moving it, so this binding is still
        // perfectly usable on the next line.
        let pool = release_pid(pool, pid);
        assert_eq!(pid, 4);
        assert_eq!(pool, vec![1, 2, 3, 4]);
    }

    #[test]
    fn a_moved_command_has_to_be_handed_back() {
        let name = String::from("sh");

        // `measure_command` takes the String by value, so `name` is MOVED
        // into it. The only reason the text is still reachable on the next
        // line is that the function handed ownership back.
        let (name, bytes) = measure_command(name);
        assert_eq!(bytes, 2);
        assert_eq!(name, "sh");

        let (name, bytes) = measure_command(name);
        assert_eq!(bytes, 2);
        assert_eq!(name, "sh");

        let (empty, bytes) = measure_command(String::new());
        assert_eq!(bytes, 0);
        assert!(empty.is_empty());
    }
}
