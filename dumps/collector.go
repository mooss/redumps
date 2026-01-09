package dumps

import (
	"bufio"
	"fmt"
	"os"
	"sort"
)

// Collector helps processing data using a line-oriented pipeline: read, transform, accumulate.
// While running it silently tallies errors occuring during processing.
type Collector struct {
	errorCounts map[string]int
}

// Collect feeds every line from scanner into processor.
// Lines that processor rejects are tallied, not propagated.
func (proc *Collector) Collect(scanner *bufio.Scanner, processor func(string) error) error {
	if proc.errorCounts == nil {
		proc.errorCounts = make(map[string]int)
	}

	for scanner.Scan() {
		line := scanner.Text()
		if err := processor(line); err != nil {
			proc.ReportError(err)
		}
	}

	if err := scanner.Err(); err != nil {
		return fmt.Errorf("scanner error: %w", err)
	}
	return nil
}

func (proc *Collector) ReportError(err error) {
	proc.errorCounts[err.Error()]++
}

// PrintErrorSummary dumps the tally of problems encountered.
func (proc *Collector) PrintErrorSummary() {
	if len(proc.errorCounts) == 0 {
		return
	}

	type kv struct {
		msg   string
		count int
	}
	var errs []kv
	for msg, cnt := range proc.errorCounts {
		errs = append(errs, kv{msg, cnt})
	}

	sort.Slice(errs, func(i, j int) bool { return errs[i].count > errs[j].count })
	fmt.Fprintf(os.Stderr, "\nError summary (most frequent first):\n")
	for _, e := range errs {
		fmt.Fprintf(os.Stderr, "%d occurrences: %s\n", e.count, e.msg)
	}
}
