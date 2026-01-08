package dumps

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"time"
)

type Processor struct {
	postCount         int
	commentCount      int
	totalPostScore    int
	totalCommentScore int
	startTime         time.Time
	errorCounts       map[string]int
}

func (p *Processor) ProcessLine(line string) error {
	itemType, err := detectType(line)
	if err != nil {
		return fmt.Errorf("detect type: %w", err)
	}

	switch itemType {
	case "post":
		score, err := processPost(line, p.postCount+1)
		if err != nil {
			return fmt.Errorf("process post: %w", err)
		}
		p.postCount++
		p.totalPostScore += score

	case "comment":
		score, err := processComment(line, p.commentCount+1)
		if err != nil {
			return fmt.Errorf("process comment: %w", err)
		}
		p.commentCount++
		p.totalCommentScore += score

	default:
		return fmt.Errorf("unknown item type: %s", line)
	}

	return nil
}

func (p *Processor) Process(scanner *bufio.Scanner) error {
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

func (p *Processor) Report() {
	postAvg := 0.0
	if p.postCount > 0 {
		postAvg = float64(p.totalPostScore) / float64(p.postCount)
	}

	commentAvg := 0.0
	if p.commentCount > 0 {
		commentAvg = float64(p.totalCommentScore) / float64(p.commentCount)
	}

	fmt.Printf("\nProcessed %d posts with total score %d (average: %.2f)\n",
		p.postCount, p.totalPostScore, postAvg)
	fmt.Printf("Processed %d comments with total score %d (average: %.2f)\n",
		p.commentCount, p.totalCommentScore, commentAvg)
	p.PrintErrorSummary()
}

func (p *Processor) PrintErrorSummary() {
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

///////////////////////
// Utility functions //

func truncate(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

// detectType checks for the presence of specific fields to determine the type.
func detectType(line string) (string, error) {
	// Check for "title" or "body" field by scanning the raw JSON.
	// This is faster than unmarshaling into a struct and should be correct while there is no nested
	// data.
	titleIdx := -1
	bodyIdx := -1

	for i := 0; i < len(line)-8; i++ {
		if line[i] == '"' && line[i+1:i+7] == "title\"" && line[i+7] == ':' &&
			(i == 0 || line[i-1] != '\\') /* Make sure the string is not escaped */ {
			titleIdx = i
			break
		}
	}

	for i := 0; i < len(line)-7; i++ {
		if line[i] == '"' && line[i+1:i+6] == "body\"" && line[i+6] == ':' &&
			(i == 0 || line[i-1] != '\\') /* Make sure the string is not escaped */ {
			bodyIdx = i
			break
		}
	}

	if titleIdx != -1 && (bodyIdx == -1 || titleIdx < bodyIdx) {
		return "post", nil
	}
	if bodyIdx != -1 {
		return "comment", nil
	}
	return "unknown", nil
}

func processPost(line string, postCount int) (int, error) {
	var post RedditSubmission
	if err := json.Unmarshal([]byte(line), &post); err != nil {
		return 0, err
	}
	fmt.Printf("Post #%d: %s (Score: %d)\n", postCount, post.Title, post.Score)
	return post.Score, nil
}

func processComment(line string, commentCount int) (int, error) {
	var comment RedditComment
	if err := json.Unmarshal([]byte(line), &comment); err != nil {
		return 0, err
	}
	bodyPreview := truncate(comment.Body, 50)
	fmt.Printf("Comment #%d by %s: %s (Score: %d)\n", commentCount, comment.Author, bodyPreview, comment.Score)
	return comment.Score, nil
}
