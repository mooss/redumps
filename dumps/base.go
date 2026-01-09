package dumps

import (
	"bufio"
	"fmt"
	"os"
	"sort"
)

// BaseProcessor provides common fields and methods for processing Reddit data.
type BaseProcessor struct {
	errorCounts map[string]int
}

// Collect applies a processing function to each line of the given scanner.
func (proc *BaseProcessor) Collect(scanner *bufio.Scanner, processor func(string) error) error {
	if proc.errorCounts == nil {
		proc.errorCounts = make(map[string]int)
	}

	for scanner.Scan() {
		line := scanner.Text()
		if err := processor(line); err != nil {
			// Parsing can be error-prone when the data is not sanitized, so a processing error is
			// not fatal.
			proc.ReportError(err)
		}
	}

	if err := scanner.Err(); err != nil {
		return fmt.Errorf("scanner error: %w", err)
	}
	return nil
}

func (proc *BaseProcessor) ReportError(err error) {
	proc.errorCounts[err.Error()]++
}

// PrintErrorSummary prints error statistics.
func (proc *BaseProcessor) PrintErrorSummary() {
	if len(proc.errorCounts) == 0 {
		return
	}

	// Convert to slice for sorting.
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
