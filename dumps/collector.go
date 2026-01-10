package dumps

import (
	"bufio"
	"fmt"
	"os"
	"runtime"
	"sort"
	"sync"
)

// Collector helps processing data using a line-oriented pipeline: read, transform, accumulate.
// While running it silently tallies errors occurring during processing.
type Collector struct {
	errorCounts    map[string]int
	BytesProcessed int64
}

// Collect feeds every line from scanner into processor using a concurrent batched worker pool.
// Lines that processor rejects are tallied, not propagated.
func (coll *Collector) Collect(scanner *bufio.Scanner, processor func([]byte) error) error {
	if coll.errorCounts == nil {
		coll.errorCounts = make(map[string]int)
	}

	// Configure scanner for maximum throughput
	scanner.Split(bufio.ScanLines)

	// Higher batch size means higher performance but also higher memory usage.
	// See tradeoff example below (not a serious benchmark).
	//
	// | Batch size | Peak memory usage | Throughput |
	// |------------+-------------------+------------|
	// |         32 |           ~30 MiB | ~550 MiB/s |
	// |        512 |            ~90MiB | ~680 MiB/s |
	batchSize := 512

	// Channel for lines from producer to batcher.
	lines := make(chan []byte, 16384)

	// Channel for batches from batcher to workers.
	batches := make(chan [][]byte, 64)

	// Error channel for collecting errors from workers.
	errCh := make(chan error, 16)

	var wg sync.WaitGroup

	// Step 1: Producer - read lines from scanner
	wg.Go(func() {
		defer close(lines)
		for scanner.Scan() {
			// Copy bytes to avoid scanner buffer issues
			data := make([]byte, len(scanner.Bytes()))
			copy(data, scanner.Bytes())
			lines <- data
		}
	})

	// Step 2: Batcher - gather lines into batches
	wg.Go(func() {
		defer close(batches)
		batch := make([][]byte, 0, batchSize)
		for line := range lines {
			batch = append(batch, line)
			if len(batch) >= batchSize {
				// Create a copy of the batch to send
				batchCopy := make([][]byte, len(batch))
				copy(batchCopy, batch)
				batches <- batchCopy
				batch = batch[:0]
			}
		}
		// Send remaining lines as final batch
		if len(batch) > 0 {
			batchCopy := make([][]byte, len(batch))
			copy(batchCopy, batch)
			batches <- batchCopy
		}
	})

	// Step 3: Worker pool - process batches concurrently
	workerCount := runtime.GOMAXPROCS(0) * 2 // Double max procs seems to be a sweet spot.
	for range workerCount {
		wg.Go(func() {
			for batch := range batches {
				for _, data := range batch {
					if err := processor(data); err != nil {
						errCh <- err
					} else {
						coll.BytesProcessed += int64(len(data))
					}
				}
			}
		})
	}

	// Wait for all stages to complete
	go func() {
		wg.Wait()
		close(errCh)
	}()

	// Collect errors.
	for err := range errCh {
		coll.ReportError(err)
	}

	if err := scanner.Err(); err != nil {
		return fmt.Errorf("scanner error: %w", err)
	}
	return nil
}

func (coll *Collector) ReportError(err error) {
	coll.errorCounts[err.Error()]++
}

// PrintErrorSummary dumps the tally of problems encountered.
func (coll *Collector) PrintErrorSummary() {
	if len(coll.errorCounts) == 0 {
		return
	}

	type kv struct {
		msg   string
		count int
	}
	var errs []kv
	for msg, cnt := range coll.errorCounts {
		errs = append(errs, kv{msg, cnt})
	}

	sort.Slice(errs, func(i, j int) bool { return errs[i].count > errs[j].count })
	fmt.Fprintf(os.Stderr, "\nError summary (most frequent first):\n")
	for _, e := range errs {
		fmt.Fprintf(os.Stderr, "%d occurrences: %s\n", e.count, e.msg)
	}
}
