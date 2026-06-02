use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    // 1. Define the backends inside an Arc so tasks can read it.
    let backends = Arc::new(vec!["backend-1", "backend-2", "backend-3"]);

    // 2. Define a thread-safe counter to track request index for Round-Robin.
    // (An AtomicUsize is perfect here since we only need to increment an integer).
    let counter = Arc::new(AtomicUsize::new(0));

    println!("Simulating 5 requests...");

    for request_id in 1..=5 {
        // We need to clone the Arc pointers so each task gets its own copy of the pointer.
        let backends_clone = Arc::clone(&backends);
        let counter_clone = Arc::clone(&counter);

        // TODO: Spawn a tokio task for this request
        tokio::spawn(async move{
            let current_count = counter_clone.fetch_add(1, Ordering::SeqCst);
            println!("Sending rq to {}", backends_clone[(current_count as usize) % backends_clone.len()]);
        });
        // tokio::spawn(async move {
        //    1. Fetch and increment the counter safely.
        //    2. Determine the target backend using the modulo (%) operator.
        //    3. Print: "Request {request_id} routed to {backend}"
        // });
    }

    // Since spawned tasks run in the background, we need to wait a moment 
    // before main() exits, otherwise the program terminates before they run.
    sleep(Duration::from_millis(500)).await;
}
