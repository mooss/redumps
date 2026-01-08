package dumps

import (
	"bufio"
	"encoding/json"
	"fmt"
	"redumps/errs"
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

type SubmissionStats struct {
	BaseProcessor
}

func (p *SubmissionStats) Process(scanner *bufio.Scanner) error {
	return p.process(scanner, p.processSubmission)
}

func (p *SubmissionStats) Report() {
	p.BaseProcessor.Report("submissions")
}

func (p *SubmissionStats) processSubmission(line string) error {
	var post RedditSubmission
	if err := json.Unmarshal([]byte(line), &post); err != nil {
		return errs.Prefix(err, "submission stats")
	}

	p.IncrementCount(post.Score)
	fmt.Printf("Submission #%d: %s (Score: %d)\n", p.count, post.Title, post.Score)
	return nil
}
