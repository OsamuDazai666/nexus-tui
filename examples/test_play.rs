#[tokio::main]
async fn main() {
    let id = "NjDRdEnhdknWHcfiv"; // Ghost Stories
    let ep = 1;
    let mode = "sub";
    let quality = "1080p";

    println!("Fetching stream for {}, ep {}...", id, ep);
    match nexus_tui::player::stream_anime(id, ep, "Ghost Stories", mode, quality).await {
        Ok(_) => println!("Success! mpv spawned."),
        Err(e) => println!("Error resolving stream: {}", e),
    }
}
