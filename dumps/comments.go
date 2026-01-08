package dumps

import (
	"bufio"
	"encoding/json"
	"fmt"
	"redumps/errs"
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
	BaseProcessor
}

func (p *CommentProcessor) Process(scanner *bufio.Scanner) error {
	return p.process(scanner, p.processComment)
}

func (p *CommentProcessor) Report() {
	p.BaseProcessor.Report("comments")
}

func (p *CommentProcessor) processComment(line string) error {
	var comment RedditComment
	if err := json.Unmarshal([]byte(line), &comment); err != nil {
		return errs.Prefix(err, "process comment")
	}

	bodyPreview := truncate(comment.Body, 50)
	p.IncrementCount(comment.Score)
	fmt.Printf("Comment #%d by %s: %s (Score: %d)\n", p.count, comment.Author, bodyPreview, comment.Score)
	return nil
}
