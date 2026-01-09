package dumps

import (
	"fmt"
	"redumps/errs"
	"sort"

	"github.com/buger/jsonparser"
)

// FieldsProcessor counts occurrences of each field in JSON objects.
type FieldsProcessor struct {
	fieldCounts  map[string]int
	totalObjects int
}

// Process processes a single JSON line.
func (p *FieldsProcessor) Process(data []byte) error {
	if p.fieldCounts == nil {
		p.fieldCounts = make(map[string]int)
	}

	err := jsonparser.ObjectEach(data, func(key []byte, _ []byte, _ jsonparser.ValueType, _ int) error {
		p.fieldCounts[string(key)]++
		return nil
	})

	if err != nil {
		return errs.Prefix(err, "parse JSON fields")
	}

	p.totalObjects++
	return nil
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

	fmt.Printf("\nField counts across %d objects:\n", p.totalObjects)
	for _, stat := range stats {
		fmt.Printf("  %s: %d\n", stat.name, stat.count)
	}
}
