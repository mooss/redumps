package dumps

import (
	"bufio"
	"fmt"
	"os"
	"sort"
)

// BaseProcessor provides common fields and methods for processing Reddit data.
type BaseProcessor struct {
	count       int
	scoreSum    int
	errorCounts map[string]int
}

// Process handles reading from scanner and processing each line.
func (p *BaseProcessor) process(scanner *bufio.Scanner, processor func(string) error) error {
	if p.errorCounts == nil {
		p.errorCounts = make(map[string]int)
	}

	for scanner.Scan() {
		line := scanner.Text()
		if err := processor(line); err != nil {
			p.errorCounts[err.Error()]++
			// Continue processing next lines.
		}
	}

	if err := scanner.Err(); err != nil {
		return fmt.Errorf("scanner error: %w", err)
	}
	return nil
}

// Report prints processing statistics.
func (p *BaseProcessor) Report(unitName string) {
	avg := 0.0
	if p.count > 0 {
		avg = float64(p.scoreSum) / float64(p.count)
	}

	fmt.Printf("\nProcessed %d %s with total score %d (average: %.2f)\n",
		p.count, unitName, p.scoreSum, avg)
	p.PrintErrorSummary()
}

// PrintErrorSummary prints error statistics.
func (p *BaseProcessor) PrintErrorSummary() {
	if len(p.errorCounts) == 0 {
		return
	}

	// Convert to slice for sorting.
	type kv struct {
		msg   string
		count int
	}
	var errs []kv
	for msg, cnt := range p.errorCounts {
		errs = append(errs, kv{msg, cnt})
	}

	sort.Slice(errs, func(i, j int) bool { return errs[i].count > errs[j].count })
	fmt.Fprintf(os.Stderr, "\nError summary (most frequent first):\n")
	for _, e := range errs {
		fmt.Fprintf(os.Stderr, "%d occurrences: %s\n", e.count, e.msg)
	}
}

// IncrementCount increments the count and adds to score sum.
func (p *BaseProcessor) IncrementCount(score int) {
	p.count++
	p.scoreSum += score
}

// truncate is a utility function to shorten strings for display.
func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}
