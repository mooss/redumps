package dumps

import (
	"bufio"
	"encoding/json"
	"fmt"
	"redumps/errs"
	"sort"
)

// FieldsProcessor counts occurrences of each field in JSON objects.
type FieldsProcessor struct {
	BaseProcessor
	fieldCounts  map[string]int
	totalObjects int
}

// Process reads each line, parses it as JSON, and counts fields.
func (p *FieldsProcessor) Process(scanner *bufio.Scanner) error {
	return p.process(scanner, p.processLine)
}

// Report prints the field counts in descending order.
func (p *FieldsProcessor) Report() {
	if p.fieldCounts == nil {
		fmt.Println("No fields found")
		return
	}

	// Convert to slice for sorting.
	type fieldStat struct {
		name  string
		count int
	}
	stats := make([]fieldStat, 0, len(p.fieldCounts))
	for field, count := range p.fieldCounts {
		stats = append(stats, fieldStat{name: field, count: count})
	}

	// Sort by count descending, then by name ascending.
	sort.Slice(stats, func(i, j int) bool {
		if stats[i].count != stats[j].count {
			return stats[i].count > stats[j].count
		}
		return stats[i].name < stats[j].name
	})

	fmt.Printf("\nField counts across %d objects:\n", p.count)
	for _, stat := range stats {
		fmt.Printf("  %s: %d\n", stat.name, stat.count)
	}
}

// processLine processes a single JSON line.
func (p *FieldsProcessor) processLine(line string) error {
	var obj map[string]any
	if err := json.Unmarshal([]byte(line), &obj); err != nil {
		return errs.Prefix(err, "parse JSON object")
	}

	if p.fieldCounts == nil {
		p.fieldCounts = make(map[string]int)
	}

	// Count each field in the object.
	for field := range obj {
		p.fieldCounts[field]++
	}

	p.IncrementCount(0) // Score is not relevant for field counting.
	return nil
}
