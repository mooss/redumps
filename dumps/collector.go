package dumps

import (
	"bufio"
	"fmt"
	"os"
	"sort"
)

// Collector helps processing data using a line-oriented pipeline: read, transform, accumulate.
// While running it silently tallies errors occurring during processing.
type Collector struct {
	errorCounts    map[string]int
	BytesProcessed int64
}

// Collect feeds every line from scanner into processor.
// Lines that processor rejects are tallied, not propagated.
func (coll *Collector) Collect(scanner *bufio.Scanner, processor func([]byte) error) error {
	if coll.errorCounts == nil {
		coll.errorCounts = make(map[string]int)
	}

	for scanner.Scan() {
		data := scanner.Bytes()
		if err := processor(data); err != nil {
			coll.ReportError(err)
		} else {
			coll.BytesProcessed += int64(len(data))
		}
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
