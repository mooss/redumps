package dumps

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"sort"
)

type RedditSubmission struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	Author      string `json:"author"`
	Score       int    `json:"score"`
	NumComments int    `json:"num_comments"`
}

///////////////
// Processor //

type SubmissionProcessor struct {
	count       int
	scoreSum    int
	errorCounts map[string]int
}

func (p *SubmissionProcessor) ProcessLine(line string) error {
	score, err := p.processSubmission(line, p.count+1)
	if err != nil {
		return fmt.Errorf("process submission: %w", err)
	}
	p.count++
	p.scoreSum += score
	return nil
}

func (p *SubmissionProcessor) Process(scanner *bufio.Scanner) error {
	if p.errorCounts == nil {
		p.errorCounts = make(map[string]int)
	}

	for scanner.Scan() {
		line := scanner.Text()
		if err := p.ProcessLine(line); err != nil {
			p.errorCounts[err.Error()]++
			// Continue processing next lines.
		}
	}
	if err := scanner.Err(); err != nil {
		return fmt.Errorf("scanner error: %w", err)
	}
	return nil
}

func (p *SubmissionProcessor) Report() {
	postAvg := 0.0
	if p.count > 0 {
		postAvg = float64(p.scoreSum) / float64(p.count)
	}

	fmt.Printf("\nProcessed %d submissions with total score %d (average: %.2f)\n",
		p.count, p.scoreSum, postAvg)
	p.PrintErrorSummary()
}

func (p *SubmissionProcessor) PrintErrorSummary() {
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

func (p *SubmissionProcessor) processSubmission(line string, postCount int) (int, error) {
	var post RedditSubmission
	if err := json.Unmarshal([]byte(line), &post); err != nil {
		return 0, err
	}
	fmt.Printf("Submission #%d: %s (Score: %d)\n", postCount, post.Title, post.Score)
	return post.Score, nil
}

///////////////////////
// Utility functions //

func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}
