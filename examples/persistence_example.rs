use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use graphdb::storage::persistence::{
    CompressionType, FlushConfig, FlushManager, PageId, TableType,
};

fn main() {
    println!("=== Persistence Module Integration Example ===\n");

    example_dirty_page_tracking();
    example_compression();
    example_flush_manager();
    example_incremental_flush();
}

fn example_dirty_page_tracking() {
    println!("1. Dirty Page Tracking Example");
    println!("--------------------------------");

    let config = FlushConfig {
        flush_threshold: 100,
        flush_interval: Duration::from_secs(30),
        compression: CompressionType::None,
        background_flush_enabled: false,
        work_dir: PathBuf::from("./data"),
    };

    let flush_manager = FlushManager::new(config);

    let page1 = PageId {
        table_type: TableType::Vertex,
        label_id: 1,
        block_number: 0,
    };

    let page2 = PageId {
        table_type: TableType::Edge,
        label_id: 2,
        block_number: 1,
    };

    flush_manager.mark_dirty(page1);
    flush_manager.mark_dirty(page2);

    println!("Marked pages as dirty");
    println!("Dirty page count: {}", flush_manager.get_dirty_page_count());
    println!("Should flush: {}", flush_manager.should_flush());
    println!();
}

fn example_compression() {
    println!("2. Compression Example");
    println!("----------------------");

    let config = FlushConfig {
        compression: CompressionType::Zstd { level: 3 },
        ..Default::default()
    };

    let flush_manager = FlushManager::new(config);

    let data = b"This is a test string that will be compressed using Zstd algorithm. \
                 The compression should reduce the size of this repetitive data.";

    println!("Original data length: {} bytes", data.len());

    match flush_manager.compress_data(data) {
        Ok(compressed) => {
            println!("Compressed data length: {} bytes", compressed.len());
            println!("Compression ratio: {:.2}%", 
                (compressed.len() as f64 / data.len() as f64) * 100.0);

            match flush_manager.decompress_data(&compressed) {
                Ok(decompressed) => {
                    println!("Decompressed successfully: {}", 
                        decompressed.len() == data.len());
                }
                Err(e) => println!("Decompression error: {:?}", e),
            }
        }
        Err(e) => println!("Compression error: {:?}", e),
    }
    println!();
}

fn example_flush_manager() {
    println!("3. Flush Manager Example");
    println!("------------------------");

    let config = FlushConfig {
        flush_threshold: 10,
        flush_interval: Duration::from_secs(60),
        compression: CompressionType::Snappy,
        background_flush_enabled: false,
        work_dir: PathBuf::from("./data"),
    };

    let flush_manager = FlushManager::new(config);

    for i in 0..15 {
        let page_id = PageId {
            table_type: TableType::Vertex,
            label_id: 1,
            block_number: i,
        };
        flush_manager.mark_dirty(page_id);
    }

    println!("Marked 15 pages as dirty");
    println!("Dirty page count: {}", flush_manager.get_dirty_page_count());
    println!("Should flush (threshold=10): {}", flush_manager.should_flush());

    match flush_manager.flush_dirty_pages() {
        Ok(pages) => {
            println!("Flushed {} dirty pages", pages.len());
        }
        Err(e) => println!("Flush error: {:?}", e),
    }

    println!("Dirty page count after flush: {}", flush_manager.get_dirty_page_count());
    println!();
}

fn example_incremental_flush() {
    println!("4. Incremental Flush Example");
    println!("----------------------------");

    let config = FlushConfig {
        flush_threshold: 5,
        flush_interval: Duration::from_secs(1),
        compression: CompressionType::Zstd { level: 3 },
        background_flush_enabled: false,
        work_dir: PathBuf::from("./data"),
    };

    let flush_manager = FlushManager::new(config);

    println!("Simulating incremental writes...");

    for batch in 0..3 {
        println!("\nBatch {}:", batch + 1);

        for i in 0..3 {
            let page_id = PageId {
                table_type: TableType::Vertex,
                label_id: batch as u16,
                block_number: i,
            };
            flush_manager.mark_dirty(page_id);
        }

        println!("  Dirty pages: {}", flush_manager.get_dirty_page_count());

        if flush_manager.should_flush() {
            match flush_manager.flush_dirty_pages() {
                Ok(pages) => {
                    println!("  Flushed {} pages", pages.len());
                }
                Err(e) => println!("  Flush error: {:?}", e),
            }
        } else {
            println!("  No flush needed yet");
        }
    }

    if flush_manager.get_dirty_page_count() > 0 {
        match flush_manager.flush_dirty_pages() {
            Ok(pages) => {
                println!("\nFinal flush: {} pages", pages.len());
            }
            Err(e) => println!("Final flush error: {:?}", e),
        }
    }

    println!("\nExample completed successfully!");
}
