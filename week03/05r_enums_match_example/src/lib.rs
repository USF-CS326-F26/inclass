// ╔══════════════════════════════════════════════════════════════════════╗
// ║  05r — Enums, Option, and exhaustive match — IN-CLASS EXAMPLE        ║
// ║  Same shape as the exercise, different lifecycle: one slot of the    ║
// ║  buffer cache instead of one slot of the process table.              ║
// ╚══════════════════════════════════════════════════════════════════════╝
//
// Read this next to `exercises/05r_enums_match/skeleton/lib.rs`. Every item
// here has a twin there. If you can explain why the twins are the same shape,
// you have understood enums; the domain is decoration.
//
// The table here is deliberately shorter than the exercise's — five rows
// instead of seven — and the states are different. What carries across is the
// technique: one arm per variant with no `_`, a `match` on a tuple, a guard
// that lets a wrong request fall through, and an `Option` for "that is not a
// legal move".

/// What one buffer-cache slot currently holds.
///
/// UNDERSTAND: three of the four variants carry data. A cached buffer is
///   caching *something* — a block number — and a locked one also records who
///   locked it. A `Free` slot has nothing to say, so it carries nothing.
///
///   That is the whole difference from a C `enum`, which is an `int` wearing a
///   costume: `b->state = 47;` compiles, runs, and means nothing, and nothing
///   reminds you which of the twelve `switch` statements needs revisiting when
///   a fifth state appears.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufState {
    /// Holding nothing; available to be filled.
    Free,
    /// Holding a block, and it matches what is on disk.
    Clean { block: u32 },
    /// Holding a block that has been written to and not yet flushed.
    Dirty { block: u32 },
    /// Held by one process, which is using it right now.
    Locked { block: u32, by: u32 },
}

/// Something that happens *to* a buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufEvent {
    /// Read a block off the disk into this slot.
    Fill { block: u32 },
    /// Claim exclusive use of it, on behalf of process `by`.
    Lock { by: u32 },
    /// Give it back. Only the process holding it may do this.
    Unlock { by: u32 },
    /// Write through it, which makes it differ from the disk.
    Write,
    /// Flush it to disk, so it matches again.
    Flush,
    /// Throw it out of the cache, freeing the slot.
    Evict,
}

impl BufState {
    /// Does this slot hold data the cache could serve?
    ///
    /// UNDERSTAND: one `match` arm per variant and no `_` catch-all. Add a
    ///   fifth state to `BufState` later and this function stops compiling
    ///   until you say what the new state means. A compiler handing you that
    ///   to-do list is the whole reason `BufState` is an enum and not an int.
    pub fn holds_data(&self) -> bool {
        match self {
            BufState::Clean { .. } => true,
            BufState::Dirty { .. } => true,
            BufState::Locked { .. } => true,
            BufState::Free => false,
        }
    }

    /// Would evicting this slot lose data that is not on disk yet?
    ///
    /// UNDERSTAND: `matches!` is a `match` that answers yes or no. It is the
    ///   right tool when exactly one variant is interesting AND the question
    ///   really is "is it this one" — but note that it has the same blind spot
    ///   as `_`: a new variant is silently `false`.
    pub fn must_be_flushed_first(&self) -> bool {
        matches!(self, BufState::Dirty { .. })
    }
}

/// Where does `event` move a buffer that is currently in `state`?
///
/// UNDERSTAND: `None` means "that transition is illegal", and it is a value
///   the caller cannot ignore — there is no null to forget to check.
///
///   The legal moves, and nothing else:
///
/// ```text
///   Free                  Fill { block }      ->  Clean { block }
///   Clean { block }       Lock { by }         ->  Locked { block, by }
///   Locked { block, by }  Unlock { by: b }    ->  Clean { block }   if b == by
///   Locked { block, .. }  Write               ->  Dirty { block }
///   Dirty { block }       Flush               ->  Clean { block }
///   Clean { .. }          Evict               ->  Free
/// ```
pub fn next_state(state: BufState, event: BufEvent) -> Option<BufState> {
    match (state, event) {
        // A free slot is filled from disk, and now matches it.
        (BufState::Free, BufEvent::Fill { block }) => Some(BufState::Clean { block }),

        // Claiming a cached buffer records who holds it. The pattern binds
        // `block` out of the old state and `by` out of the event, and both go
        // into the new state.
        (BufState::Clean { block }, BufEvent::Lock { by }) => {
            Some(BufState::Locked { block, by })
        }

        // A GUARD. Only the holder may unlock. When `b != by` the guard fails
        // and matching CONTINUES with the arms below rather than leaving the
        // match — so somebody else's unlock falls through to `_ => None`.
        (BufState::Locked { block, by }, BufEvent::Unlock { by: b }) if b == by => {
            Some(BufState::Clean { block })
        }

        // Writing through a held buffer makes it differ from the disk.
        (BufState::Locked { block, .. }, BufEvent::Write) => {
            Some(BufState::Dirty { block })
        }

        // Flushing puts it back in agreement.
        (BufState::Dirty { block }, BufEvent::Flush) => Some(BufState::Clean { block }),

        // Only a clean, unheld buffer may be thrown away. Evicting a dirty one
        // would lose the write; evicting a locked one would pull it out from
        // under whoever is using it.
        (BufState::Clean { .. }, BufEvent::Evict) => Some(BufState::Free),

        // Everything else is illegal. This `_` is honest: the pairs it covers
        // are enumerable and we mean all of them. Note that it must come LAST
        // — `match` takes the first arm that fits, so a `_` placed earlier
        // would swallow everything below it.
        _ => None,
    }
}

/// Which block a slot is caching, if it is caching one.
///
/// UNDERSTAND: three variants carry a block, so this is a `match` rather than
///   an `if let`. `if let` is the right shape when exactly ONE variant matters
///   — see `holder` below.
pub fn cached_block(state: BufState) -> Option<u32> {
    match state {
        BufState::Clean { block } => Some(block),
        BufState::Dirty { block } => Some(block),
        BufState::Locked { block, .. } => Some(block),
        BufState::Free => None,
    }
}

/// Which process holds this buffer, if any.
///
/// UNDERSTAND: only one variant matters, so this is a one-arm `match` written
///   as `if let`. The pattern binds `by`; `{ .. }` on the same pattern would
///   say "this variant has fields and I do not need them".
pub fn holder(state: BufState) -> Option<u32> {
    if let BufState::Locked { by, .. } = state {
        Some(by)
    } else {
        None
    }
}

/// Apply `event`, or leave the buffer alone if the transition is illegal.
///
/// UNDERSTAND: this is the only place the `Option` from `next_state` is opened
///   up. Written out as a two-arm `match` so you can see what it does;
///   `.unwrap_or(state)` is the same thing spelled shorter — "the value inside
///   the `Some`, or this fallback if it was `None`".
pub fn step(state: BufState, event: BufEvent) -> BufState {
    match next_state(state, event) {
        Some(new_state) => new_state,
        None => state,
    }
}

// ---------------------------------------------------------------------------
// The same kind of contract the exercise has, rewritten for the buffer cache.
// Read these first: they say what everything above is for.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_a_filled_slot_holds_data() {
        assert!(!BufState::Free.holds_data());
        assert!(BufState::Clean { block: 3 }.holds_data());
        assert!(BufState::Dirty { block: 3 }.holds_data());
        assert!(BufState::Locked { block: 3, by: 1 }.holds_data());
    }

    #[test]
    fn only_a_dirty_buffer_has_to_be_flushed() {
        assert!(BufState::Dirty { block: 3 }.must_be_flushed_first());
        assert!(!BufState::Clean { block: 3 }.must_be_flushed_first());
        assert!(!BufState::Free.must_be_flushed_first());
        assert!(!BufState::Locked { block: 3, by: 1 }.must_be_flushed_first());
    }

    #[test]
    fn a_free_slot_is_filled_from_disk() {
        assert_eq!(
            next_state(BufState::Free, BufEvent::Fill { block: 7 }),
            Some(BufState::Clean { block: 7 })
        );
        // The block number travels out of the event and into the new state.
        assert_eq!(cached_block(step(BufState::Free, BufEvent::Fill { block: 7 })), Some(7));
    }

    #[test]
    fn an_event_that_is_legal_from_one_state_is_none_from_another() {
        // Flush makes sense for a buffer that differs from the disk, and
        // nowhere else.
        assert_eq!(
            next_state(BufState::Dirty { block: 7 }, BufEvent::Flush),
            Some(BufState::Clean { block: 7 })
        );
        assert_eq!(next_state(BufState::Clean { block: 7 }, BufEvent::Flush), None);
        assert_eq!(next_state(BufState::Free, BufEvent::Flush), None);

        // And a free slot cannot be locked before it holds anything.
        assert_eq!(next_state(BufState::Free, BufEvent::Lock { by: 1 }), None);
    }

    #[test]
    fn only_the_process_holding_a_buffer_may_unlock_it() {
        let held = BufState::Locked { block: 7, by: 3 };

        assert_eq!(
            next_state(held, BufEvent::Unlock { by: 3 }),
            Some(BufState::Clean { block: 7 })
        );
        // Process 4 does not hold it. The guard fails, matching continues,
        // nothing below fits, and `_ => None` answers.
        assert_eq!(next_state(held, BufEvent::Unlock { by: 4 }), None);
    }

    #[test]
    fn a_write_makes_a_held_buffer_differ_from_the_disk() {
        let held = BufState::Locked { block: 7, by: 3 };
        assert_eq!(next_state(held, BufEvent::Write), Some(BufState::Dirty { block: 7 }));

        // A buffer nobody holds cannot be written through.
        assert_eq!(next_state(BufState::Clean { block: 7 }, BufEvent::Write), None);
    }

    #[test]
    fn a_dirty_or_held_buffer_is_never_evicted() {
        // This is the property the whole example exists for. Evicting a dirty
        // buffer loses the write; evicting a held one pulls it out from under
        // whoever is using it.
        assert_eq!(next_state(BufState::Dirty { block: 7 }, BufEvent::Evict), None);
        assert_eq!(
            next_state(BufState::Locked { block: 7, by: 3 }, BufEvent::Evict),
            None
        );
        assert_eq!(
            next_state(BufState::Clean { block: 7 }, BufEvent::Evict),
            Some(BufState::Free)
        );
    }

    #[test]
    fn the_block_and_the_holder_come_back_out_of_the_state() {
        assert_eq!(cached_block(BufState::Clean { block: 7 }), Some(7));
        assert_eq!(cached_block(BufState::Dirty { block: 9 }), Some(9));
        assert_eq!(cached_block(BufState::Locked { block: 2, by: 1 }), Some(2));
        assert_eq!(cached_block(BufState::Free), None);

        assert_eq!(holder(BufState::Locked { block: 2, by: 5 }), Some(5));
        assert_eq!(holder(BufState::Clean { block: 2 }), None);
        assert_eq!(holder(BufState::Free), None);
    }

    #[test]
    fn an_impossible_event_leaves_the_buffer_exactly_where_it_was() {
        let dirty = BufState::Dirty { block: 7 };
        assert_eq!(step(dirty, BufEvent::Evict), dirty);
        assert_eq!(step(dirty, BufEvent::Lock { by: 1 }), dirty);
        assert_eq!(step(dirty, BufEvent::Flush), BufState::Clean { block: 7 });
    }

    #[test]
    fn a_buffer_walks_the_whole_lifecycle_and_ends_where_it_started() {
        let mut b = BufState::Free;

        b = step(b, BufEvent::Fill { block: 42 });
        assert_eq!(b, BufState::Clean { block: 42 });
        assert!(b.holds_data());

        b = step(b, BufEvent::Lock { by: 3 });
        assert_eq!(b, BufState::Locked { block: 42, by: 3 });
        assert_eq!(holder(b), Some(3));

        b = step(b, BufEvent::Write);
        assert_eq!(b, BufState::Dirty { block: 42 });
        assert!(b.must_be_flushed_first());

        // Note what the write cost: the buffer is no longer locked, so it
        // cannot be unlocked either. Every arrow the table leaves out is a
        // move the cache must refuse.
        assert_eq!(next_state(b, BufEvent::Unlock { by: 3 }), None);

        b = step(b, BufEvent::Flush);
        assert_eq!(b, BufState::Clean { block: 42 });
        assert!(!b.must_be_flushed_first());

        b = step(b, BufEvent::Evict);
        assert_eq!(b, BufState::Free);
        assert!(!b.holds_data());
    }
}
