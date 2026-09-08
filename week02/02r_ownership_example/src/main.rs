// The live demo. Run it, then read it:
//
//     cargo run
//
// Every line that moves a value is marked. The last section is a move error
// you can uncomment on screen — that is the part worth doing slowly.

use ownership_example::{claim_pid, measure_command, new_pid_pool, release_pid, NO_PID};

fn main() {
    println!("== a pool of 4 process IDs ==");

    // `new_pid_pool` builds a Vec inside itself and MOVES it out to us. From
    // here on, this `pool` binding is the one owner of that heap buffer.
    let pool = new_pid_pool(4);
    println!("free pids: {:?}", pool);

    println!("\n== spawn three processes ==");

    // Each call MOVES `pool` into `claim_pid` and takes ownership back out of
    // the returned tuple. The name `pool` on the left is a brand new binding
    // that shadows the old one -- which is dead anyway, having been moved.
    let (pool, sh) = claim_pid(pool);
    println!("spawned sh   as pid {sh:<2}  free pids: {pool:?}");

    let (pool, ls) = claim_pid(pool);
    println!("spawned ls   as pid {ls:<2}  free pids: {pool:?}");

    let (pool, grep) = claim_pid(pool);
    println!("spawned grep as pid {grep:<2}  free pids: {pool:?}");

    // Three live processes, three different PIDs, and not one of them is still
    // in the pool. Nobody had to remember to remove them: `remove` moved the
    // number out of the only pool that exists.
    assert!(sh != ls && ls != grep && sh != grep);

    println!("\n== reap ls, then spawn again ==");

    // `ls` is a usize, so this COPIES the number in. The binding survives.
    let pool = release_pid(pool, ls);
    println!("reaped pid {ls:<2}          free pids: {pool:?}");

    let (pool, reused) = claim_pid(pool);
    println!("spawned cat  as pid {reused:<2}  free pids: {pool:?}");
    println!("(pid {ls} came straight back out -- the pool is LIFO)");

    println!("\n== run the table dry ==");

    // One PID left. Claim it, then ask for one more.
    let (pool, last) = claim_pid(pool);
    println!("claimed the last pid {last},   free pids: {pool:?}");

    let (pool, none) = claim_pid(pool);
    if none == NO_PID {
        println!("one more claim -> NO_PID (this is fork returning -1)");
    }
    // Even when it fails, `claim_pid` hands the pool back. Had it not, the
    // pool would have been dropped inside the function and our process table
    // would be gone.
    println!("the pool is still ours, and still empty: {pool:?}");

    println!("\n== a String is moved, a usize is copied ==");

    let name = String::from("grep");
    // `name` is MOVED into measure_command. The only reason we can print it
    // below is that the function handed ownership back in the tuple.
    let (name, bytes) = measure_command(name);
    println!("command {name:?} is {bytes} bytes");

    // Ask the class to predict this one before you run it: `last` is still
    // usable after being passed around by value, because usize is Copy.
    let copied = last;
    println!("pid {last} copied into another binding: {copied}");

    println!("\n== the error worth reading out loud ==");
    println!("uncomment the block at the bottom of src/main.rs and rebuild");

    // ---------------------------------------------------------------------
    // UNCOMMENT ME in class. rustc says:
    //
    //   error[E0382]: borrow of moved value: `name`
    //      |     let (name, bytes) = measure_command(name);
    //      |          ---- move occurs because `name` has type `String`,
    //      |               which does not implement the `Copy` trait
    //      |     let taken = name;
    //      |                 ---- value moved here
    //      |     println!("{name}");
    //      |                ^^^^ value borrowed here after move
    //
    // "moved" means "given away". This is a use-after-free, reported before
    // the program has run even once.
    //
    // let taken = name;
    // println!("{name}");
    // println!("{taken}");
    // ---------------------------------------------------------------------
}
