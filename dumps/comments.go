package dumps

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"sort"
)

type RedditComment struct {
	ID         string `json:"id"`
	Author     string `json:"author"`
	Body       string `json:"body"`
	Score      int    `json:"score"`
	ParentID   string `json:"parent_id"`
	LinkID     string `json:"link_id"`
	Subreddit  string `json:"subreddit"`
	CreatedUTC string `json:"created_utc"`
}

// UnmarshalJSON is a custom unmarshaller handling the fact that CreatedUTC is sometimes a string
// and sometimes a number.
func (c *RedditComment) UnmarshalJSON(data []byte) error {
	type Alias RedditComment
	temp := &struct {
		CreatedUTC any `json:"created_utc"`
		*Alias
	}{
		Alias: (*Alias)(c),
	}

	if err := json.Unmarshal(data, &temp); err != nil {
		return err
	}

	switch v := temp.CreatedUTC.(type) {
	case string:
		c.CreatedUTC = v
	case float64:
		c.CreatedUTC = fmt.Sprintf("%.0f", v)
	case int:
		c.CreatedUTC = fmt.Sprintf("%d", v)
	case nil:
		c.CreatedUTC = ""
	default: // At worse, try to coerce to string.
		c.CreatedUTC = fmt.Sprintf("%v", v)
	}

	return nil
}

///////////////
// Processor //

type CommentProcessor struct {
	count       int
	scoreSum    int
	errorCounts map[string]int
}

func (p *CommentProcessor) ProcessLine(line string) error {
	score, err := p.processComment(line, p.count+1)
	if err != nil {
		return fmt.Errorf("process comment: %w", err)
	}
	p.count++
	p.scoreSum += score
	return nil
}

func (p *CommentProcessor) Process(scanner *bufio.Scanner) error {
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

func (p *CommentProcessor) Report() {
	commentAvg := 0.0
	if p.count > 0 {
		commentAvg = float64(p.scoreSum) / float64(p.count)
	}

	fmt.Printf("\nProcessed %d comments with total score %d (average: %.2f)\n",
		p.count, p.scoreSum, commentAvg)
	p.PrintErrorSummary()
}

func (p *CommentProcessor) PrintErrorSummary() {
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

func (p *CommentProcessor) processComment(line string, commentCount int) (int, error) {
	var comment RedditComment
	if err := json.Unmarshal([]byte(line), &comment); err != nil {
		return 0, err
	}
	bodyPreview := truncate(comment.Body, 50)
	fmt.Printf("Comment #%d by %s: %s (Score: %d)\n", commentCount, comment.Author, bodyPreview, comment.Score)
	return comment.Score, nil
}
